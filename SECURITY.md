# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |

## Reporting a Vulnerability

**Do not report security vulnerabilities via GitHub Issues.**

Please use one of the following:

- **GitHub Private Vulnerability Reporting**: [Security tab](https://github.com/tsucky230/comP/security/advisories/new) → "Report a vulnerability"
- **Email**: tsuki80ke@gmail.com (subject: `[comP] Security`)

### What to include

- Description of the vulnerability and its potential impact
- Steps to reproduce
- Affected version(s)
- Any suggested fix (optional)

### Response timeline

| Stage | Target |
|---|---|
| Acknowledgement | Within 72 hours |
| Initial assessment | Within 7 days |
| Fix or mitigation | Within 90 days |

We follow coordinated disclosure: please allow us to release a fix before
publishing details publicly.

## Scope

In scope:
- comP VSCode extension (`src/`)
- Rust daemon (`daemon/`)
- MCP protocol handling

Out of scope:
- VSCode itself
- Dependencies (report to upstream maintainers)
- Issues requiring physical access to the machine
