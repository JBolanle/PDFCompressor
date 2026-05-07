# Security Policy

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Email **hello@onaviadigital.com** with:
- A description of the issue and its potential impact
- Steps to reproduce or a proof-of-concept
- Any suggested mitigations if you have them

I aim to acknowledge reports within 48 hours and will keep you updated as the issue is investigated and resolved.

## Scope

This app runs locally and does not transmit data over the network. The main attack surfaces are:

- The bundled Ghostscript binary (supply-chain integrity)
- Tauri IPC commands exposed to the frontend webview
- File path handling and output path resolution

## Supported Versions

Only the latest release is actively maintained.
