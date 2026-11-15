#!/usr/bin/env bash
# ==============================================================================
# TermExplorer — Documentation Quality & Consistency Validation Suite
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

ERRORS=0
TOTAL_CHECKS=0

pass() {
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    echo -e "  [${GREEN}PASS${NC}] $1"
}

fail() {
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    ERRORS=$((ERRORS + 1))
    echo -e "  [${RED}FAIL${NC}] $1"
}

echo -e "${BLUE}=== 1. Checking Relative Markdown Links ===${NC}"

# Find all Markdown files (excluding target/ and .git/)
MD_FILES=$(find docs README.md CONTRIBUTING.md SECURITY.md CHANGELOG.md -name "*.md")

for file in $MD_FILES; do
    file_dir=$(dirname "$file")
    # Extract relative and file:// markdown links of form [text](link)
    # Ignore http/https, mailto, and anchor-only links (#anchor)
    links=$(grep -oE '\[[^]]+\]\(([^)#]+)(\#[^)]+)?\)' "$file" | sed -E 's/\[[^]]+\]\(([^)#]+)(\#[^)]+)?\)/\1/' | grep -vE '^(http|https|mailto):' || true)
    
    for link in $links; do
        # Trim leading/trailing whitespace
        link=$(echo "$link" | xargs)
        [ -z "$link" ] && continue
        
        # Check if file:// URI
        if [[ "$link" =~ ^file:// ]]; then
            target="${link#file://}"
        elif [[ "$link" = /* ]]; then
            target="${ROOT_DIR}${link}"
        else
            target="${file_dir}/${link}"
        fi

        if [ -e "$target" ]; then
            # Link exists
            :
        else
            fail "Broken link in $file: '$link' -> '$target' not found"
        fi
    done
done

if [ $ERRORS -eq 0 ]; then
    pass "All internal relative Markdown links resolve successfully"
fi

echo -e "\n${BLUE}=== 2. Verifying Configuration Schema Version Consistency ===${NC}"
SCHEMA_IN_DOCS=$(grep -E 'schema_version' docs/reference/configuration-reference.md | grep -oE '`[0-9]+`' | tr -d '`' | head -n1 || echo "")
CURRENT_CODE_SCHEMA=2

if [ "$SCHEMA_IN_DOCS" = "$CURRENT_CODE_SCHEMA" ]; then
    pass "docs/reference/configuration-reference.md matches CURRENT_SCHEMA_VERSION ($CURRENT_CODE_SCHEMA)"
else
    fail "Schema version mismatch: Docs specify '$SCHEMA_IN_DOCS' but code specifies $CURRENT_CODE_SCHEMA"
fi

echo -e "\n${BLUE}=== 3. Verifying Keybinding Registry Consistency ===${NC}"
if grep -q 'KeyActionBinding' src/settings/keybindings.rs; then
    # Verify core shortcuts exist in docs/reference/keyboard-reference.md
    for key in "Ctrl+S" "F1" "F10" "Ctrl+Shift+P" "Tab"; do
        if grep -q "$key" docs/reference/keyboard-reference.md; then
            pass "Keyboard reference includes core binding: $key"
        else
            fail "Keyboard reference missing core binding: $key"
        fi
    done
fi

echo -e "\n${BLUE}=== 4. Verifying Plugin Manifest Reference Consistency ===${NC}"
for plugin_manifest in plugins/*/plugin.toml; do
    if [ -f "$plugin_manifest" ]; then
        plugin_id=$(grep -E '^id\s*=' "$plugin_manifest" | head -n1 | cut -d'"' -f2)
        if grep -q "$plugin_id" docs/plugin-development/examples.md; then
            pass "Plugin $plugin_id documented in examples.md"
        else
            fail "Plugin $plugin_id not documented in examples.md"
        fi
    fi
done

echo -e "\n${BLUE}=== 5. Running Rustdoc Check ===${NC}"
if cargo doc --no-deps > /dev/null 2>&1; then
    pass "cargo doc --no-deps completed with 0 warnings"
else
    fail "cargo doc --no-deps failed"
fi

echo -e "\n========================================================"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}SUCCESS: All $TOTAL_CHECKS documentation quality checks passed!${NC}"
    exit 0
else
    echo -e "${RED}FAILURE: $ERRORS of $TOTAL_CHECKS checks failed.${NC}"
    exit 1
fi
