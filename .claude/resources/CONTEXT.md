# Git-Setup Tool Context

## Project Overview

Git-Setup is a sophisticated Git configuration management tool that integrates with 1Password to manage multiple Git identities and SSH/GPG signing configurations. It allows developers to quickly switch between different Git profiles (work, personal, open source, etc.) with proper cryptographic signing setup.

## Core Purpose and User Experience

### What the Tool Does
1. **Profile Management**: Allows users to create, edit, delete, and apply Git configuration profiles
2. **1Password Integration**: Fetches SSH and GPG keys directly from 1Password vaults
3. **Commit Signing**: Automatically configures Git to sign commits using SSH, GPG, x509, or gitsign (keyless)
4. **Auto-Detection**: Can automatically detect and apply the correct profile based on repository remote URLs
5. **Interactive TUI**: Provides a user-friendly terminal interface for all operations

### User Workflow
1. **If config exists with profiles**: User can immediately use `git setup <profile_name>` to apply a profile
2. **If config exists without profiles**: User creates a new profile via `git setup --add <profile_name>` then applies it
3. **Import from 1Password**: User runs `git setup --import` to import existing SSH keys from 1Password's agent.toml (requires existing agent.toml with configured keys)
4. **Profile Application**: In any Git repository, user runs `git setup <profile-name>` to apply that configuration
5. **Automatic Signing**: All subsequent commits are automatically signed with the configured key

## Technical Architecture

### Language and Framework Context
The original implementation is in Go, using:
- **Cobra** for CLI command handling
- **Bubble Tea** for TUI (Terminal User Interface)
- **TOML** for configuration storage
- **1Password CLI** for key management
- **External CLI tools** for signing operations

For the Rust implementation, equivalent technologies would be:
- **Clap** for CLI command handling
- **Ratatui** for TUI
- **toml** crate for configuration
- **Command execution** for external CLI integration
- **tracing** crate for logging and error handling

### Data Flow
1. **Configuration Loading**:
   - Read from `~/.config/git/setup/config.toml`
   - Load profiles and default settings
   - Initialize paths defined in config.toml:
     - `config.global.path` (default: `~/.config/git/config`)
     - `config.default.path` (default: `.git/config`)
     - `config.system.path` (default: `/etc/gitconfig`)

2. **Profile Management**:
   - Profiles stored in TOML configuration
   - Each profile contains:
     - `name`: Profile identifier
     - `gitUserName`: Git user.name
     - `gitUserEmail`: Git user.email
     - `sshKeyTitle`: Title of SSH key in 1Password
     - `vaultName`: 1Password vault name
     - `signingKey`: Public key or key reference
     - `keyType`: ssh, gpg, x509, or gitsign
     - `allowedSigners`: Path to allowed signers file
     - `match`: URL patterns for auto-detection
     - `repos`: Specific repository paths
     - `includeIfDirs`: Directories for includeIf directives
     - `sshKeySource`: 1password, authorized_keys, or file
     - `sshKeyPath`: Path for static SSH keys
     - `hostPatterns`: SSH host patterns
     - `onePassword`: Boolean flag for 1Password integration
     - `scope`: local, global, or system
   - No database - all persistent storage is in TOML files
   - Imported files and all configurations must be validated and pass to be used

3. **1Password Integration**:
   - Uses `op` CLI tool via command execution
   - Supports SSH & GPG key reading, creation, and updates
   - GPG items use custom JSON structure:
     ```json
     {
       "title": "GPGKeyName",
       "category": "GPGKey",
       "sections": [
         {"id": "pub", "label": "Public"},
         {"id": "priv", "label": "Private"}
       ],
       "fields": [
         {
           "id": "key",
           "section": {"id": "pub"},
           "type": "text",
           "label": "key",
           "value": "Base64PubKeyHere"
         },
         {
           "id": "pw",
           "section": {"id": "priv"},
           "type": "password",
           "label": "password",
           "value": "GPGPasswordHere"
         },
         {
           "id": "key",
           "section": {"id": "priv"},
           "type": "password",
           "label": "key",
           "value": "Base64PrivKeyHere"
         }
       ]
     }
     ```
   - GPG items must have "gpg" tag
   - ALL cached 1Password vault listings MUST be encrypted and ONLY stored in memory

4. **Git Configuration**:
   - Target gitconfig files are defined in config.toml
   - Modifies configurations based on signing method:

   **SSH via 1Password**:
   - `gpg.format = ssh`
   - `gpg.ssh.program = path/to/op-ssh-agent` (platform-specific)
   - `user.signingkey = ssh-ed25519 AAAAC3...` (actual public key content)
   - `commit.gpgsign = true`
   - `tag.gpgsign = true`
   - `gpg.ssh.allowedSignersFile = path/to/file`

   **Manual/Static SSH**:
   - `gpg.format = ssh`
   - `gpg.ssh.program = path/to/ssh-agent` (platform-specific)
   - `user.signingkey = path/to/public.key`
   - `commit.gpgsign = true`
   - `tag.gpgsign = true`
   - `gpg.ssh.allowedSignersFile = path/to/file`

   **GPG**:
   - `user.signingkey` (obtained from gpg)
   - `gpg.program = path/to/gpg` (platform-specific)
   - `commit.gpgsign = true`
   - `tag.gpgsign = true`

   **Gitsign (keyless)**:
   - `gpg.x509.program = gitsign` (platform-specific)
   - `gpg.format = x509`
   - `commit.gpgsign = true`
   - `tag.gpgsign = true`

   **x509**:
   - `gpg.x509.program = smimesign` (platform-specific)
   - `gpg.format = x509`
   - `commit.gpgsign = true`
   - `tag.gpgsign = true`

## Key Concepts for Rust Implementation

### Profile Structure
A profile represents a complete Git identity configuration. All profiles are stored in `~/.config/git/setup/config.toml` (no separate profile files).

### Configuration Paths
- Main config: `~/.config/git/setup/config.toml`
- Git configs: Defined in config.toml with defaults
- Agent config: `~/.config/1Password/ssh/agent.toml`
- Allowed signers: Defined in config.toml (default: `~/.config/git/allowed_signers`)

### External Tool Integration
The tool wraps several external CLI tools:
- `git` - For repository operations and configuration
- `op` - 1Password CLI for key management
- `ssh-keygen` - For SSH key operations
- `gpg` - For GPG key operations
- `gitsign` - For keyless signing (Sigstore)
- `smimesign` - For x509 signing

Platform-specific paths should be configurable, with fallback to PATH search.

### GPG Key Handling
When 1Password integration is enabled:
1. Import GPG key from 1Password using custom JSON structure
2. Retrieve password using `op://<item_id>` syntax
3. Import private key into GPG keyring using password via `--passphrase-fd`

When 1Password integration is disabled or key not found:
1. Call `gpg --list-keys` to find existing keys
2. If no keys found, ask user if they want to create via `gpg --gen-key`

### Error Handling Philosophy
- Implement tracing for better error handling in all functions
- Fail fast on critical errors (missing 1Password token, etc.)
- Provide clear, actionable error messages
- Non-critical operations (like updating allowed_signers) should log warnings but not fail
- Always validate user input before processing

## Important Implementation Notes

### Test-Driven Development (TDD) Requirements
The Rust implementation MUST follow TDD principles:
1. Write tests FIRST for each component
2. Tests should cover both happy paths and error cases
3. Use table-driven tests for comprehensive coverage
4. Mock external dependencies (1Password CLI, file system operations)
5. Aim for >80% code coverage on critical paths

### Cross-Platform Considerations
- Each CLI wrapper call should expect platform-specific names
- Use combination of platform-specific feature gates and traits for external CLI tool wrappers
- SSH program paths differ between platforms
- File permissions are important (0600 for sensitive files)
- Path separators and home directory detection must be platform-aware
- Use Rust's built-in cross-platform abstractions where possible

### Security Considerations
- Never store private keys or passwords
- Service account tokens should only be read from environment variables
- Config files containing sensitive paths should have restricted permissions
- All signing operations happen through external tools
- Cached 1Password data must be encrypted and memory-only

### Migration and Compatibility
- Must be able to read existing config files from Go version
- Maintain the exact same CLI interface for drop-in replacement
- Preserve all command-line flags and their behaviors
- Output format should match for scripting compatibility

## Architecture Principles

### Separation of Concerns
1. **Configuration Layer**: Handles all TOML file operations
2. **Profile Management Layer**: Business logic for profile CRUD operations
3. **External Tool Layer**: Abstractions for git, op, ssh, gpg, gitsign, smimesign commands
4. **TUI Layer**: User interface components and navigation
5. **CLI Layer**: Command-line argument parsing and dispatch

### Dependency Injection
- Use traits for external tool wrappers to enable testing
- Configuration should be injected, not globally accessed
- TUI components should be decoupled from business logic

### Error Propagation
- Use Rust's Result type extensively
- Create custom error types for different failure modes
- Provide context with errors using error chaining
- Convert external tool errors into meaningful user messages
- Use tracing crate for structured logging

### Validation Requirements
- Profile schema validation dependent on keyType
- Duplicate profile names not allowed
- Paths must resolve to valid files or directories
- Configuration must not cause conflicts
- All imported configurations must pass validation

## Performance Considerations
- Config file parsing should be lazy where possible
- TUI should be responsive even with many profiles
- External tool calls should have appropriate timeouts
- 1Password vault listings cached in encrypted memory during session

## References to Original Implementation
When implementing in Rust, frequently refer back to:
- `/Users/analyst/dotfiles/git/commands/git-setup-go/internal/config/` - Configuration structure and loading
- `/Users/analyst/dotfiles/git/commands/git-setup-go/internal/profile/` - Profile management logic
- `/Users/analyst/dotfiles/git/commands/git-setup-go/internal/onepassword/` - 1Password CLI integration patterns
- `/Users/analyst/dotfiles/git/commands/git-setup-go/internal/tui/models/` - TUI component behaviors
- `/Users/analyst/dotfiles/git/commands/git-setup-go/internal/git/` - Git configuration manipulation
- `/Users/analyst/dotfiles/git/commands/git-setup-go/internal/agent/` - Agent.toml handling
- `/Users/analyst/dotfiles/git/commands/git-setup-go/internal/gpg/` - GPG operations

Remember: This is a TOOL, not a library. User experience and reliability are paramount.
