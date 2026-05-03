# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of Arcane Codex seriously. If you have discovered a security vulnerability, we appreciate your help in disclosing it to us in a responsible manner.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via:

1. **GitHub Security Advisory** (Preferred): Use the [Security Advisories](https://github.com/hyls9527/ArcaneCodex/security/advisories) feature to report a vulnerability.

2. **Email**: Send details to the maintainer directly.

### What to Include

Please include the following information in your report:

- Type of vulnerability (e.g., buffer overflow, SQL injection, cross-site scripting)
- Full paths of source file(s) related to the vulnerability
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the vulnerability

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Target**: Critical vulnerabilities within 30 days

### Disclosure Policy

- We will acknowledge your email within 48 hours
- We will send a more detailed response within 7 days indicating the next steps
- We will keep you informed of the progress towards a fix
- We may ask for additional information or guidance

## Security Best Practices

When using Arcane Codex:

1. **API Keys**: Store your AI provider API keys securely. Never commit them to version control.
2. **Local Data**: All your photos and metadata are stored locally. Ensure your system is secure.
3. **Updates**: Keep the application updated to the latest version for security patches.
4. **Backups**: Regularly backup your database using the built-in backup feature.

## Known Security Considerations

- **Local Storage**: All data is stored locally on your machine. Physical access to your computer means access to your data.
- **AI Processing**: When using cloud AI providers (OpenAI, etc.), image descriptions are sent to their servers. For maximum privacy, use local AI (LM Studio, Ollama).
- **No Telemetry**: This application does not collect or send any usage data.

Thank you for helping keep Arcane Codex and its users safe!
