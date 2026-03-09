# Security Policy

## Reporting a Vulnerability

The Mini Gateway team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

**Do not report security vulnerabilities through public GitHub issues.**

### How to Report

To report a security vulnerability, please use one of the following methods:

1. **GitHub Private Vulnerability Reporting** -- Use the [Security Advisories](https://github.com/zonblade/mini-gateway-rs/security/advisories/new) feature to privately report the vulnerability.
2. **Email** -- If private vulnerability reporting is not available, open a PR with the prefix `REPORT:` containing only the necessary details.

### What to Include

When reporting a vulnerability, please include:

- A description of the vulnerability
- Steps to reproduce the issue
- The potential impact
- Any suggested fixes (if available)
- The affected component(s) (router-core, router-api, router-cli, etc.)

### Response Timeline

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours.
- **Assessment**: We will provide an initial assessment within 7 days.
- **Resolution**: We aim to release a fix for confirmed vulnerabilities as soon as possible, depending on complexity.

### Disclosure Policy

- We follow coordinated disclosure practices.
- We will credit reporters in the security advisory (unless you prefer to remain anonymous).
- Please do not disclose the vulnerability publicly until we have released a fix.

## Supported Versions

Security updates are provided for the latest release only. We recommend always running the most recent version.

## Scope

The following components are in scope for security reports:

| Component | In Scope |
|-----------|----------|
| router-core | Yes |
| router-api | Yes |
| router-cli | Yes |
| router-rds | Yes |
| Documentation / website | No |
| Third-party dependencies | Report upstream, but notify us if it affects Mini Gateway |

## Security Best Practices

When deploying Mini Gateway:

- Run `router-api` on internal networks only -- it is not designed for public exposure.
- Use TLS termination for all external traffic.
- Keep your installation up to date.
- Follow the principle of least privilege for configuration access.
