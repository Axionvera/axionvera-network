# Security Policy

## Supported Versions

The following versions of Axionvera Network are currently being supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

**Security Patch Policy:**
- Only the latest minor version (0.1.x) receives security patches
- Security updates are released as patch versions (e.g., 0.1.1 → 0.1.2)
- Major vulnerabilities may result in immediate hotfix releases
- All users are strongly encouraged to upgrade to the latest version promptly

## Reporting a Vulnerability

We take the security of Axionvera Network seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### **IMPORTANT: Do Not Open Public Issues**

**Please do not open public GitHub issues for security vulnerabilities.** This prevents potential exploitation before a fix is available.

### Secure Contact Method

**Email:** security@axionvera.network (preferred)

**PGP Key:** Available at [keybase.io/axionvera](https://keybase.io/axionvera) or upon request

**Response Timeline:**
- We will acknowledge receipt of your report within **48 hours**
- We will send a more detailed response within **7 days** indicating the next steps
- We will keep you informed of our progress throughout the process

### What to Include

To help us triage and respond quickly, please include:

1. **Description**: Clear description of the vulnerability
2. **Impact**: Potential impact if exploited
3. **Reproduction**: Step-by-step instructions to reproduce the issue
4. **Affected Components**: Which parts of the system are affected
5. **Suggested Fix**: If you have suggestions for addressing the issue

### Responsible Disclosure Process

We follow a responsible disclosure process:

1. **Initial Report**: You submit a detailed report via secure email
2. **Acknowledgment**: We confirm receipt within 48 hours
3. **Assessment**: Our security team evaluates the report (5-10 days)
4. **Fix Development**: We develop and test a patch
5. **Release**: Security fix is deployed and announced
6. **Disclosure**: After 30 days from release, we encourage public disclosure

We kindly ask that you:
- Allow us a reasonable time to fix the issue before public disclosure
- Make an effort to work with us to understand and resolve the issue
- Provide us with sufficient time to prepare a patch before disclosure

## Security Best Practices

When deploying Axionvera Network, please follow these security best practices:

### Infrastructure Security
- Use TLS/SSL for all network communications
- Deploy behind a firewall with restricted access
- Use VPN or SSH tunnels for administrative access
- Regularly update underlying OS and dependencies
- Monitor logs for suspicious activity

### Application Security
- Never share private keys or credentials
- Use environment variables for sensitive configuration
- Enable audit logging in production
- Regularly rotate credentials and API keys
- Implement rate limiting and DDoS protection

### Smart Contract Security
- Review contract code before deployment
- Test thoroughly on testnet before mainnet
- Consider third-party security audits
- Monitor contract events and transactions
- Have incident response procedures in place

## Threat Model

This section outlines what is and isn't considered a security vulnerability in the context of Axionvera Network.

### ✅ **Considered Vulnerabilities**

The following are considered security vulnerabilities and should be reported:

#### Remote Code Execution (RCE)
- Arbitrary code execution on network nodes
- Unauthorized command execution through API endpoints
- Code injection vulnerabilities

#### Authentication & Authorization Bypass
- Bypassing authentication mechanisms
- Privilege escalation attacks
- Unauthorized access to admin endpoints

#### Data Integrity Violations
- Unauthorized modification of vault balances
- Manipulation of reward calculations
- Tampering with transaction history

#### Denial of Service (DoS)
- Resource exhaustion attacks (memory, CPU, disk)
- Amplification attacks
- Attacks that crash network nodes

#### Information Disclosure
- Exposure of private keys or secrets
- Leakage of user data
- Unauthorized access to sensitive configuration

#### Smart Contract Exploits
- Reentrancy attacks
- Integer overflow/underflow
- Logic errors allowing fund theft
- Flash loan attack vectors

### ❌ **NOT Considered Vulnerabilities**

The following are generally **not** considered security vulnerabilities:

#### Expected Protocol Behavior
- Economic exploits that work as designed (e.g., MEV, arbitrage)
- Game theory exploits within protocol rules
- Front-running on public mempools

#### Non-Critical Issues
- Missing best practices without direct exploit path
- Theoretical attacks with no practical demonstration
- Issues requiring unrealistic assumptions (e.g., private key compromise)

#### External Dependencies
- Vulnerabilities in underlying blockchain (Stellar/Soroban)
- Issues in third-party libraries (report upstream)
- Browser-specific security warnings

#### Social Engineering
- Phishing attacks (report to appropriate platforms)
- Impersonation attacks
- Community management issues

#### Configuration Issues
- Insecure default configurations (unless exploitable remotely)
- User misconfiguration of their own deployments
- Lack of security features (vs. broken security features)

### 🤔 **Gray Areas**

Some issues require careful consideration:

#### Rate Limiting
- Missing rate limits: Generally not a vulnerability unless combined with other issues
- Rate limit bypass: May be a vulnerability if it enables DoS or brute force

#### Information Leakage
- Metadata leakage: Depends on sensitivity and exploitability
- Timing attacks: Considered vulnerabilities if practically exploitable

#### Consensus Issues
- Network partitioning: Considered vulnerabilities
- Transaction ordering: Generally not vulnerabilities unless manipulated

## Security Updates

Security updates will be communicated through:

- **GitHub Security Advisories**: For critical vulnerabilities
- **Release Notes**: General security fixes in new versions
- **Direct Communication**: For severe issues affecting specific deployments
- **Blog Posts**: Educational content about security improvements

## Acknowledgments

We would like to thank the following for their contributions to our security:

- Security researchers who responsibly disclose vulnerabilities
- The Stellar Security Team for collaboration on Soroban-related issues
- The open-source community for ongoing security reviews

## Contact

For any questions about this security policy, please contact:
- **Email**: security@axionvera.network
- **Twitter**: @AxionveraNet (for general inquiries only, NOT for reporting vulnerabilities)

---

*This security policy is subject to change without notice. Last updated: March 26, 2026.*
