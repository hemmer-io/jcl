# Security Policy

## Supported Versions

We currently support the following versions of JCL with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of JCL seriously. If you discover a security vulnerability, please follow these steps:

### 1. Do Not Publicly Disclose

Please **do not** create a public GitHub issue for security vulnerabilities. This helps protect users who haven't yet updated to a patched version.

### 2. Report Privately

Send a detailed report to: **security@hemmer.io**

Include in your report:
- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if you have one)
- Your contact information for follow-up questions

### 3. What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Initial Assessment**: We will provide an initial assessment within 5 business days
- **Updates**: We will keep you informed of our progress
- **Resolution**: We aim to release a fix within 30 days for critical vulnerabilities

### 4. Coordinated Disclosure

We believe in responsible disclosure and will:
- Work with you to understand and validate the issue
- Develop and test a fix
- Prepare a security advisory
- Release a patched version
- Publicly disclose the vulnerability after a fix is available

We request that you:
- Give us reasonable time to fix the issue before public disclosure
- Do not exploit the vulnerability beyond what's necessary to demonstrate it

## Security Best Practices

When using JCL:

### For Application Developers

1. **Input Validation**: Always validate and sanitize external input before evaluating it with JCL
2. **Sandboxing**: Consider running JCL evaluation in a sandboxed environment for untrusted input
3. **File Access**: Be cautious with file-related functions (`file()`, `templatefile()`) when processing untrusted configurations
4. **Resource Limits**: Implement timeouts and resource limits to prevent denial-of-service

### For Library Users

1. **Keep Updated**: Always use the latest version of JCL to get security fixes
2. **Dependency Auditing**: Regularly run `cargo audit` to check for vulnerabilities in dependencies
3. **Code Review**: Review JCL configurations before deployment
4. **Principle of Least Privilege**: Run JCL with minimal necessary permissions

## Known Security Considerations

### Safe by Design

JCL is designed with security in mind:

- **No Arbitrary Code Execution**: JCL cannot execute arbitrary system commands
- **Sandboxed Evaluation**: The evaluator is sandboxed and cannot access system resources except through explicit functions
- **No Network Access**: JCL has no built-in network capabilities
- **Immutability**: Variables are immutable by default, reducing state-related bugs

### Functions with Side Effects

Be aware of these functions when processing untrusted input:

- `file(path)` - Reads files from the filesystem
- `templatefile(path, vars)` - Reads template files
- `fileexists(path)` - Checks file existence

These functions are restricted to the filesystem permissions of the process running JCL.

### Resource Exhaustion

While JCL includes protections against infinite loops and stack overflow, be cautious with:

- Deeply nested data structures
- Very large lists or maps
- Complex recursive function definitions
- Extremely long strings

Consider implementing:
- Maximum evaluation depth limits
- Memory usage caps
- Execution timeouts

## Security Updates

Security updates will be:
- Released as patch versions (e.g., 1.0.1)
- Documented in the CHANGELOG with a `[SECURITY]` tag
- Announced via GitHub Security Advisories
- Published to relevant security databases

## Vulnerability Disclosure Process

1. **Report Received**: Security team acknowledges the report
2. **Verification**: Team verifies and assesses the severity
3. **Fix Development**: Team develops and tests a fix
4. **Security Advisory**: Draft advisory is prepared (but not published)
5. **Release**: Patched version is released
6. **Public Disclosure**: Advisory is published, CVE is filed if applicable
7. **Credit**: Reporter is credited (unless they prefer to remain anonymous)

## Severity Ratings

We use the following severity ratings:

- **Critical**: Exploitable remotely with no user interaction, leads to complete system compromise
- **High**: Exploitable with minimal user interaction, significant impact
- **Medium**: Exploitable with substantial user interaction, moderate impact
- **Low**: Difficult to exploit, minimal impact

## Bug Bounty

We currently do not have a bug bounty program, but we greatly appreciate security researchers who responsibly disclose vulnerabilities. We will:

- Publicly credit researchers (with their permission)
- Include them in our security acknowledgments
- Consider their contributions when evaluating future bug bounty programs

## Contact

For security concerns, contact: **security@hemmer.io**

For general issues, use GitHub Issues.

## Acknowledgments

We thank the following researchers for responsibly disclosing security vulnerabilities:

_None yet - be the first!_

---

Last updated: 2025-11-18
