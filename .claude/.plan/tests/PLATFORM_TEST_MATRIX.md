# Platform Test Matrix for git-setup-rs

This comprehensive test matrix ensures git-setup-rs works correctly across all supported platforms.

## Supported Platforms

### Tier 1 (Must Work Perfectly)
| Platform | OS Version | Architecture | Shell | Notes |
|----------|------------|--------------|-------|-------|
| Windows 11 | 22H2+ | x86_64 | PowerShell 7+, cmd | Primary Windows target |
| Windows 10 | 21H2+ | x86_64 | PowerShell 5.1, Git Bash | Legacy Windows |
| macOS 14 | Sonoma | arm64 (M1/M2) | zsh, bash | Apple Silicon |
| macOS 13 | Ventura | x86_64 | zsh, bash | Intel Macs |
| Ubuntu 22.04 | LTS | x86_64 | bash, zsh | Primary Linux |
| Ubuntu 20.04 | LTS | x86_64 | bash | Legacy Linux |

### Tier 2 (Should Work)
| Platform | OS Version | Architecture | Shell | Notes |
|----------|------------|--------------|-------|-------|
| Fedora 39 | Latest | x86_64 | bash, zsh | RPM-based |
| Arch Linux | Rolling | x86_64 | bash, zsh, fish | Bleeding edge |
| Debian 12 | Bookworm | x86_64 | bash | Stable |
| Windows 11 | 22H2+ | arm64 | PowerShell | Surface Pro X |
| macOS 12 | Monterey | x86_64 | zsh | Older macOS |

## Core Functionality Tests

### 1. Installation Tests

#### Windows Installation
```powershell
# Test 1.1: PowerShell installer
irm https://install.git-setup.rs | iex
git-setup --version

# Test 1.2: Manual installation
# Download git-setup.exe
./git-setup.exe --version

# Test 1.3: PATH configuration
where git-setup
# Should be in PATH after restart
```

#### macOS Installation
```bash
# Test 1.4: Homebrew
brew install git-setup-rs/tap/git-setup
git-setup --version

# Test 1.5: Shell script
curl -fsSL https://install.git-setup.rs | sh
git-setup --version

# Test 1.6: Manual universal binary
# Download and verify both architectures work
file git-setup  # Should show "universal binary"
```

#### Linux Installation
```bash
# Test 1.7: Shell installer
curl -fsSL https://install.git-setup.rs | sh
git-setup --version

# Test 1.8: Package managers
# Ubuntu/Debian
sudo apt install git-setup

# Fedora
sudo dnf install git-setup
```

### 2. Path Handling Tests

#### Windows Path Tests
```powershell
# Test 2.1: Drive letters
git-setup config set test.path "C:\Users\Test"
git-setup config get test.path
# Expected: Normalized path

# Test 2.2: UNC paths
git-setup config set test.unc "\\\\server\\share\\folder"
# Should handle correctly

# Test 2.3: Long paths (>260 chars)
$longPath = "C:\Very\" + ("Long\" * 50) + "Path"
git-setup config set test.long $longPath
# Should work with long path support

# Test 2.4: Special characters
git-setup profile create "my-project (2023)"
# Should handle spaces and parentheses
```

#### Unix Path Tests
```bash
# Test 2.5: Home directory expansion
git-setup config set test.home "~/Documents"
git-setup config get test.home
# Expected: /home/username/Documents

# Test 2.6: Symlinks
ln -s /tmp/real /tmp/link
git-setup config set test.link "/tmp/link/file"
# Should resolve correctly

# Test 2.7: Relative paths
cd /home/user
git-setup config set test.rel "./config/file"
# Should resolve to absolute
```

### 3. Profile Management Tests

#### Basic Profile Operations
```bash
# Test 3.1: Create profile
git-setup profile create --name work \
  --email work@company.com \
  --user-name "John Doe"

# Test 3.2: List profiles
git-setup profile list
# Expected: Shows "work" profile

# Test 3.3: Show profile
git-setup profile show work
# Expected: All details displayed

# Test 3.4: Edit profile
git-setup profile edit work
# Should open editor or prompt

# Test 3.5: Delete profile
git-setup profile delete work --force
# Should remove without confirmation
```

#### Profile Inheritance
```bash
# Test 3.6: Create base profile
git-setup profile create --name base \
  --email base@company.com

# Test 3.7: Create child profile
git-setup profile create --name dev \
  --extends base \
  --email dev@company.com

# Test 3.8: Verify inheritance
git-setup profile show dev
# Should show merged configuration
```

### 4. Git Integration Tests

#### Local Repository
```bash
# Test 4.1: Apply profile locally
mkdir test-repo && cd test-repo
git init
git-setup profile use work

# Test 4.2: Verify Git config
git config user.email
# Expected: work@company.com

# Test 4.3: Switch profiles
git-setup profile use personal
git config user.email
# Expected: personal@example.com
```

#### Global Configuration
```bash
# Test 4.4: Apply globally
git-setup profile use work --global

# Test 4.5: Verify global config
git config --global user.email
# Expected: work@company.com

# Test 4.6: Override precedence
cd repo-with-local-config
git config user.email
# Local should override global
```

### 5. UI Tests

#### CLI Tests
```bash
# Test 5.1: Help system
git-setup --help
git-setup profile --help
git-setup profile create --help

# Test 5.2: Output formats
git-setup profile list --format json
git-setup profile list --format yaml

# Test 5.3: Quiet mode
git-setup profile use work --quiet
# No output on success

# Test 5.4: Verbose mode
git-setup profile use work --verbose
# Detailed output
```

#### TUI Tests
```bash
# Test 5.5: Launch TUI
git-setup tui

# Test 5.6: Keyboard navigation
# - Arrow keys move selection
# - Enter selects
# - Escape goes back
# - q quits

# Test 5.7: Terminal compatibility
# Test in different terminals:
# - Windows Terminal
# - iTerm2
# - GNOME Terminal
# - Alacritty

# Test 5.8: Resize handling
# Resize terminal while TUI running
# Should adapt layout
```

### 6. Security Tests

#### Credential Handling
```bash
# Test 6.1: 1Password integration
git-setup signing setup --method ssh --key op://Personal/GitHub/private

# Test 6.2: No credential leaks
git-setup profile show work --show-secrets 2>&1 | grep -i password
# Should not show actual passwords

# Test 6.3: Secure storage
ls -la ~/.config/git-setup/
# Check file permissions (600 or 644)
```

#### Path Security
```bash
# Test 6.4: Path traversal prevention
git-setup profile create --name "../../../etc/passwd"
# Should be rejected

# Test 6.5: Symlink attacks
ln -s /etc/passwd ~/.config/git-setup/evil
git-setup profile show evil
# Should not follow malicious symlinks
```

### 7. Platform-Specific Tests

#### Windows-Specific
```powershell
# Test 7.1: Windows signing
git-setup signing setup --method x509

# Test 7.2: Registry integration
# Check if PATH updated in registry

# Test 7.3: WSL interop
wsl git-setup --version
# Should work from WSL
```

#### macOS-Specific
```bash
# Test 7.4: Keychain integration
git-setup signing setup --method ssh
# Should offer to use keychain

# Test 7.5: Gatekeeper
# Download and run without quarantine issues

# Test 7.6: Universal binary
lipo -info $(which git-setup)
# Should show both architectures
```

#### Linux-Specific
```bash
# Test 7.7: XDG compliance
echo $XDG_CONFIG_HOME
ls $XDG_CONFIG_HOME/git-setup/
# Should respect XDG standards

# Test 7.8: Package manager integration
# Updates should work through package manager
```

### 8. Performance Tests

Run on each platform:
```bash
# Test 8.1: Startup time
time git-setup --version
# Should be <100ms

# Test 8.2: Profile operations
time git-setup profile list
# Should be <20ms

# Test 8.3: Memory usage
/usr/bin/time -v git-setup profile list
# Check Maximum resident set size

# Test 8.4: Large profile count
# Create 100 profiles
for i in {1..100}; do
  git-setup profile create --name "test$i" --email "test$i@example.com"
done
time git-setup profile list
# Should still be fast
```

### 9. Error Handling Tests

```bash
# Test 9.1: Missing profile
git-setup profile use nonexistent
# Clear error message

# Test 9.2: Invalid configuration
echo "invalid toml {" > ~/.config/git-setup/profiles/bad.toml
git-setup profile list
# Should handle gracefully

# Test 9.3: Permission denied
chmod 000 ~/.config/git-setup/profiles/work.toml
git-setup profile show work
# Should show permission error

# Test 9.4: Disk full
# Fill disk, try to create profile
# Should handle gracefully
```

### 10. Integration Tests

```bash
# Test 10.1: Full workflow
git-setup profile create --name project --email me@project.com
mkdir new-project && cd new-project
git init
git-setup profile use project
echo "# Project" > README.md
git add README.md
git commit -m "Initial commit"
git log --show-signature
# Everything should work together
```

## Test Automation

### CI Test Matrix
```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust: [stable, beta]
    include:
      - os: ubuntu-20.04
        rust: stable
      - os: macos-12
        rust: stable
      - os: windows-2019
        rust: stable
```

### Platform-Specific CI Tests
```yaml
# Windows-specific tests
- name: Test Windows paths
  if: matrix.os == 'windows-latest'
  run: |
    cargo test --features windows-tests
    
# macOS-specific tests  
- name: Test macOS universal
  if: matrix.os == 'macos-latest'
  run: |
    cargo test --features macos-tests
    
# Linux-specific tests
- name: Test Linux XDG
  if: matrix.os == 'ubuntu-latest'
  run: |
    cargo test --features linux-tests
```

## Manual Test Checklist

Before each release, manually verify:

### Windows
- [ ] PowerShell 5.1 installation works
- [ ] PowerShell 7+ installation works
- [ ] Git Bash compatibility
- [ ] Windows Terminal rendering
- [ ] Long path support
- [ ] UNC path handling

### macOS
- [ ] Intel Mac installation
- [ ] Apple Silicon installation  
- [ ] Universal binary runs on both
- [ ] Homebrew installation
- [ ] Keychain integration
- [ ] Gatekeeper approval

### Linux
- [ ] Ubuntu package installation
- [ ] Fedora package installation
- [ ] Shell installer works
- [ ] XDG compliance
- [ ] Various terminals work
- [ ] Permission handling

### Cross-Platform
- [ ] Profile portability
- [ ] Path normalization
- [ ] Git integration
- [ ] Performance targets met
- [ ] Error messages clear
- [ ] Documentation accurate

## Known Platform Issues

### Windows
- Git Bash may show path warnings (cosmetic)
- Some antivirus may flag as unknown

### macOS  
- First run requires security approval
- Rosetta 2 needed for x86 on M1 (automatic)

### Linux
- Snap confinement may limit file access
- Some distros need manual PATH setup

---

*Test comprehensively. A tool is only as good as its weakest platform support.*