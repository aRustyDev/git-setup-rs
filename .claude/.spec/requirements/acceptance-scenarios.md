# Acceptance Scenarios - git-setup-rs

## Core User Scenarios

### SCENARIO: CORE-001 - First-Time Profile Creation
**Feature**: Profile Management  
**Priority**: Must Have

**Given**: A developer has installed git-setup-rs but has no profiles configured  
**When**: They run `git setup` or launch the TUI  
**Then**: They are guided through a profile creation wizard that configures their first Git identity

**Test Data**:
```json
{
  "input": {
    "profile_name": "work",
    "user_name": "Jane Developer",
    "user_email": "jane@company.com",
    "signing_method": "ssh",
    "key_source": "1password",
    "key_reference": "op://Private/Work SSH Key/public key"
  },
  "expected": {
    "status": "success",
    "profile_created": true,
    "git_config_updated": true,
    "signing_configured": true
  }
}
```

**Acceptance Criteria**:
- [ ] Profile creation completes in <30 seconds
- [ ] TUI validates email format in real-time
- [ ] 1Password integration lists available SSH keys
- [ ] Profile is immediately usable for Git operations
- [ ] Configuration persists across terminal sessions

### SCENARIO: CORE-002 - Quick Profile Switching
**Feature**: Profile Switching  
**Priority**: Must Have

**Given**: A developer has multiple profiles configured (work, personal, client)  
**When**: They execute `git setup --profile personal` in a repository  
**Then**: All Git configurations switch to the personal profile settings

**Test Data**:
```toml
[profiles.work]
name = "Jane Developer"
email = "jane@company.com"
signingkey = "op://Private/Work SSH Key/public key"

[profiles.personal]
name = "Jane Smith"
email = "jane.smith@gmail.com"
signingkey = "~/.ssh/id_ed25519_personal"
```

**Acceptance Criteria**:
- [ ] Profile switch completes in <3 seconds
- [ ] Git config reflects new identity immediately
- [ ] Previous config is cleanly replaced (no remnants)
- [ ] Signing configuration updates correctly
- [ ] Operation works for --local, --global, and --system scopes

### SCENARIO: CORE-003 - Auto-Detection by Repository URL
**Feature**: Automatic Profile Detection  
**Priority**: Must Have

**Given**: A profile has URL patterns configured for auto-detection  
**When**: The developer enters a Git repository matching the pattern  
**Then**: git-setup-rs prompts to apply the matching profile

**Test Data**:
```yaml
profiles:
  work:
    name: "Jane Developer"
    email: "jane@company.com"
    auto_detect:
      - "github.com:company-org/*"
      - "gitlab.company.internal:*"
```

**Acceptance Criteria**:
- [ ] Detection triggers within 100ms of entering directory
- [ ] User is prompted with matching profile name
- [ ] Prompt can be accepted (Enter), declined (Esc), or remembered
- [ ] Detection works with SSH and HTTPS remote URLs
- [ ] Multiple matching patterns prioritize most specific match

### SCENARIO: CORE-004 - 1Password SSH Agent Integration
**Feature**: 1Password Integration  
**Priority**: Must Have

**Given**: Developer has 1Password CLI installed and authenticated  
**When**: They create a profile and select "1Password" as key source  
**Then**: They can select from their 1Password SSH keys and use them for signing

**Test Data**:
```bash
# Mock 1Password CLI output
op item list --categories "SSH Key" --format=json
[
  {
    "id": "abc123",
    "title": "Work SSH Key",
    "vault": {"name": "Private"},
    "category": "SSH_KEY"
  },
  {
    "id": "def456",
    "title": "Personal GitHub",
    "vault": {"name": "Personal"},
    "category": "SSH_KEY"
  }
]
```

**Acceptance Criteria**:
- [ ] Lists all SSH keys from accessible vaults
- [ ] Biometric authentication triggers appropriately
- [ ] Selected key reference is stored (not the key itself)
- [ ] Git commits can be signed without manual key handling
- [ ] Works without SERVICE_ACCOUNT_TOKEN

### SCENARIO: CORE-005 - Complex Signing Configuration
**Feature**: Git Signing Configuration  
**Priority**: Must Have

**Given**: A developer needs different signing methods for different profiles  
**When**: They configure profiles with GPG, SSH, x509, or gitsign  
**Then**: Each profile correctly configures Git for the specified signing method

**Test Data**:
```toml
[profiles.personal]
signing_method = "ssh"
signing_key = "~/.ssh/id_ed25519"

[profiles.work]
signing_method = "gpg"
signing_key = "ABCD1234EFGH5678"

[profiles.opensource]
signing_method = "gitsign"
gitsign_oidc_provider = "github"
```

**Acceptance Criteria**:
- [ ] SSH signing configures allowed_signers file
- [ ] GPG signing sets correct gpg.program
- [ ] x509 signing works with certificates
- [ ] gitsign configures Sigstore integration
- [ ] Signing method changes don't conflict

## Edge Cases

### SCENARIO: EDGE-001 - Missing 1Password CLI
**Priority**: Must Have

**Given**: User selects 1Password integration but doesn't have `op` CLI installed  
**When**: They attempt to use 1Password features  
**Then**: Clear error message with installation instructions is shown

**Acceptance Criteria**:
- [ ] Error detected before attempting operations
- [ ] Message includes platform-specific install commands
- [ ] Graceful fallback to manual key entry
- [ ] TUI doesn't crash, remains usable

### SCENARIO: EDGE-002 - Corrupted Profile Configuration
**Priority**: Should Have

**Given**: The profiles.toml file has been corrupted or manually edited incorrectly  
**When**: git-setup-rs attempts to load profiles  
**Then**: It provides helpful error messages and recovery options

**Acceptance Criteria**:
- [ ] Parsing errors show line/column numbers
- [ ] Backup of last known good config exists
- [ ] Option to restore from backup
- [ ] Option to start fresh with wizard
- [ ] Critical errors are logged for debugging

### SCENARIO: EDGE-003 - Cross-Platform Path Handling
**Priority**: Must Have

**Given**: A profile is created on Windows with path `C:\Users\jane\.ssh\key`  
**When**: The same profile is synced to macOS/Linux  
**Then**: Paths are automatically adjusted to platform conventions

**Acceptance Criteria**:
- [ ] Windows paths convert to Unix paths
- [ ] Home directory expansion works (~)
- [ ] Environment variables are resolved
- [ ] WSL paths are handled correctly
- [ ] Invalid paths show clear errors

## Performance Scenarios

### SCENARIO: PERF-001 - Large Profile Set
**Priority**: Should Have

**Given**: A power user has 50+ profiles configured  
**When**: They search for a profile in the TUI  
**Then**: Fuzzy search returns results instantly

**Test Data**:
```yaml
# Generate 100 test profiles
profiles:
  client-01-prod: { name: "Dev", email: "dev@client01.com" }
  client-01-staging: { name: "Dev", email: "dev@client01-staging.com" }
  # ... 98 more profiles
```

**Acceptance Criteria**:
- [ ] TUI launches in <100ms regardless of profile count
- [ ] Fuzzy search responds in <10ms per keystroke
- [ ] Memory usage stays under 50MB
- [ ] Profile list renders without lag
- [ ] Scrolling is smooth at 60fps

### SCENARIO: PERF-002 - Rapid Profile Switching
**Priority**: Should Have

**Given**: A developer is switching between multiple client projects  
**When**: They rapidly switch profiles 10 times in succession  
**Then**: Each switch completes successfully without delays or conflicts

**Acceptance Criteria**:
- [ ] No file lock conflicts occur
- [ ] Each switch completes in <3 seconds
- [ ] Git config remains consistent
- [ ] No memory leaks over repeated operations
- [ ] CPU usage stays reasonable (<25%)

## Security Scenarios

### SCENARIO: SEC-001 - Credential Memory Safety
**Priority**: Must Have

**Given**: Sensitive data (keys, tokens) is processed by git-setup-rs  
**When**: The data is no longer needed  
**Then**: It is securely wiped from memory using zeroize

**Acceptance Criteria**:
- [ ] No credentials in memory dumps
- [ ] Zeroize applied to all SecureString types
- [ ] No credentials in swap files
- [ ] Clean shutdown clears all sensitive data
- [ ] Crash dumps don't contain secrets

### SCENARIO: SEC-002 - Remote Profile Verification
**Priority**: Must Have

**Given**: A user imports a profile from a GitHub Gist  
**When**: The profile is downloaded  
**Then**: Its integrity is verified before use

**Test Data**:
```yaml
# Profile includes hash
meta:
  source: "https://gist.github.com/user/abc123"
  sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
```

**Acceptance Criteria**:
- [ ] SHA-256 hash verification passes
- [ ] Modified profiles are rejected
- [ ] User warned of verification failures
- [ ] HTTPS required for remote sources
- [ ] Local cache validated on each use

---
*These scenarios form the basis for acceptance testing and TDD implementation.*