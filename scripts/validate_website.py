#!/usr/bin/env python3
"""Static GitHub Pages checks: local links, fragments, assets and basic semantics."""
from __future__ import annotations
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1] / "website"

class Document(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.ids: set[str] = set()
        self.links: list[tuple[str, str, str, str]] = []
        self.assets: list[tuple[str, str]] = []
        self.h1 = 0
        self.main = 0
        self.title = 0
        self.html_lang = False
        self.viewport_meta = False
        self.images_without_alt: list[str] = []
        self.buttons_without_name = 0

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "html": self.html_lang = bool(a.get("lang"))
        if tag == "meta" and a.get("name", "").lower() == "viewport": self.viewport_meta = bool(a.get("content"))
        if a.get("id"): self.ids.add(a["id"])
        if tag in ("a", "link") and a.get("href"):
            self.links.append((tag, a["href"], a.get("target", ""), a.get("rel", "")))
        if tag in ("img", "script") and a.get("src"):
            self.assets.append((tag, a["src"]))
        if tag == "img" and "alt" not in a:
            self.images_without_alt.append(a.get("src", "(missing src)"))
        if tag == "h1": self.h1 += 1
        if tag == "main": self.main += 1
        if tag == "title": self.title += 1
        if tag == "button":
            self._button_text = True
            self._button_aria = bool(a.get("aria-label") or a.get("title"))
            self._button_content = ""

    def handle_data(self, data):
        if getattr(self, "_button_text", False): self._button_content += data

    def handle_endtag(self, tag):
        if tag == "button" and getattr(self, "_button_text", False):
            if not self._button_aria and not self._button_content.strip(): self.buttons_without_name += 1
            self._button_text = False


def main() -> int:
    if not ROOT.is_dir():
        print(f"Missing website directory: {ROOT}", file=sys.stderr)
        return 1
    html_files = sorted(ROOT.glob("*.html"))
    if not html_files:
        print("No HTML pages found", file=sys.stderr)
        return 1
    errors: list[str] = []
    checked = 0
    for file in html_files:
        parser = Document()
        source = file.read_text(encoding="utf-8")
        try:
            parser.feed(source)
            parser.close()
        except Exception as exc:
            errors.append(f"{file.name}: HTML parser error: {exc}")
            continue
        for ok, message in [
            (parser.html_lang, "missing html lang attribute"),
            (parser.viewport_meta, "missing viewport meta tag"),
            (parser.h1 == 1, f"expected exactly one h1, found {parser.h1}"),
            (parser.main == 1, f"expected exactly one main landmark, found {parser.main}"),
            (parser.title == 1, f"expected exactly one title element, found {parser.title}"),
            (not parser.images_without_alt, f"images missing alt attribute: {parser.images_without_alt}"),
            (parser.buttons_without_name == 0, f"{parser.buttons_without_name} button(s) have no accessible name"),
        ]:
            if not ok: errors.append(f"{file.name}: {message}")

        for tag, raw, target_attr, rel_attr in parser.links:
            checked += 1
            if target_attr == "_blank" and "noopener" not in rel_attr.split():
                errors.append(f"{file.name}: external new-tab link lacks rel=noopener: {raw}")
            if raw.startswith(("https://", "http://", "mailto:", "tel:", "javascript:")):
                continue
            parsed = urlsplit(raw)
            target_name = unquote(parsed.path)
            target = (file.parent / target_name).resolve() if target_name else file.resolve()
            try:
                target.relative_to(ROOT.resolve())
            except ValueError:
                errors.append(f"{file.name}: link escapes website root: {raw}")
                continue
            if not target.exists():
                errors.append(f"{file.name}: broken local {tag} href: {raw}")
                continue
            if parsed.fragment and target.suffix.lower() in (".html", ""):
                dest = Document()
                try:
                    dest.feed(target.read_text(encoding="utf-8")); dest.close()
                    if unquote(parsed.fragment) not in dest.ids:
                        errors.append(f"{file.name}: missing fragment target: {raw}")
                except OSError as exc:
                    errors.append(f"{file.name}: unable to inspect fragment target {raw}: {exc}")

        for tag, raw in parser.assets:
            checked += 1
            if raw.startswith(("https://", "http://", "data:")): continue
            target = (file.parent / unquote(urlsplit(raw).path)).resolve()
            if not target.is_file(): errors.append(f"{file.name}: missing local {tag} source: {raw}")

    required = [ROOT / "assets/style.css", ROOT / "assets/app.js", ROOT / "assets/termexplorer-mark.svg", ROOT / "assets/termexplorer-ui.svg", ROOT / "assets/architecture-diagram.svg"]
    for asset in required:
        if not asset.is_file(): errors.append(f"Missing required asset: {asset.relative_to(ROOT)}")
    css = ROOT / "assets/style.css"
    if css.is_file():
        text = css.read_text(encoding="utf-8")
        if text.count("{") != text.count("}"): errors.append("CSS: unbalanced braces")
        for breakpoint in ("860px", "640px", "380px", "1800px", "2560px"):
            if breakpoint not in text: errors.append(f"CSS: expected responsive breakpoint {breakpoint} missing")
        if "prefers-reduced-motion" not in text: errors.append("CSS: reduced-motion preference is not handled")
        if "min-width: 320px" not in text: errors.append("CSS: minimum supported viewport width must be explicit (320px)")
    js = ROOT / "assets/app.js"
    if js.is_file():
        text = js.read_text(encoding="utf-8")
        if "aria-expanded" not in text or "Escape" not in text: errors.append("JavaScript: accessible menu state and Escape handling are required")

    print(f"Checked {len(html_files)} pages and {checked} local references under {ROOT}")
    if errors:
        print("Website validation failed:")
        for error in errors: print(f"  ERROR: {error}")
        return 1
    print("Static website validation passed")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
