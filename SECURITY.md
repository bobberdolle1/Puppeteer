# Security Policy

## 🔒 Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |

## 🚨 Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

1. **DO NOT** create a public GitHub issue for security vulnerabilities
2. Email the maintainers directly or use GitHub's private vulnerability reporting
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution Timeline**: Depends on severity
  - Critical: 24-72 hours
  - High: 1-2 weeks
  - Medium: 2-4 weeks
  - Low: Next release

### Security Features

PersonaForge includes several built-in security measures:

#### 🛡️ Prompt Injection Protection
- 40+ attack pattern detection (EN/RU)
- Input sanitization
- Role marker escaping
- Strike system with temporary blocks

#### 🔐 Authentication
- Owner-only access via `OWNER_ID`
- Telegram WebApp HMAC-SHA256 validation
- No public API endpoints

#### 🚦 Rate Limiting
- Adaptive rate limiting based on violation history
- Per-user request throttling
- LLM queue management

#### 💾 Data Security
- Local SQLite database
- No external data transmission (except Telegram API)
- Ollama runs locally

## 🔧 Security Best Practices

When deploying PersonaForge:

1. **Keep secrets secure**
   - Never commit `.env` to version control
   - Use strong, unique `TELOXIDE_TOKEN`
   - Restrict `OWNER_ID` to trusted users

2. **Network security**
   - Run Ollama on localhost only
   - Use HTTPS for Mini App (required by Telegram)
   - Consider firewall rules for production

3. **Updates**
   - Keep Rust and dependencies updated
   - Monitor `cargo audit` for vulnerabilities
   - Subscribe to security advisories

4. **Monitoring**
   - Review logs for suspicious activity
   - Monitor `/security_status` command
   - Check blocked users periodically

## 📋 Security Checklist

- [ ] `.env` file is in `.gitignore`
- [ ] `OWNER_ID` is set correctly
- [ ] Ollama is not exposed to public network
- [ ] Mini App uses HTTPS
- [ ] Dependencies are up to date
- [ ] Security audit passes (`cargo audit`)
