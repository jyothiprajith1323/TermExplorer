# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

---

## Reporting a Vulnerability

We take the security of **TermExplorer** seriously, especially regarding:
- Filesystem safety (path traversal prevention in archive decompression).
- Child process execution and shell escapes in the integrated terminal emulator.
- Safe parsing of untrusted binary and multimedia preview formats.

If you believe you have discovered a security vulnerability in TermExplorer, **please do not open a public issue**.

Instead, please send an email to the maintainer:
- **Email:** `jyothiprajith1323@gmail.com`
- **Subject:** `[SECURITY] Potential vulnerability in TermExplorer`

Please include:
1. A description of the vulnerability and attack vector.
2. Steps to reproduce or a Proof of Concept (PoC).
3. The potential impact of the issue.

We will acknowledge receipt within 48 hours and work with you to remediate the vulnerability prior to public disclosure.
