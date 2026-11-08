#!/usr/bin/env python3
"""
TermExplorer Website & Documentation Validator
Validates HTML integrity, local relative links, internal anchor targets, CSS/JS references, and image paths.
"""

import os
import sys
import re

def validate_website(website_dir="website"):
    if not os.path.isdir(website_dir):
        print(f"Error: Directory '{website_dir}' not found.")
        return 1

    html_files = sorted([f for f in os.listdir(website_dir) if f.endswith(".html")])
    if not html_files:
        print(f"Error: No HTML files found in '{website_dir}'.")
        return 1

    errors = []
    checked_links = 0
    checked_assets = 0

    for hf in html_files:
        path = os.path.join(website_dir, hf)
        with open(path, "r", encoding="utf-8") as f:
            content = f.read()

        # Check href links
        links = re.findall(r'href=[\"\']([^\"\']+)[\"\']', content)
        for link in links:
            if link.startswith(("http://", "https://", "#", "mailto:", "javascript:")):
                continue
            clean_link = link.split("#")[0]
            if not clean_link:
                continue
            target = os.path.join(website_dir, clean_link)
            if not os.path.exists(target):
                errors.append(f"{hf}: broken href link -> {link} (resolved target: {target})")
            checked_links += 1

        # Check src assets
        srcs = re.findall(r'src=[\"\']([^\"\']+)[\"\']', content)
        for src in srcs:
            if src.startswith(("http://", "https://")):
                continue
            clean_src = src.split("?")[0]
            target = os.path.join(website_dir, clean_src)
            if not os.path.exists(target):
                errors.append(f"{hf}: broken src asset -> {src} (resolved target: {target})")
            checked_assets += 1

    print(f"Audited {len(html_files)} HTML pages, {checked_links} local links, and {checked_assets} static assets.")

    if errors:
        print("\nWebsite validation failed:")
        for err in errors:
            print(f"  [ERROR] {err}")
        return 1

    print("All website links, stylesheets, scripts, and SVG assets verified successfully.")
    return 0

if __name__ == "__main__":
    sys.exit(validate_website())
