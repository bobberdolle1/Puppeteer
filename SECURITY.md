# Security Policy

## Supported Versions

Currently, only the latest version of Puppeteer receives security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of Puppeteer seriously. If you discover a security vulnerability, please follow these steps:

### How to Report

1. **DO NOT** open a public GitHub issue for security vulnerabilities
2. Email the maintainers directly at [INSERT EMAIL ADDRESS]
3. Include the following information:
   - Description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Investigation**: We will investigate the issue and determine its severity
- **Updates**: We will keep you informed of our progress
- **Resolution**: We will work on a fix and release a security patch
- **Credit**: We will credit you in the security advisory (unless you prefer to remain anonymous)

### Response Timeline

- **Critical vulnerabilities**: Patch within 7 days
- **High severity**: Patch within 14 days
- **Medium/Low severity**: Patch within 30 days

## Security Best Practices

When using Puppeteer, follow these security guidelines:

### Configuration

1. **Environment Variables**: Never commit `.env` files to version control
2. **API Keys**: Store all API keys and tokens securely
3. **Owner IDs**: Restrict `OWNER_IDS` to trusted administrators only
4. **Database**: Ensure database files have appropriate file permissions (600)

### Deployment

1. **Docker**: Use the provided Dockerfile for secure containerization
2. **Network**: Run behind a firewall and restrict network access
3. **Updates**: Keep Rust dependencies up to date with `cargo update`
4. **Monitoring**: Monitor logs for suspicious activity

### Telegram Security

1. **2FA**: Enable two-factor authentication on all Telegram accounts
2. **Session Files**: Protect TDLib session files (stored in `data/tdlib/`)
3. **Rate Limits**: Respect Telegram's rate limits to avoid account restrictions
4. **ToS Compliance**: Ensure usage complies with Telegram's Terms of Service

### Code Security

1. **Input Validation**: All user inputs are validated before processing
2. **SQL Injection**: We use SQLx with parameterized queries
3. **Prompt Injection**: Built-in detection system for malicious prompts
4. **Rate Limiting**: Implemented to prevent abuse

## Known Security Considerations

### Telegram MTProto

- Using MTProto userbots may violate Telegram's Terms of Service
- Accounts may be restricted or banned for automated behavior
- Use at your own risk and responsibility

### AI/LLM Integration

- LLM responses are not guaranteed to be safe or appropriate
- System prompts can be bypassed with sophisticated prompt injection
- Monitor generated content for compliance with your use case

### Database

- SQLite database files contain sensitive session data
- Ensure proper file permissions and backup encryption
- Do not expose database files publicly

## Security Features

Puppeteer includes several built-in security features:

1. **Owner-Only Access**: Admin commands restricted to `OWNER_IDS`
2. **Prompt Injection Detection**: Monitors for malicious prompt patterns
3. **Strike System**: Tracks and blocks abusive users
4. **Rate Limiting**: Prevents API abuse and spam
5. **Session Isolation**: Each userbot has isolated TDLib sessions
6. **Secure Defaults**: Conservative default configuration

## Vulnerability Disclosure Policy

We follow responsible disclosure practices:

1. Security researchers are given reasonable time to report vulnerabilities
2. We will not take legal action against researchers who follow this policy
3. We will publicly acknowledge researchers who report valid vulnerabilities
4. We will coordinate disclosure timing with the reporter

## Security Updates

Security updates will be:

- Released as patch versions (e.g., 0.1.1)
- Documented in `CHANGELOG.md`
- Announced in GitHub releases
- Tagged with `security` label

## Contact

For security concerns, contact:
- Email: [INSERT EMAIL ADDRESS]
- GitHub: Open a security advisory (not a public issue)

---

**Last Updated**: February 28, 2024
