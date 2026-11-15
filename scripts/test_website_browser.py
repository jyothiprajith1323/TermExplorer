#!/usr/bin/env python3
"""Offline responsive rendering smoke tests using Python Playwright + Chromium."""
from __future__ import annotations
import base64
import shutil
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1] / "website"
PAGES = ["index.html", "features.html", "architecture.html", "docs.html", "roadmap.html"]
SIZES = [(320, 640), (360, 800), (390, 844), (430, 932), (600, 900), (768, 1024), (820, 1180), (844, 390), (860, 1000), (861, 1000), (932, 430), (1024, 768), (1100, 800), (1200, 600), (1366, 768), (1600, 900), (1920, 1080), (2560, 1440), (3440, 1440), (3840, 2160)]


def inline_page(name: str) -> str:
    html = (ROOT / name).read_text(encoding="utf-8")
    css = (ROOT / "assets/style.css").read_text(encoding="utf-8")
    js = (ROOT / "assets/app.js").read_text(encoding="utf-8")
    html = html.replace('<link rel="stylesheet" href="assets/style.css">', f"<style>{css}</style>")
    html = html.replace('<script src="assets/app.js" defer></script>', f"<script>{js}</script>")
    for asset in ("termexplorer-mark.svg", "termexplorer-ui.svg", "architecture-diagram.svg"):
        path = ROOT / "assets" / asset
        if path.exists():
            encoded = base64.b64encode(path.read_bytes()).decode("ascii")
            uri = f"data:image/svg+xml;base64,{encoded}"
            html = html.replace(f'assets/{asset}', uri)
    return html


def main() -> int:
    chrome = shutil.which("chromium") or shutil.which("chromium-browser") or shutil.which("google-chrome")
    if not chrome:
        print("SKIP: no Chromium executable found", file=sys.stderr)
        return 2
    passed = 0
    errors: list[str] = []
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True, executable_path=chrome, args=["--no-sandbox"])
            page = browser.new_page()
            for route in PAGES:
                document = inline_page(route)
                for width, height in SIZES:
                    page.set_viewport_size({"width": width, "height": height})
                    page.set_content(document, wait_until="load")
                    if page.locator("main").count() != 1 or page.locator("h1").count() != 1:
                        errors.append(f"{route} {width}x{height}: main/h1 structural failure")
                    dimensions = page.evaluate("""() => ({
                      viewport: document.documentElement.clientWidth,
                      document: document.documentElement.scrollWidth,
                      body: document.body.scrollWidth,
                      images: [...document.images].filter(i => !i.complete || i.naturalWidth === 0).map(i => i.alt || i.src),
                      overflowImages: [...document.images].filter(i => i.getBoundingClientRect().width > document.documentElement.clientWidth + 1).map(i => i.alt || i.src)
                    })""")
                    if dimensions["document"] > dimensions["viewport"] + 1 or dimensions["body"] > dimensions["viewport"] + 1:
                        errors.append(f"{route} {width}x{height}: horizontal overflow {dimensions}")
                    if dimensions["images"]:
                        errors.append(f"{route} {width}x{height}: broken images {dimensions['images']}")
                    if dimensions["overflowImages"]:
                        errors.append(f"{route} {width}x{height}: oversized images {dimensions['overflowImages']}")
                    passed += 1
            page.set_viewport_size({"width": 360, "height": 800})
            page.set_content(inline_page("index.html"), wait_until="load")
            toggle = page.locator(".menu-toggle")
            toggle.click()
            if toggle.get_attribute("aria-expanded") != "true" or not page.locator("#primary-nav").is_visible():
                errors.append("Mobile navigation did not open")
            page.keyboard.press("Escape")
            if toggle.get_attribute("aria-expanded") != "false" or not toggle.evaluate("el => document.activeElement === el"):
                errors.append("Mobile navigation did not close/focus correctly on Escape")
            # High-DPI checks use CSS viewport sizes with 2x and 3x device scale factors.
            for dpr in (2, 3):
                hi_dpi = browser.new_page(viewport={"width": 390, "height": 844}, device_scale_factor=dpr)
                hi_dpi.set_content(inline_page("index.html"), wait_until="load")
                dims = hi_dpi.evaluate("({vw: innerWidth, doc: document.documentElement.scrollWidth})")
                if dims["doc"] > dims["vw"] + 1:
                    errors.append(f"High-DPI {dpr}x viewport overflow: {dims}")
                hi_dpi.close()
            page.set_viewport_size({"width": 1440, "height": 900})
            page.set_content(inline_page("index.html"), wait_until="load")
            page.screenshot(path="/mnt/data/termexplorer-home-desktop.png", full_page=True)
            page.set_viewport_size({"width": 390, "height": 844})
            page.set_content(inline_page("index.html"), wait_until="load")
            page.screenshot(path="/mnt/data/termexplorer-home-mobile.png", full_page=True)
            browser.close()
    except Exception as exc:
        print(f"Browser test could not run: {exc}", file=sys.stderr)
        return 2
    print(f"Responsive page/viewport combinations passed: {passed}/{len(PAGES) * len(SIZES)}")
    print("Mobile navigation checks: " + ("passed" if not errors else "see failures"))
    if errors:
        for error in errors:
            print("FAIL:", error)
        return 1
    print("Browser responsive smoke tests passed")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
