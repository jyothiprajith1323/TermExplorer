#!/usr/bin/env bash
# TermExplorer Packaging & Installation Acceptance Test Suite
# Validates portable archive, Debian package, installer/uninstaller, and runtime behavior in isolated environments.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${REPO_ROOT}/dist"
TEST_TMP_BASE="/tmp/te-pack-test-$$"

PASSED=0
FAILED=0

pass() {
    echo -e "  \033[0;32m[PASS]\033[0m $1"
    PASSED=$((PASSED + 1))
}

fail() {
    echo -e "  \033[0;31m[FAIL]\033[0m $1"
    FAILED=$((FAILED + 1))
}

cleanup() {
    rm -rf "${TEST_TMP_BASE}"
}
trap cleanup EXIT

mkdir -p "${TEST_TMP_BASE}"

echo "=========================================================="
echo " TermExplorer Sprint 6A Packaging Test Suite"
echo "=========================================================="
echo "Repository:    ${REPO_ROOT}"
echo "Dist Dir:      ${DIST_DIR}"
echo "Test Sandbox:  ${TEST_TMP_BASE}"
echo "Start Time:    $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="

# -----------------------------------------------------------------------------
echo ""
echo "=== Phase 1: Artifact Existence & Checksum Validation ==="
# -----------------------------------------------------------------------------

VERSION="$(grep -m1 '^version = ' "${REPO_ROOT}/Cargo.toml" | cut -d '"' -f 2)"
TARBALL="${DIST_DIR}/termexplorer-v${VERSION}-linux-x86_64.tar.gz"
DEB_FILE="${DIST_DIR}/termexplorer_${VERSION}_amd64.deb"
CHECKSUMS="${DIST_DIR}/SHA256SUMS"
MANIFEST="${DIST_DIR}/MANIFEST.txt"

if [ -f "${TARBALL}" ]; then
    pass "Portable archive exists: $(basename "${TARBALL}")"
else
    fail "Portable archive missing: ${TARBALL}"
fi

if [ -f "${DEB_FILE}" ]; then
    pass "Debian package exists: $(basename "${DEB_FILE}")"
else
    fail "Debian package missing: ${DEB_FILE}"
fi

if [ -f "${CHECKSUMS}" ]; then
    pass "SHA256SUMS file exists"
    (
        cd "${DIST_DIR}"
        if sha256sum -c SHA256SUMS >/dev/null 2>&1; then
            pass "All artifacts in SHA256SUMS match cryptographic hashes"
        else
            fail "SHA256SUMS verification failed"
        fi
    )
else
    fail "SHA256SUMS missing"
fi

if [ -f "${MANIFEST}" ]; then
    pass "Release artifact manifest exists: MANIFEST.txt"
    if grep -q "termexplorer" "${MANIFEST}"; then
        pass "MANIFEST.txt contains valid artifact rows"
    else
        fail "MANIFEST.txt does not contain expected rows"
    fi
else
    fail "MANIFEST.txt missing"
fi

# -----------------------------------------------------------------------------
echo ""
echo "=== Phase 2: Portable Archive Structure & Safety ==="
# -----------------------------------------------------------------------------

EXTRACT_DIR="${TEST_TMP_BASE}/extract_tarball"
mkdir -p "${EXTRACT_DIR}"

tar -xzf "${TARBALL}" -C "${EXTRACT_DIR}"
ROOT_FOLDER="${EXTRACT_DIR}/termexplorer-v${VERSION}-linux-x86_64"

if [ -d "${ROOT_FOLDER}" ]; then
    pass "Archive extracts into root folder: termexplorer-v${VERSION}-linux-x86_64"
else
    fail "Archive does not extract into expected root folder"
fi

# Check required files
for relpath in \
    "bin/termexplorer" \
    "share/man/man1/termexplorer.1" \
    "share/applications/termexplorer.desktop" \
    "share/icons/hicolor/scalable/apps/termexplorer.svg" \
    "share/bash-completion/completions/termexplorer" \
    "share/zsh/site-functions/_termexplorer" \
    "share/fish/vendor_completions.d/termexplorer.fish" \
    "share/doc/termexplorer/README.md" \
    "share/doc/termexplorer/LICENSE" \
    "install.sh" \
    "uninstall.sh"; do
    if [ -f "${ROOT_FOLDER}/${relpath}" ]; then
        pass "Archive contains ${relpath}"
    else
        fail "Archive missing ${relpath}"
    fi
done

# Check permissions
if [ -x "${ROOT_FOLDER}/bin/termexplorer" ]; then
    pass "Binary bin/termexplorer has executable permissions (0755)"
else
    fail "Binary bin/termexplorer is not executable"
fi

if [ -x "${ROOT_FOLDER}/install.sh" ] && [ -x "${ROOT_FOLDER}/uninstall.sh" ]; then
    pass "Installer scripts have executable permissions"
else
    fail "Installer scripts lack executable permissions"
fi

# Verify no unwanted files (e.g. .git, .DS_Store, target, etc.)
UNWANTED=$(find "${ROOT_FOLDER}" -name ".git*" -o -name ".DS_Store" -o -name "*.bak" 2>/dev/null || true)
if [ -z "${UNWANTED}" ]; then
    pass "Archive is clean of repository and temporary artifacts"
else
    fail "Archive contains unwanted files: ${UNWANTED}"
fi

# -----------------------------------------------------------------------------
echo ""
echo "=== Phase 3: Binary Execution & CLI Behavior ==="
# -----------------------------------------------------------------------------

BIN="${ROOT_FOLDER}/bin/termexplorer"

# Version check
VERSION_OUT="$("${BIN}" --version 2>&1 || true)"
if [[ "${VERSION_OUT}" == *"termexplorer ${VERSION}"* ]]; then
    pass "termexplorer --version reports '${VERSION_OUT}'"
else
    fail "termexplorer --version output incorrect: '${VERSION_OUT}'"
fi

# Help check
HELP_OUT="$("${BIN}" --help 2>&1 || true)"
if [[ "${HELP_OUT}" == *"USAGE:"* ]] && [[ "${HELP_OUT}" == *"--health"* ]]; then
    pass "termexplorer --help outputs expected usage documentation"
else
    fail "termexplorer --help output missing expected sections"
fi

# Health check
HEALTH_OUT="$("${BIN}" --health 2>&1 || true)"
if [[ "${HEALTH_OUT}" == *"System Diagnostics & Health Check"* ]] && [[ "${HEALTH_OUT}" == *"Health Status:"* ]]; then
    pass "termexplorer --health runs successfully and reports system status"
else
    fail "termexplorer --health output invalid"
fi

# Invalid option check (exit code 2)
set +e
"${BIN}" --invalid-option >/dev/null 2>&1
INVALID_EXIT_CODE=$?
set -e
if [ "${INVALID_EXIT_CODE}" -eq 2 ]; then
    pass "Invalid option exits with code 2 (helpful argument error)"
else
    fail "Invalid option exited with ${INVALID_EXIT_CODE} (expected 2)"
fi

# -----------------------------------------------------------------------------
echo ""
echo "=== Phase 4: Installer & Uninstaller Validation in Isolated Prefix ==="
# -----------------------------------------------------------------------------

# Test 1: Standard prefix
PREFIX_STD="${TEST_TMP_BASE}/opt/termexplorer"
"${ROOT_FOLDER}/install.sh" --prefix "${PREFIX_STD}" >/dev/null

if [ -x "${PREFIX_STD}/bin/termexplorer" ] && [ -f "${PREFIX_STD}/share/termexplorer/install-manifest.txt" ]; then
    pass "Installer successfully installed into standard prefix: ${PREFIX_STD}"
else
    fail "Installation failed in standard prefix"
fi

# Validate installed binary runs
INSTALLED_VER="$("${PREFIX_STD}/bin/termexplorer" -V)"
if [[ "${INSTALLED_VER}" == *"termexplorer ${VERSION}"* ]]; then
    pass "Installed binary executes cleanly: ${INSTALLED_VER}"
else
    fail "Installed binary execution failed"
fi

# Uninstall from standard prefix
"${PREFIX_STD}/share/termexplorer/uninstall.sh" >/dev/null
if [ ! -f "${PREFIX_STD}/bin/termexplorer" ] && [ ! -f "${PREFIX_STD}/share/termexplorer/install-manifest.txt" ]; then
    pass "Uninstaller successfully cleaned up files in standard prefix"
else
    fail "Uninstaller left files behind in standard prefix"
fi

# Test 2: Prefix containing spaces
PREFIX_SPACES="${TEST_TMP_BASE}/prefix with spaces"
"${ROOT_FOLDER}/install.sh" --prefix "${PREFIX_SPACES}" >/dev/null
if [ -x "${PREFIX_SPACES}/bin/termexplorer" ]; then
    pass "Installer handles path with spaces: '${PREFIX_SPACES}'"
else
    fail "Installer failed with path with spaces"
fi
"${PREFIX_SPACES}/share/termexplorer/uninstall.sh" >/dev/null
if [ ! -f "${PREFIX_SPACES}/bin/termexplorer" ]; then
    pass "Uninstaller successfully cleans up path with spaces"
else
    fail "Uninstaller failed to clean up path with spaces"
fi

# Test 3: Prefix containing Unicode characters
PREFIX_UNICODE="${TEST_TMP_BASE}/prefix_üñîçødé_ワークスペース"
"${ROOT_FOLDER}/install.sh" --prefix "${PREFIX_UNICODE}" >/dev/null
if [ -x "${PREFIX_UNICODE}/bin/termexplorer" ]; then
    pass "Installer handles Unicode paths: '${PREFIX_UNICODE}'"
else
    fail "Installer failed with Unicode path"
fi
"${PREFIX_UNICODE}/share/termexplorer/uninstall.sh" >/dev/null
if [ ! -f "${PREFIX_UNICODE}/bin/termexplorer" ]; then
    pass "Uninstaller successfully cleans up Unicode path"
else
    fail "Uninstaller failed to clean up Unicode path"
fi

# Test 4: User configuration preservation
FAKE_HOME="${TEST_TMP_BASE}/fake_user_home"
mkdir -p "${FAKE_HOME}/.config/termexplorer"
echo 'theme = "frappe"' > "${FAKE_HOME}/.config/termexplorer/config.toml"

PREFIX_USER_TEST="${TEST_TMP_BASE}/user_prefix"
HOME="${FAKE_HOME}" XDG_CONFIG_HOME="${FAKE_HOME}/.config" "${ROOT_FOLDER}/install.sh" --prefix "${PREFIX_USER_TEST}" >/dev/null
HOME="${FAKE_HOME}" XDG_CONFIG_HOME="${FAKE_HOME}/.config" "${PREFIX_USER_TEST}/share/termexplorer/uninstall.sh" >/dev/null

if [ -f "${FAKE_HOME}/.config/termexplorer/config.toml" ]; then
    CONFIG_CONTENT="$(cat "${FAKE_HOME}/.config/termexplorer/config.toml")"
    if [[ "${CONFIG_CONTENT}" == *'theme = "frappe"'* ]]; then
        pass "User configuration in ~/.config/termexplorer is strictly preserved across uninstallation"
    else
        fail "User configuration content was altered during uninstallation"
    fi
else
    fail "User configuration was unexpectedly deleted during uninstallation"
fi

# Test 5: Dry-run behavior
PREFIX_DRY="${TEST_TMP_BASE}/dryrun_prefix"
"${ROOT_FOLDER}/install.sh" --dry-run --prefix "${PREFIX_DRY}" >/dev/null
if [ ! -d "${PREFIX_DRY}/bin" ]; then
    pass "Dry-run flag (--dry-run) does not make filesystem modifications"
else
    fail "Dry-run created directories unexpectedly"
fi

# -----------------------------------------------------------------------------
echo ""
echo "=== Phase 5: Debian Package (.deb) Validation ==="
# -----------------------------------------------------------------------------

DEB_INFO="$(dpkg-deb --info "${DEB_FILE}")"

if echo "${DEB_INFO}" | grep -q "Package: termexplorer"; then
    pass "Debian package name is 'termexplorer'"
else
    fail "Debian package name is incorrect"
fi

if echo "${DEB_INFO}" | grep -q "Version: ${VERSION}"; then
    pass "Debian package version is '${VERSION}'"
else
    fail "Debian package version is incorrect"
fi

if echo "${DEB_INFO}" | grep -q "Architecture: amd64"; then
    pass "Debian package architecture is 'amd64'"
else
    fail "Debian package architecture is incorrect"
fi

if echo "${DEB_INFO}" | grep -q "Depends: libc6"; then
    pass "Debian package declares dependency on libc6"
else
    fail "Debian package missing libc6 dependency"
fi

DEB_CONTENTS="$(dpkg-deb --contents "${DEB_FILE}")"

for debpath in \
    "./usr/bin/termexplorer" \
    "./usr/share/man/man1/termexplorer.1.gz" \
    "./usr/share/applications/termexplorer.desktop" \
    "./usr/share/icons/hicolor/scalable/apps/termexplorer.svg" \
    "./usr/share/bash-completion/completions/termexplorer" \
    "./usr/share/zsh/vendor-completions/_termexplorer" \
    "./usr/share/fish/vendor_completions.d/termexplorer.fish" \
    "./usr/share/doc/termexplorer/copyright" \
    "./usr/share/doc/termexplorer/changelog.Debian.gz"; do
    if echo "${DEB_CONTENTS}" | grep -q "${debpath}"; then
        pass "Debian package includes ${debpath}"
    else
        fail "Debian package missing ${debpath}"
    fi
done

# Extract debian package to sandbox and run binary
DEB_EXTRACT="${TEST_TMP_BASE}/deb_extract"
mkdir -p "${DEB_EXTRACT}"
dpkg-deb -x "${DEB_FILE}" "${DEB_EXTRACT}"
DEB_BIN="${DEB_EXTRACT}/usr/bin/termexplorer"

if [ -x "${DEB_BIN}" ]; then
    pass "Debian package extracted binary is executable"
    DEB_BIN_VER="$("${DEB_BIN}" -V)"
    if [[ "${DEB_BIN_VER}" == *"termexplorer ${VERSION}"* ]]; then
        pass "Extracted Debian binary runs and reports version: ${DEB_BIN_VER}"
    else
        fail "Extracted Debian binary failed version check"
    fi
else
    fail "Extracted Debian binary not executable"
fi

# Verify md5sums inside .deb control
dpkg-deb -e "${DEB_FILE}" "${DEB_EXTRACT}/DEBIAN"
(
    cd "${DEB_EXTRACT}"
    if md5sum -c DEBIAN/md5sums >/dev/null 2>&1; then
        pass "Debian package DEBIAN/md5sums matches all installed files"
    else
        fail "Debian md5sums verification failed"
    fi
)

# -----------------------------------------------------------------------------
echo ""
echo "=== Phase 6: Dynamic Linkage & ABI Verification ==="
# -----------------------------------------------------------------------------

ABI_LIBS="$(readelf -d "${ROOT_FOLDER}/bin/termexplorer" | grep "NEEDED" | awk -F'[' '{print $2}' | tr -d ']' || true)"
echo "  Dynamic dependencies: $(echo ${ABI_LIBS} | tr '\n' ' ')"

# Verify only standard C/GCC libraries are dynamically needed
UNEXPECTED_LIBS=0
for lib in ${ABI_LIBS}; do
    case "$lib" in
        libc.so.6|libm.so.6|libgcc_s.so.1|ld-linux-x86-64.so.2)
            ;;
        *)
            fail "Unexpected dynamic library dependency: ${lib}"
            UNEXPECTED_LIBS=1
            ;;
    esac
done

if [ "${UNEXPECTED_LIBS}" -eq 0 ]; then
    pass "Binary strictly links only standard glibc runtime (libc.so.6, libm.so.6, libgcc_s.so.1)"
fi

# -----------------------------------------------------------------------------
echo ""
echo "=========================================================="
echo " Test Results Summary"
echo "=========================================================="
echo "Passed: ${PASSED}"
echo "Failed: ${FAILED}"
echo "End Time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="

if [ "${FAILED}" -eq 0 ]; then
    echo -e "\033[0;32mALL PACKAGING TESTS PASSED SUCCESSFULLY!\033[0m"
    exit 0
else
    echo -e "\033[0;31mSOME PACKAGING TESTS FAILED.\033[0m"
    exit 1
fi
