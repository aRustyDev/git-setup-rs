# Phase 8: Advanced Features - Work Plan

## Prerequisites

Before starting Phase 8, ensure you have a solid foundation from the previous sub-phases.

**Required from Previous Sub-Phases**:
- ✅ Pattern matching system (Phase 5A)
- ✅ Health monitoring framework (Phase 5B)
- ✅ Basic signing methods working (Phase 7)
- ✅ All previous checkpoints approved

**Required Knowledge**:
- **Advanced Signing**: x509, Sigstore concepts (*helpful*)
- **Network Security**: HTTPS, certificate validation (*required*)
- **Config Formats**: TOML/YAML parsing (*required*)
- **Security**: Input validation, sanitization (*critical*)

💡 **Junior Dev Resources**:
- 📚 [Sigstore Docs](https://docs.sigstore.dev/) - Keyless signing intro
- 📖 [x509 Certificates Explained](https://smallstep.com/blog/everything-pki/) - Comprehensive guide
- 📖 [Secure Remote Fetching](https://owasp.org/www-community/attacks/) - OWASP guide
- 🔧 Practice: Try examples in `examples/advanced/`
- 📝 [Security Best Practices](../../resources/security-best-practices.md)
- 🔐 [Security Implementation Examples](../SECURITY_IMPLEMENTATION_EXAMPLES.md) - Advanced security patterns
- ⚡ [Async/Await Examples](../ASYNC_AWAIT_EXAMPLES.md) - For network operations

## Quick Reference - Essential Resources

### Advanced Signing Methods
1. **x509**: Certificate-based signing (enterprise)
2. **Sigstore**: Keyless signing with OIDC
3. **Remote Import**: Fetching configs from URLs
4. **Validation**: Always validate external data

### Security Principles
- Never trust remote data
- Validate all inputs
- Use HTTPS only
- Implement timeouts
- Sanitize configurations

## Overview

Phase 8 completes the advanced features with modern signing methods and secure remote configuration import. This phase requires careful attention to security.

**What You'll Build**:
1. x509 certificate signing support
2. Sigstore integration for keyless signing
3. Secure remote configuration import
4. Comprehensive validation framework
5. Advanced security features

**Success Looks Like**:
- Enterprise users can use x509 certificates
- Developers can use keyless Sigstore signing
- Teams can share configurations via URLs
- All imports are secure and validated

**Time Estimate**: 2 weeks (80 hours)
- Week 1: Advanced signing methods (40h)
- Week 2: Remote import and security (40h)

## Week 1: Advanced Signing Methods

### 5D.1 x509 Certificate Signing (20 hours)

#### Task 5D.1.1: x509 Certificate Support (10 hours)

💡 **Junior Dev Concept**: x509 Certificate Signing
**What it is**: Using digital certificates (like HTTPS certs) to sign commits
**Why use it**: Required in many enterprise environments
**How it works**: Similar to GPG but uses standard x509 certificates

**Prerequisites**:
- [ ] Basic understanding of certificates
- [ ] Test certificate for development
- [ ] Read about S/MIME

**Visual x509 Signing Flow**:
```
Certificate Authority          Your Certificate           Git Commit
────────────────────          ────────────────          ───────────
Issues certificate ──────> Contains your identity ────> Signs commit
                           + public/private key          with certificate
                           
                           Trusted by organization      Verified by CA chain
```

**Step-by-Step Implementation**:

1. **Create x509 Module Structure** (2 hours)
   ```rust
   // src/signing/x509/mod.rs
   
   use std::path::PathBuf;
   use chrono::{DateTime, Utc};
   
   /// x509 certificate information
   #[derive(Debug, Clone)]
   pub struct X509CertInfo {
       /// Path to certificate file
       pub cert_path: PathBuf,
       
       /// Subject name (CN)
       pub subject: String,
       
       /// Issuer name
       pub issuer: String,
       
       /// Certificate serial number
       pub serial: String,
       
       /// Valid from date
       pub not_before: DateTime<Utc>,
       
       /// Valid until date
       pub not_after: DateTime<Utc>,
       
       /// Email addresses in certificate
       pub emails: Vec<String>,
       
       /// Whether private key is available
       pub has_private_key: bool,
   }
   
   /// Errors specific to x509 operations
   #[derive(Debug, thiserror::Error)]
   pub enum X509Error {
       #[error("Certificate not found: {0}")]
       NotFound(String),
       
       #[error("Certificate expired on {0}")]
       Expired(DateTime<Utc>),
       
       #[error("Certificate not yet valid until {0}")]
       NotYetValid(DateTime<Utc>),
       
       #[error("No email in certificate")]
       NoEmail,
       
       #[error("Private key not found")]
       NoPrivateKey,
   }
   ```

2. **Implement Certificate Discovery** (4 hours)
   ```rust
   // src/signing/x509/discovery.rs
   
   use openssl::x509::X509;
   use openssl::pkey::PKey;
   
   pub struct X509Discovery;
   
   impl X509Discovery {
       /// Find x509 certificates suitable for signing
       pub fn discover_certificates() -> Result<Vec<X509CertInfo>, SigningError> {
           let mut certs = Vec::new();
           
           // Check system certificate stores
           certs.extend(Self::check_system_store()?);
           
           // Check user certificate directory
           if let Some(user_certs) = Self::check_user_certificates()? {
               certs.extend(user_certs);
           }
           
           // Filter for signing-capable certificates
           let signing_certs: Vec<_> = certs
               .into_iter()
               .filter(|cert| Self::can_sign(cert))
               .collect();
           
           Ok(signing_certs)
       }
       
       /// Check if certificate can be used for signing
       fn can_sign(cert: &X509CertInfo) -> bool {
           // Must have private key
           if !cert.has_private_key {
               return false;
           }
           
           // Must be currently valid
           let now = Utc::now();
           if now < cert.not_before || now > cert.not_after {
               return false;
           }
           
           // Must have email for Git
           !cert.emails.is_empty()
       }
       
       /// Parse certificate file
       fn parse_certificate(path: &Path) -> Result<X509CertInfo, SigningError> {
           let cert_pem = fs::read_to_string(path)?;
           let cert = X509::from_pem(cert_pem.as_bytes())?;
           
           // Extract subject
           let subject = cert.subject_name()
               .entries()
               .find(|e| e.object().nid() == openssl::nid::Nid::COMMONNAME)
               .and_then(|e| e.data().as_utf8().ok())
               .map(|s| s.to_string())
               .unwrap_or_else(|| "Unknown".to_string());
           
           // Extract emails from SAN or subject
           let emails = Self::extract_emails(&cert);
           
           // Check for private key
           let key_path = path.with_extension("key");
           let has_private_key = key_path.exists();
           
           Ok(X509CertInfo {
               cert_path: path.to_path_buf(),
               subject,
               issuer: Self::get_issuer(&cert),
               serial: format!("{:X}", cert.serial_number()),
               not_before: Self::asn1_to_chrono(cert.not_before()),
               not_after: Self::asn1_to_chrono(cert.not_after()),
               emails,
               has_private_key,
           })
       }
   }
   ```

3. **Configure Git for x509** (4 hours)
   ```rust
   // src/signing/x509/config.rs
   
   pub struct X509SigningConfig;
   
   impl X509SigningConfig {
       /// Configure Git to use x509 signing
       pub fn configure(
           cert: &X509CertInfo,
           scope: ConfigScope,
       ) -> Result<(), SigningError> {
           let git_config = GitConfig::new(scope);
           
           // Set signing format to x509
           git_config.set("gpg.format", "x509")?;
           
           // Set certificate path
           git_config.set("user.signingkey", &cert.cert_path.to_string_lossy())?;
           
           // Set gpg program to handle x509 (Git uses gpgsm)
           git_config.set("gpg.x509.program", "gpgsm")?;
           
           // Enable S/MIME
           git_config.set("gpg.smimesign.program", "smimesign")?;
           
           Ok(())
       }
       
       /// Test x509 signing
       pub fn test_signing(cert: &X509CertInfo) -> Result<bool, SigningError> {
           // Create test repo
           let temp_dir = tempfile::tempdir()?;
           
           // Initialize Git repo
           Command::new("git")
               .current_dir(&temp_dir)
               .args(&["init"])
               .output()?;
           
           // Configure signing
           Self::configure(cert, ConfigScope::Local)?;
           
           // Create test commit
           let test_file = temp_dir.path().join("test.txt");
           fs::write(&test_file, "x509 test")?;
           
           Command::new("git")
               .current_dir(&temp_dir)
               .args(&["add", "test.txt"])
               .output()?;
           
           let output = Command::new("git")
               .current_dir(&temp_dir)
               .args(&["commit", "-S", "-m", "Test x509 signing"])
               .env("GIT_COMMITTER_EMAIL", &cert.emails[0])
               .output()?;
           
           Ok(output.status.success())
       }
   }
   ```

**Testing Your Implementation**:
```rust
#[test]
fn test_certificate_parsing() {
    // Create test certificate
    let cert_pem = include_str!("../../../tests/fixtures/test_cert.pem");
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), cert_pem).unwrap();
    
    let cert_info = X509Discovery::parse_certificate(temp_file.path()).unwrap();
    assert!(!cert_info.emails.is_empty());
    assert!(cert_info.subject.contains("Test User"));
}
```

---

### 5D.2 Sigstore Integration (20 hours)

#### Task 5D.2.1: Keyless Signing with Sigstore (10 hours)

💡 **Junior Dev Concept**: Sigstore Keyless Signing
**What it is**: Sign commits without managing keys - uses your email identity
**Why revolutionary**: No key management, uses OIDC (Google/GitHub/etc login)
**How it works**: Proves you control an email at signing time

**Visual Sigstore Flow**:
```
You                    Identity Provider         Sigstore              Git
───                    ─────────────────        ────────              ───
Login with ─────────> Verify identity ───> Issue certificate ───> Sign commit
Google/GitHub         (OIDC)                (short-lived)         (keyless!)
```

**Step-by-Step Implementation**:

1. **Create Sigstore Module** (3 hours)
   ```rust
   // src/signing/sigstore/mod.rs
   
   use sigstore::cosign::{CosignCapabilities, Signer};
   use sigstore::oauth::{OAuthFlow, IdentityToken};
   use sigstore::rekor::Client as RekorClient;
   
   /// Sigstore signing configuration
   #[derive(Debug, Clone)]
   pub struct SigstoreConfig {
       /// Identity provider (google, github, microsoft)
       pub identity_provider: String,
       
       /// Email to use for signing
       pub email: String,
       
       /// Fulcio URL (certificate authority)
       pub fulcio_url: String,
       
       /// Rekor URL (transparency log)
       pub rekor_url: String,
       
       /// OIDC issuer URL
       pub oidc_issuer: String,
   }
   
   impl Default for SigstoreConfig {
       fn default() -> Self {
           Self {
               identity_provider: "google".to_string(),
               email: String::new(),
               fulcio_url: "https://fulcio.sigstore.dev".to_string(),
               rekor_url: "https://rekor.sigstore.dev".to_string(),
               oidc_issuer: "https://oauth2.sigstore.dev/auth".to_string(),
           }
       }
   }
   
   /// Sigstore signing implementation
   pub struct SigstoreSigner {
       config: SigstoreConfig,
       signer: Option<Box<dyn Signer>>,
   }
   
   impl SigstoreSigner {
       pub fn new(config: SigstoreConfig) -> Self {
           Self {
               config,
               signer: None,
           }
       }
       
       /// Authenticate with identity provider
       pub async fn authenticate(&mut self) -> Result<(), SigningError> {
           println!("🔐 Authenticating with Sigstore...");
           
           // Create OAuth flow
           let oauth_flow = OAuthFlow::new(
               &self.config.oidc_issuer,
               &self.config.identity_provider,
           )?;
           
           // Get authorization URL
           let auth_url = oauth_flow.authorization_url();
           
           println!("Please visit this URL to authenticate:");
           println!("{}", auth_url);
           
           // Open browser automatically
           if let Err(e) = webbrowser::open(&auth_url) {
               eprintln!("Failed to open browser: {}", e);
           }
           
           // Wait for callback with token
           let token = oauth_flow.wait_for_token().await
               .map_err(|e| SigningError::Sigstore(format!("OAuth failed: {}", e)))?;
           
           // Create signer with token
           let signer = self.create_signer(token).await?;
           self.signer = Some(signer);
           
           println!("✅ Successfully authenticated!");
           Ok(())
       }
       
       /// Create signer from identity token
       async fn create_signer(&self, token: IdentityToken) -> Result<Box<dyn Signer>, SigningError> {
           // Get certificate from Fulcio
           let fulcio_client = FulcioClient::new(&self.config.fulcio_url)?;
           
           let cert_response = fulcio_client
               .get_certificate(&token)
               .await
               .map_err(|e| SigningError::Sigstore(format!("Fulcio error: {}", e)))?;
           
           // Create ephemeral key pair
           let key_pair = EphemeralKeyPair::generate()?;
           
           // Create signer
           let signer = CosignSigner::new(
               cert_response.certificate,
               cert_response.chain,
               key_pair,
           )?;
           
           Ok(Box::new(signer))
       }
   }
   ```

2. **Implement Git Integration** (4 hours)
   ```rust
   // src/signing/sigstore/git.rs
   
   /// Configure Git for Sigstore signing
   pub struct SigstoreGitConfig;
   
   impl SigstoreGitConfig {
       /// Set up Git to use Sigstore
       pub fn configure(email: &str) -> Result<(), SigningError> {
           let git_config = GitConfig::new(ConfigScope::Global);
           
           // Set signing format to x509 (Sigstore uses certificates)
           git_config.set("gpg.format", "x509")?;
           
           // Use gitsign as the signing program
           git_config.set("gpg.x509.program", "gitsign")?;
           
           // Set signing key to email (Sigstore uses email as identity)
           git_config.set("user.signingkey", email)?;
           
           // Enable commit signing
           git_config.set("commit.gpgsign", "true")?;
           
           // Configure gitsign
           git_config.set("gitsign.fulcio", "https://fulcio.sigstore.dev")?;
           git_config.set("gitsign.rekor", "https://rekor.sigstore.dev")?;
           git_config.set("gitsign.issuer", "https://oauth2.sigstore.dev/auth")?;
           
           Ok(())
       }
       
       /// Test Sigstore signing
       pub async fn test_signing() -> Result<bool, SigningError> {
           // Create test repository
           let temp_dir = tempfile::tempdir()?;
           
           // Initialize repo
           Command::new("git")
               .current_dir(&temp_dir)
               .args(&["init"])
               .output()?;
           
           // Create test file
           let test_file = temp_dir.path().join("sigstore_test.txt");
           fs::write(&test_file, "Sigstore signing test")?;
           
           // Add and commit with signing
           Command::new("git")
               .current_dir(&temp_dir)
               .args(&["add", "sigstore_test.txt"])
               .output()?;
           
           let output = Command::new("git")
               .current_dir(&temp_dir)
               .args(&["commit", "-S", "-m", "Test Sigstore signing"])
               .output()?;
           
           if output.status.success() {
               // Verify signature
               let verify = Command::new("git")
                   .current_dir(&temp_dir)
                   .args(&["log", "--show-signature", "-1"])
                   .output()?;
               
               let output_str = String::from_utf8_lossy(&verify.stdout);
               Ok(output_str.contains("Good signature") || 
                  output_str.contains("Sigstore"))
           } else {
               let error = String::from_utf8_lossy(&output.stderr);
               Err(SigningError::Sigstore(format!("Signing failed: {}", error)))
           }
       }
   }
   ```

3. **Create Sigstore Verification** (3 hours)
   ```rust
   /// Verify Sigstore signatures
   pub struct SigstoreVerifier {
       rekor_client: RekorClient,
   }
   
   impl SigstoreVerifier {
       pub fn new(rekor_url: &str) -> Result<Self, SigningError> {
           let rekor_client = RekorClient::new(rekor_url)?;
           Ok(Self { rekor_client })
       }
       
       /// Verify a Git commit signature
       pub async fn verify_commit(&self, commit_sha: &str) -> Result<VerificationResult, SigningError> {
           // Get signature from Git
           let output = Command::new("git")
               .args(&["show", "--format=%GG", commit_sha])
               .output()?;
           
           let signature = String::from_utf8_lossy(&output.stdout);
           
           // Parse certificate from signature
           let cert = self.parse_certificate(&signature)?;
           
           // Check transparency log
           let entry = self.rekor_client
               .search_by_hash(&cert.fingerprint())
               .await?;
           
           if let Some(log_entry) = entry {
               // Verify inclusion proof
               let verified = self.rekor_client
                   .verify_inclusion(&log_entry)
                   .await?;
               
               Ok(VerificationResult {
                   valid: verified,
                   signer_email: cert.subject_email()?,
                   timestamp: log_entry.integrated_time,
                   transparency_log: Some(log_entry.uuid),
               })
           } else {
               Ok(VerificationResult {
                   valid: false,
                   signer_email: cert.subject_email().unwrap_or_default(),
                   timestamp: None,
                   transparency_log: None,
               })
           }
       }
   }
   ```

**Testing Sigstore Integration**:
```rust
#[tokio::test]
async fn test_sigstore_flow() {
    // Skip if gitsign not installed
    if Command::new("gitsign").arg("--version").output().is_err() {
        eprintln!("Skipping Sigstore test: gitsign not installed");
        return;
    }
    
    let config = SigstoreConfig {
        email: "test@example.com".to_string(),
        ..Default::default()
    };
    
    // Configure Git
    SigstoreGitConfig::configure(&config.email).unwrap();
    
    // Test signing (will prompt for auth in real usage)
    // In tests, we mock the OAuth flow
    let result = SigstoreGitConfig::test_signing().await;
    assert!(result.is_ok());
}
```

**Debugging Guide**:

**Issue**: "gitsign not found"
**Solution**: Install gitsign: 
```bash
# macOS
brew install sigstore/tap/gitsign

# Linux
wget https://github.com/sigstore/gitsign/releases/latest/download/gitsign_linux_amd64
chmod +x gitsign_linux_amd64
sudo mv gitsign_linux_amd64 /usr/local/bin/gitsign
```

**Issue**: OAuth flow fails
**Solution**: Check browser popup blockers, ensure internet connectivity

**Issue**: Certificate expired
**Info**: Sigstore certificates are short-lived (10 minutes), this is normal

---

### 🛑 CHECKPOINT 8.1: Advanced Signing Methods Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 20 hours + 4 hours review = 24 hours total

**Pre-Checkpoint Checklist**:
- [ ] x509 certificate discovery working
- [ ] x509 Git configuration functional
- [ ] Sigstore authentication flow complete
- [ ] Sigstore signing operational
- [ ] Verification mechanisms in place
- [ ] All signing tests passing

**Review Focus**:
- Security of key handling
- Certificate validation
- OAuth flow security

---

## Advanced Concept: Understanding Sigstore

### What is Sigstore?

💡 **Junior Dev Deep Dive**: Sigstore Ecosystem
**Problem it solves**: Traditional code signing requires managing private keys, which is hard
**Innovation**: Use your existing identity (Google/GitHub account) to sign
**Trust model**: Short-lived certificates + transparency logs = verifiable history

**Visual Sigstore Architecture**:
```
┌─────────────────────────────────────────────────────────────┐
│                    Sigstore Ecosystem                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Developer                Identity Provider                 │
│     You     ─────────>    (Google/GitHub)                  │
│      │                          │                           │
│      │                          │ Verify Identity           │
│      │                          ▼                           │
│      │                    Fulcio (CA)                       │
│      │                 Issues Certificate                   │
│      │                          │                           │
│      ▼                          ▼                           │
│  Sign Commit ─────────> Certificate + Signature            │
│      │                          │                           │
│      │                          ▼                           │
│      │                    Rekor (Log)                       │
│      │                 Records Everything                   │
│      │                          │                           │
│      ▼                          ▼                           │
│  Push to Git ─────────> Others Can Verify                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Key Components Explained

#### 1. **Fulcio - Certificate Authority**
```
What it does: Issues short-lived certificates
How it works:
1. You prove identity via OAuth
2. Fulcio issues certificate with your email
3. Certificate expires in 10 minutes
4. No long-term keys to manage!
```

#### 2. **Rekor - Transparency Log**
```
What it does: Immutable record of all signatures
How it works:
1. Every signature gets logged
2. Creates merkle tree for verification
3. Anyone can audit the log
4. Prevents backdating signatures
```

#### 3. **Cosign - Signing Tool**
```
What it does: Actually signs artifacts
How it works:
1. Gets ephemeral key pair
2. Gets certificate from Fulcio
3. Signs with private key
4. Logs to Rekor
5. Discards private key
```

### Why Sigstore is Revolutionary

**Traditional Signing**:
```
Problems:
- Generate GPG/SSH key
- Keep private key secure forever
- Distribute public key
- Handle key rotation
- Manage revocation
```

**Sigstore Signing**:
```
Benefits:
- No permanent keys
- Use existing identity
- Automatic key rotation
- Built-in transparency
- Easy verification
```

### Implementation Best Practices

1. **Identity Provider Selection**
   ```rust
   // Choose based on your organization
   match organization_type {
       OrgType::OpenSource => "github",  // Use GitHub identity
       OrgType::Enterprise => "google",  // Use Google Workspace
       OrgType::Microsoft => "microsoft", // Use Azure AD
   }
   ```

2. **Certificate Caching**
   ```rust
   // Certificates are short-lived, but cache during session
   struct CertificateCache {
       cert: Certificate,
       expires_at: Instant,
   }
   
   impl CertificateCache {
       fn is_valid(&self) -> bool {
           Instant::now() < self.expires_at
       }
   }
   ```

3. **Offline Verification**
   ```rust
   // Can verify signatures without internet
   // using embedded Rekor public key
   const REKOR_PUBLIC_KEY: &str = include_str!("rekor.pub");
   ```

### Security Considerations

**OAuth Security**:
- Always use PKCE flow
- Validate redirect URLs
- Check token expiry
- Use secure random state

**Certificate Validation**:
- Verify certificate chain
- Check email claim matches
- Validate timestamps
- Ensure not expired

**Transparency Log**:
- Verify inclusion proof
- Check consistency proof
- Validate tree hash
- Monitor for tampering

---

## Advanced Concept: Remote Configuration Security

### Threat Model for Remote Configs

💡 **Junior Dev Security Lesson**: Never Trust Remote Data
**Threats**: Man-in-the-middle, DNS hijacking, compromised servers
**Defense**: Multiple layers of validation
**Goal**: Safe even if attacker controls the network

**Visual Threat Model**:
```
Attacker Capabilities          Our Defenses
─────────────────────         ──────────────
DNS Hijacking        ───►     HTTPS Only + Cert Pinning
                             
MITM Attack         ───►      Certificate Validation
                             
Malicious Config    ───►      Content Validation
                             
Large Files         ───►      Size Limits
                             
Slow Response       ───►      Timeouts
                             
Local File Access   ───►      Path Restrictions
```

### Security Implementation Layers

1. **Network Security**
   ```rust
   // Force HTTPS with modern TLS
   let client = Client::builder()
       .https_only(true)
       .min_tls_version(Version::TLS_1_2)
       .danger_accept_invalid_certs(false)
       .build()?;
   ```

2. **Content Security Policy**
   ```rust
   struct SecurityPolicy {
       max_size: usize,              // 1MB max
       allowed_schemes: Vec<&str>,   // ["https"]
       forbidden_hosts: Vec<&str>,   // ["localhost", "127.0.0.1"]
       timeout: Duration,            // 30 seconds
   }
   ```

3. **Input Validation Pipeline**
   ```rust
   async fn validate_remote_config(data: &str) -> Result<Profile, SecurityError> {
       // Stage 1: Size check
       if data.len() > MAX_CONFIG_SIZE {
           return Err(SecurityError::TooLarge);
       }
       
       // Stage 2: Parse safely
       let parsed = parse_with_limits(data)?;
       
       // Stage 3: Schema validation
       validate_schema(&parsed)?;
       
       // Stage 4: Semantic validation
       validate_semantics(&parsed)?;
       
       // Stage 5: Sanitization
       sanitize_config(parsed)
   }
   ```

---

## Week 2: Remote Configuration Import

### 5D.3 Secure Remote Import (40 hours)

#### Task 5D.3.1: Remote Configuration Fetching (20 hours)

💡 **Junior Dev Concept**: Secure Remote Configuration
**What it is**: Downloading profile configurations from URLs
**Why carefully**: Remote data can be malicious
**Security first**: Validate everything, trust nothing

**Security Requirements**:
1. HTTPS only (no HTTP)
2. Certificate validation
3. Size limits
4. Timeout protection
5. Content validation
6. Sanitization

**Implementation**:

1. **Create Remote Import Module** (5 hours)
   ```rust
   // src/remote/import.rs
   
   use reqwest::{Client, Certificate};
   use std::time::Duration;
   
   /// Secure remote configuration importer
   pub struct RemoteImporter {
       client: Client,
       max_size: usize,
       timeout: Duration,
   }
   
   impl RemoteImporter {
       pub fn new() -> Result<Self, RemoteError> {
           let client = Client::builder()
               .timeout(Duration::from_secs(30))
               .use_rustls_tls() // Use Rust TLS
               .https_only(true) // Enforce HTTPS
               .build()?;
           
           Ok(Self {
               client,
               max_size: 1_048_576, // 1MB max
               timeout: Duration::from_secs(30),
           })
       }
       
       /// Import profile from URL
       pub async fn import_profile(
           &self,
           url: &str,
       ) -> Result<Profile, RemoteError> {
           // Validate URL
           let url = self.validate_url(url)?;
           
           // Fetch with limits
           let response = self.client
               .get(&url)
               .timeout(self.timeout)
               .send()
               .await?;
           
           // Check response
           if !response.status().is_success() {
               return Err(RemoteError::HttpError(response.status()));
           }
           
           // Check content type
           let content_type = response
               .headers()
               .get("content-type")
               .and_then(|v| v.to_str().ok())
               .unwrap_or("");
           
           if !self.is_valid_content_type(content_type) {
               return Err(RemoteError::InvalidContentType(
                   content_type.to_string()
               ));
           }
           
           // Read with size limit
           let content = self.read_limited(response).await?;
           
           // Parse and validate
           self.parse_and_validate(content)
       }
       
       /// Validate URL is safe
       fn validate_url(&self, url: &str) -> Result<String, RemoteError> {
           let parsed = url::Url::parse(url)?;
           
           // Must be HTTPS
           if parsed.scheme() != "https" {
               return Err(RemoteError::InsecureProtocol);
           }
           
           // No local addresses
           if let Some(host) = parsed.host_str() {
               if self.is_local_address(host) {
                   return Err(RemoteError::LocalAddress);
               }
           }
           
           Ok(url.to_string())
       }
       
       /// Check if address is local/private
       fn is_local_address(&self, host: &str) -> bool {
           host == "localhost" ||
           host == "127.0.0.1" ||
           host.starts_with("192.168.") ||
           host.starts_with("10.") ||
           host.starts_with("172.")
       }
   }
   ```

2. **Implement Content Validation** (10 hours)
   ```rust
   impl RemoteImporter {
       /// Parse and validate profile content
       fn parse_and_validate(
           &self,
           content: String,
       ) -> Result<Profile, RemoteError> {
           // Try different formats
           let profile = if content.trim().starts_with('{') {
               // JSON
               serde_json::from_str(&content)?
           } else if content.contains('[') && content.contains(']') {
               // TOML
               toml::from_str(&content)?
           } else {
               // YAML
               serde_yaml::from_str(&content)?
           };
           
           // Validate profile
           self.validate_profile(&profile)?;
           
           Ok(profile)
       }
       
       /// Validate imported profile
       fn validate_profile(&self, profile: &Profile) -> Result<(), RemoteError> {
           // Check required fields
           if profile.name.is_empty() {
               return Err(RemoteError::ValidationError(
                   "Profile name cannot be empty".into()
               ));
           }
           
           // Validate name (no path traversal)
           if profile.name.contains('/') || profile.name.contains('\\') {
               return Err(RemoteError::ValidationError(
                   "Invalid profile name".into()
               ));
           }
           
           // Validate email
           if !self.is_valid_email(&profile.git.user_email) {
               return Err(RemoteError::ValidationError(
                   "Invalid email address".into()
               ));
           }
           
           // Check for suspicious content
           if let Some(signing) = &profile.signing {
               if let Some(key) = &signing.ssh_key {
                   // Ensure it's a reference, not actual key
                   if key.contains("BEGIN") || key.len() > 500 {
                       return Err(RemoteError::ValidationError(
                           "Profile contains private key data".into()
                       ));
                   }
               }
           }
           
           Ok(())
       }
   }
   ```

3. **Create Import UI** (5 hours)
   ```rust
   /// Interactive import with user confirmation
   pub struct ImportWizard;
   
   impl ImportWizard {
       pub async fn import_interactive(
           &self,
           url: &str,
       ) -> Result<Profile, RemoteError> {
           println!("Importing profile from: {}", url);
           println!("⚠️  Only import from trusted sources!");
           
           // Show URL details
           let parsed = url::Url::parse(url)?;
           println!("Host: {}", parsed.host_str().unwrap_or("unknown"));
           
           // Confirm
           print!("Continue? [y/N]: ");
           io::stdout().flush()?;
           
           let mut input = String::new();
           io::stdin().read_line(&mut input)?;
           
           if !input.trim().eq_ignore_ascii_case("y") {
               return Err(RemoteError::UserCancelled);
           }
           
           // Import with progress
           let importer = RemoteImporter::new()?;
           let profile = importer.import_profile(url).await?;
           
           // Show what will be imported
           println!("\nProfile to import:");
           println!("  Name: {}", profile.name);
           println!("  Email: {}", profile.git.user_email);
           println!("  User: {}", profile.git.user_name);
           
           // Final confirmation
           print!("\nImport this profile? [y/N]: ");
           io::stdout().flush()?;
           
           input.clear();
           io::stdin().read_line(&mut input)?;
           
           if input.trim().eq_ignore_ascii_case("y") {
               Ok(profile)
           } else {
               Err(RemoteError::UserCancelled)
           }
       }
   }
   ```

**Security Testing**:
```rust
#[tokio::test]
async fn test_reject_http_url() {
    let importer = RemoteImporter::new().unwrap();
    let result = importer.import_profile("http://example.com/profile.toml").await;
    assert!(matches!(result, Err(RemoteError::InsecureProtocol)));
}

#[tokio::test]
async fn test_reject_local_url() {
    let importer = RemoteImporter::new().unwrap();
    let result = importer.import_profile("https://localhost/profile.toml").await;
    assert!(matches!(result, Err(RemoteError::LocalAddress)));
}

#[tokio::test]
async fn test_reject_private_key() {
    // Test profile with embedded private key
    let malicious = r#"
    [profile]
    name = "evil"
    [profile.signing]
    ssh_key = "-----BEGIN OPENSSH PRIVATE KEY-----\nMIIE..."
    "#;
    
    let importer = RemoteImporter::new().unwrap();
    let result = importer.parse_and_validate(malicious.to_string());
    assert!(matches!(result, Err(RemoteError::ValidationError(_))));
}
```

---

### 🛑 FINAL CHECKPOINT 8: Advanced Features Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Final Deliverables**:
- x509 certificate signing support
- Sigstore keyless signing (if implemented)
- Secure remote import with validation
- Comprehensive security measures
- All tests passing

**Security Review Required**:
- No credential leaks
- Input validation complete
- HTTPS enforcement working
- Size/timeout limits enforced
- No code injection vectors

---

## Common Issues and Solutions

### x509 Issues
**Issue**: "Certificate not found"
**Solution**: Check certificate has .key file alongside

**Issue**: "gpgsm not found"
**Solution**: Install GnuPG 2.1+ with S/MIME support

### Remote Import Issues
**Issue**: "SSL certificate problem"
**Solution**: Update CA certificates on system

**Issue**: "Timeout"
**Solution**: Check network, increase timeout limit

## Security Checklist

Before completing Phase 8:
- [ ] All remote fetches use HTTPS
- [ ] Certificate validation enabled
- [ ] Input validation comprehensive
- [ ] No private keys accepted
- [ ] Size limits enforced
- [ ] Timeouts configured
- [ ] Local addresses blocked
- [ ] User confirmation required

## Summary

Phase 8 completes the advanced features with a strong focus on security. The careful implementation ensures users can safely use modern signing methods and share configurations.

---

## Advanced Concepts Reference

### Understanding x509 Certificates

💡 **Junior Dev Deep Dive**: x509 Digital Certificates
**What they are**: Digital ID cards for computers and people
**How they work**: Like a passport - issued by trusted authority
**Usage**: HTTPS websites, email encryption, code signing

**Visual x509 Certificate Structure**:
```
┌─────────────────────────────────────────┐
│         x509 Certificate                 │
├─────────────────────────────────────────┤
│ Version: 3                              │
│ Serial Number: 12:34:56:78:90:AB        │
├─────────────────────────────────────────┤
│ Issuer (Who created it):                │
│   CN=DigiCert SHA2                      │
│   O=DigiCert Inc                        │
│   C=US                                  │
├─────────────────────────────────────────┤
│ Subject (Who it's for):                 │
│   CN=Alice Smith                        │
│   emailAddress=alice@company.com        │
│   O=ACME Corp                           │
├─────────────────────────────────────────┤
│ Validity:                               │
│   Not Before: 2024-01-01               │
│   Not After:  2025-01-01               │
├─────────────────────────────────────────┤
│ Public Key:                             │
│   Algorithm: RSA 2048                   │
│   Modulus: 30:82:01:0a:02:82...       │
├─────────────────────────────────────────┤
│ Extensions:                             │
│   Key Usage: Digital Signature          │
│   Email: alice@company.com              │
├─────────────────────────────────────────┤
│ Signature:                              │
│   Algorithm: SHA256withRSA              │
│   Value: 65:f3:45:12:89:ac...         │
└─────────────────────────────────────────┘
```

**Certificate Chain of Trust**:
```
Root CA (Self-Signed)
   │
   └─► Intermediate CA
          │
          └─► Your Certificate
```

### Understanding HTTPS/TLS

💡 **Junior Dev Security**: How HTTPS Protects You
**Problem**: Internet traffic can be intercepted
**Solution**: Encryption + authentication
**Result**: Safe communication

**Visual TLS Handshake**:
```
Client (git-setup)                    Server (github.com)
──────────────────                   ─────────────────────
     │                                        │
     ├──── 1. Hello, cipher suites ────────► │
     │                                        │
     │ ◄──── 2. Certificate, chosen cipher ─┤
     │                                        │
     ├──── 3. Verify cert, send key ──────► │
     │                                        │
     │ ◄──── 4. Finished ──────────────────┤
     │                                        │
     ├════ 5. Encrypted data ═════════════► │
     │                                        │
```

**What Each Step Does**:
1. **Client Hello**: "I support these encryption methods"
2. **Server Response**: "Here's my certificate and let's use AES"
3. **Key Exchange**: Client generates session key
4. **Confirmation**: Both sides confirm handshake
5. **Secure Channel**: All data now encrypted

### Understanding OAuth/OIDC

💡 **Junior Dev Auth**: How Sigstore Uses Your Identity
**OAuth**: Lets apps use your Google/GitHub account
**OIDC**: Adds identity information to OAuth
**Result**: Prove who you are without passwords

**Visual OAuth Flow for Sigstore**:
```
┌──────────────────────────────────────────────────────┐
│                  OAuth 2.0 + OIDC Flow               │
├──────────────────────────────────────────────────────┤
│                                                      │
│  1. User wants to sign                              │
│     └─► git-setup starts flow                       │
│                                                      │
│  2. Redirect to Google/GitHub                       │
│     └─► Browser opens login page                    │
│                                                      │
│  3. User logs in                                    │
│     └─► Google verifies password/2FA                │
│                                                      │
│  4. Consent screen                                  │
│     └─► "Allow Sigstore to see your email?"         │
│                                                      │
│  5. Authorization code                              │
│     └─► Redirect back with temporary code           │
│                                                      │
│  6. Exchange for ID token                           │
│     └─► Code → ID Token with email claim            │
│                                                      │
│  7. Use token for certificate                       │
│     └─► Fulcio trusts Google, issues cert           │
│                                                      │
└──────────────────────────────────────────────────────┘
```

**Security Properties**:
- User never gives password to git-setup
- Google/GitHub verifies identity
- Token expires quickly (10 minutes)
- Can't be replayed or stolen

### Certificate Validation Best Practices

**What to Check**:
```rust
fn validate_certificate(cert: &X509) -> Result<(), CertError> {
    // 1. Check validity period
    let now = SystemTime::now();
    if now < cert.not_before() || now > cert.not_after() {
        return Err(CertError::Expired);
    }
    
    // 2. Verify certificate chain
    verify_chain(cert, &trusted_roots)?;
    
    // 3. Check key usage
    if !cert.key_usage().includes(KeyUsage::DigitalSignature) {
        return Err(CertError::WrongUsage);
    }
    
    // 4. Validate subject
    if !cert.subject_email().is_valid() {
        return Err(CertError::InvalidSubject);
    }
    
    // 5. Check revocation (OCSP)
    check_revocation(cert)?;
    
    Ok(())
}
```

### Network Security Checklist

**For Remote Configuration Import**:
```
□ HTTPS enforced (no HTTP fallback)
□ Certificate validation enabled
□ Hostname verification on
□ Modern TLS version (1.2+)
□ Strong cipher suites only
□ Timeout configured (30s max)
□ Size limit enforced (1MB)
□ Rate limiting implemented
□ DNS validation
□ No local network access
□ Content-Type validation
□ Charset validation (UTF-8)
□ Schema validation
□ Sanitization complete
□ Error messages safe
```

### Debugging Network Issues

**Common Problems and Solutions**:

1. **Certificate Errors**
   ```
   Error: "certificate verify failed"
   Causes:
   - Self-signed certificate
   - Expired certificate
   - Wrong hostname
   - Missing CA bundle
   
   Debug:
   openssl s_client -connect host:443 -servername host
   ```

2. **Timeout Issues**
   ```
   Error: "operation timed out"
   Causes:
   - Slow network
   - Firewall blocking
   - Server overloaded
   
   Debug:
   curl -w "@curl-format.txt" -o /dev/null -s https://example.com
   ```

3. **TLS Negotiation Failures**
   ```
   Error: "SSL handshake failed"
   Causes:
   - Old TLS version
   - No common ciphers
   - Client cert required
   
   Debug:
   nmap --script ssl-enum-ciphers -p 443 host
   ```

### Additional Resources

**For Deep Learning**:
- 📚 [The Illustrated TLS Connection](https://tls.ulfheim.net/)
- 📚 [OAuth 2.0 Simplified](https://aaronparecki.com/oauth-2-simplified/)
- 📚 [x509 Certificate Primer](https://www.ssl.com/faqs/what-is-an-x509-certificate/)
- 📚 [Sigstore Architecture](https://docs.sigstore.dev/architecture/overview/)

**Security References**:
- 🔒 [OWASP TLS Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html)
- 🔒 [Mozilla SSL Configuration](https://ssl-config.mozilla.org/)
- 🔒 [Certificate Transparency](https://certificate.transparency.dev/)

---

**Next**: Phase 9 - Platform Support & Final Polish