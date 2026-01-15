# Security Policy

## Supported Versions

We actively support the following versions of CrabCamera with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.6.x   | :white_check_mark: |
| 0.5.x   | :white_check_mark: |
| < 0.5   | :x:                |

## Reporting a Vulnerability

We take the security of CrabCamera seriously. If you discover a security vulnerability, please follow these guidelines:

### :lock: Private Disclosure Process

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please report security issues privately using one of these methods:

1. **GitHub Security Advisories (Preferred)**
   - Go to the [Security tab](https://github.com/Michael-A-Kuykendall/crabcamera/security) of this repository
   - Click "Report a vulnerability"
   - Fill out the advisory form with details

2. **Direct Email**
   - Send details to: michaelallenkuykendall@gmail.com
   - Include "SECURITY: CrabCamera" in the subject line

### :memo: What to Include

Please provide the following information in your report:

- **Description**: Clear description of the vulnerability
- **Impact**: What could an attacker accomplish?
- **Reproduction**: Step-by-step instructions to reproduce the issue
- **Environment**:
  - CrabCamera version
  - Operating system (Windows/macOS/Linux)
  - Rust version
  - Camera hardware (if applicable)
- **Proof of Concept**: Code or logs demonstrating the issue
- **Suggested Fix**: If you have ideas for remediation

### :stopwatch: Response Timeline

We aim to respond to security reports according to the following timeline:

- **Initial Response**: Within 48 hours of report
- **Triage**: Within 7 days - confirm/deny vulnerability
- **Resolution**: Within 30 days for critical issues, 90 days for others
- **Disclosure**: Public disclosure after fix is released and users have time to update

### :warning: Vulnerability Severity Guidelines

We use the following criteria to classify vulnerabilities:

#### Critical
- Remote code execution via camera input
- Memory corruption leading to arbitrary code execution
- Unauthorized camera/audio access

#### High
- Denial of service via crafted input
- Memory exhaustion attacks
- Privacy leaks (unauthorized recording)

#### Medium
- Information disclosure
- Panic in safe Rust code
- Resource leaks

#### Low
- Issues requiring local access
- Minor information leaks
- Performance degradation attacks

### :trophy: Recognition

We believe in recognizing security researchers who help keep CrabCamera secure:

- **Hall of Fame**: Public recognition in our security acknowledgments
- **CVE Assignment**: For qualifying vulnerabilities
- **Acknowledgment**: Credit in release notes

*Note: We currently do not offer monetary bug bounties, but we deeply appreciate responsible disclosure.*

### :rotating_light: Emergency Contact

For critical vulnerabilities that are being actively exploited:

- **Email**: michaelallenkuykendall@gmail.com
- **Subject**: "URGENT SECURITY: CrabCamera - [Brief Description]"
- **Response**: Within 12 hours

## Security Best Practices

### For Users

1. **Keep Updated**: Always use the latest supported version
2. **Permission Management**: Only grant camera/audio permissions when needed
3. **Trusted Sources**: Only use CrabCamera from official releases

### For Developers

1. **Dependencies**: Regularly audit and update dependencies
2. **Input Validation**: Validate camera input and configuration
3. **Memory Safety**: CrabCamera is built with Rust for memory-safe execution

## Security Features

CrabCamera includes several built-in security features:

- **Memory Safety**: Built with Rust for memory-safe execution
- **Permission System**: Explicit camera/audio permission requests
- **No Unsafe Code**: Pure safe Rust in production paths
- **Platform Integration**: Uses native platform security (MediaFoundation, AVFoundation, V4L2)

## Contact

For non-security related issues, please use:
- GitHub Issues: https://github.com/Michael-A-Kuykendall/crabcamera/issues
- GitHub Discussions: https://github.com/Michael-A-Kuykendall/crabcamera/discussions

---

*This security policy is effective as of January 2026 and may be updated periodically.*
