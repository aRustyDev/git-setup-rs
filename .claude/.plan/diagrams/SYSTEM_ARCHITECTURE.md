# Git-Setup-RS System Architecture Diagrams

## Overall System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           git-setup-rs                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐ │
│  │   Phase 1   │  │   Phase 2   │  │   Phase 3   │  │   Phase 4    │ │
│  │   Security  │  │   Profiles  │  │     UI      │  │  1Password   │ │
│  │ Foundation  │  │ Management  │  │  CLI & TUI  │  │ Integration  │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬───────┘ │
│         │                 │                 │                 │         │
│  ┌──────┴─────────────────┴─────────────────┴─────────────────┴──────┐ │
│  │                        Core Platform Layer                         │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐  │ │
│  │  │   Atomic   │  │   Memory   │  │    Path    │  │   Error    │  │ │
│  │  │Operations  │  │   Safety   │  │Abstraction │  │  Handling  │  │ │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                         Phase 5: Features                        │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │  │
│  │  │    5A    │  │    5B    │  │    5C    │  │      5D      │  │  │
│  │  │ Pattern  │  │  Health  │  │ Signing  │  │   Advanced   │  │  │
│  │  │Matching  │  │Monitoring│  │ Methods  │  │   Features   │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │              Phase 6: Platform & Distribution                    │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────┐  │  │
│  │  │  Windows   │  │   macOS    │  │   Linux    │  │   Dist   │  │  │
│  │  │  Support   │  │  Universal │  │  Packages  │  │ Pipeline │  │  │
│  │  └────────────┘  └────────────┘  └────────────┘  └──────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Data Flow Architecture

```
User Input                     Processing                      Output
──────────                     ──────────                     ──────

CLI/TUI ──┐                 ┌─> Profile Manager ──┐        ┌─> Git Config
          │                 │                      │        │
          ├─> Command ──────┤                      ├────────┤
          │   Parser        │                      │        │
Keyboard ─┘                 └─> Security Layer ───┘        └─> Terminal
                                      │
                                      │
                               ┌──────┴──────┐
                               │  1Password  │
                               │   (Phase 4) │
                               └─────────────┘
```

## Security Architecture (Phase 1)

```
┌─────────────────────────────────────────────────────────────────┐
│                     Security Foundation                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Operations                    Security Layer              │
│  ───────────────                    ──────────────              │
│                                                                 │
│  Write Config ─────┐           ┌─> Atomic Operations           │
│                    │           │   └─> tempfile + rename       │
│                    ├───────────┤                                │
│  Store Credential ─┤           ├─> Memory Protection           │
│                    │           │   └─> zeroize on drop         │
│                    │           │                                │
│  Read Profile ─────┘           └─> Path Validation             │
│                                    └─> no traversal attacks     │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Threat Model                            │ │
│  │  • Concurrent access → Atomic operations                  │ │
│  │  • Memory dumps → Zeroization                            │ │
│  │  • Path injection → Validation                           │ │
│  │  • Credential leaks → SensitiveString                    │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Profile System Architecture (Phase 2)

```
┌─────────────────────────────────────────────────────────────────┐
│                      Profile Management                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Profile Storage           Profile Hierarchy                   │
│  ───────────────           ─────────────────                   │
│                                                                 │
│  ~/.config/git-setup/      base                               │
│  ├── profiles/             ├── work (extends: base)           │
│  │   ├── base.toml         │   └── client-a (extends: work)  │
│  │   ├── work.toml         └── personal                       │
│  │   ├── personal.toml                                        │
│  │   └── client-a.toml     Inheritance Flow:                  │
│  └── config.toml           base → work → client-a             │
│                                                                 │
│  Profile Structure                                              │
│  ─────────────────                                             │
│  ┌─────────────────┐                                           │
│  │     Profile     │                                           │
│  ├─────────────────┤                                           │
│  │ name: String    │                                           │
│  │ extends: Option │───┐ Inheritance                           │
│  │ git: GitConfig  │   │                                       │
│  │ signing: Option │   └─> Parent Profile                      │
│  │ detection: Rules│                                           │
│  └─────────────────┘                                           │
└─────────────────────────────────────────────────────────────────┘
```

## UI Architecture (Phase 3)

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interfaces                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Command Line Interface              Terminal UI                │
│  ──────────────────────              ───────────                │
│                                                                 │
│  $ git-setup <command>               ┌─ Git Setup ─────────┐   │
│       │                              │ > work              │   │
│       ├─> Parser (clap)              │   personal          │   │
│       │                              │   client-a          │   │
│       ├─> Command Handler            └─────────────────────┘   │
│       │                                      │                  │
│       └─> Output Formatter                   ├─> Event Loop    │
│           ├─> Text                           │                  │
│           ├─> JSON                           ├─> State Machine │
│           └─> YAML                           │                  │
│                                              └─> Renderer       │
│                                                  (ratatui)      │
│                                                                 │
│  Event Flow in TUI:                                            │
│  ─────────────────                                             │
│  Keyboard ─> crossterm ─> Event Handler ─> App State ─> UI    │
│                               │                                 │
│                               └─> Action ─> ProfileManager     │
└─────────────────────────────────────────────────────────────────┘
```

## 1Password Integration Architecture (Phase 4)

```
┌─────────────────────────────────────────────────────────────────┐
│                    1Password Integration                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  git-setup-rs                    1Password CLI                 │
│  ────────────                    ─────────────                 │
│                                                                 │
│  ┌──────────────┐  subprocess   ┌──────────────┐              │
│  │              │───────────────>│     op       │              │
│  │ OpCli Wrapper│                │  (external)  │              │
│  │              │<───────────────│              │              │
│  └──────────────┘   JSON output  └──────────────┘              │
│         │                                │                      │
│         │                                │                      │
│    Never stores                    Holds actual                │
│    credentials!                    credentials                  │
│         │                                                       │
│         └─> Only stores references:                            │
│             "op://vault/item/field"                            │
│                                                                 │
│  Security Flow:                                                 │
│  ──────────────                                                │
│  1. User authenticates to 1Password (biometric/password)       │
│  2. git-setup requests credential list                         │
│  3. User selects credential                                    │
│  4. git-setup stores reference only                           │
│  5. At use time, op fetches actual value                      │
└─────────────────────────────────────────────────────────────────┘
```

## Feature Architecture (Phase 5)

```
┌─────────────────────────────────────────────────────────────────┐
│                     Advanced Features                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  5A: Pattern Matching          5B: Health Monitoring           │
│  ────────────────────          ─────────────────────           │
│                                                                 │
│  Repo Path                     System State                    │
│      │                              │                           │
│      ├─> Rule Engine               ├─> Git Version Check       │
│      │   ├─> Path Rules            ├─> SSH Key Validation      │
│      │   ├─> Remote Rules          ├─> GPG Key Check           │
│      │   └─> File Rules            └─> Config Verification     │
│      │                                                          │
│      └─> Profile Selection         Report Generation           │
│                                                                 │
│  5C: Signing Methods           5D: Advanced Features           │
│  ───────────────────           ────────────────────           │
│                                                                 │
│  ┌─────────┐  ┌─────────┐     ┌──────────┐  ┌────────────┐  │
│  │   SSH   │  │   GPG   │     │   x509   │  │  Sigstore  │  │
│  │  Keys   │  │  Keys   │     │  Certs   │  │  Keyless   │  │
│  └────┬────┘  └────┬────┘     └────┬─────┘  └─────┬──────┘  │
│       │            │                │               │          │
│       └────────────┴────────────────┴───────────────┘          │
│                           │                                     │
│                    Git Configuration                            │
└─────────────────────────────────────────────────────────────────┘
```

## Platform & Distribution Architecture (Phase 6)

```
┌─────────────────────────────────────────────────────────────────┐
│                  Platform & Distribution                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Platform Abstraction          Build Pipeline                  │
│  ────────────────────          ──────────────                  │
│                                                                 │
│  ┌─────────────────┐           GitHub Push                     │
│  │  PlatformOps    │                │                          │
│  │     trait       │                ├─> CI Triggered           │
│  └────────┬────────┘                │                          │
│           │                         ├─> Multi-platform Build   │
│   ┌───────┼───────┐                │   ├─> Windows            │
│   │       │       │                │   ├─> macOS (Universal)  │
│   ▼       ▼       ▼                │   └─> Linux              │
│ Windows  macOS  Linux              │                          │
│   │       │       │                ├─> cargo-dist Package     │
│   │       │       │                │                          │
│   └───────┴───────┘                └─> GitHub Release         │
│           │                                   │                │
│    Unified API                         Installers:             │
│                                        • shell script          │
│                                        • PowerShell            │
│                                        • Homebrew              │
│                                                                 │
│  Performance Pipeline:                                          │
│  ────────────────────                                          │
│  Code → Profile → Identify Hotspots → Optimize → Benchmark     │
└─────────────────────────────────────────────────────────────────┘
```

## Complete System Integration

```
                            git-setup-rs
┌───────────────────────────────────────────────────────────────┐
│                                                               │
│  User ─────┐                                      ┌─> Git     │
│            │                                      │   Config  │
│            ▼                                      │           │
│     ┌─────────────┐     ┌─────────────┐         │           │
│     │   CLI/TUI   │────>│   Profile   │─────────┤           │
│     │  (Phase 3)  │     │  (Phase 2)  │         │           │
│     └─────────────┘     └─────────────┘         │           │
│            │                    │                 │           │
│            │                    │                 └─> Signing │
│            ▼                    ▼                     Keys    │
│     ┌─────────────┐     ┌─────────────┐                     │
│     │  Commands   │     │  1Password  │                     │
│     │  & Events   │     │  (Phase 4)  │                     │
│     └─────────────┘     └─────────────┘                     │
│            │                    │                             │
│            ▼                    ▼                             │
│     ┌───────────────────────────────────┐                   │
│     │    Security Layer (Phase 1)       │                   │
│     │  • Atomic Ops  • Memory Safety    │                   │
│     │  • Path Safety • Secure Storage   │                   │
│     └───────────────────────────────────┘                   │
│                         │                                     │
│                         ▼                                     │
│     ┌───────────────────────────────────┐                   │
│     │   Platform Layer (Phase 6)        │                   │
│     │  • OS Abstraction • Distribution  │                   │
│     └───────────────────────────────────┘                   │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

## Development Flow

```
Phase 1: Foundation ──┐
                      ├─> Phase 2: Profiles ──┐
                      │                       ├─> Phase 3: UI ──┐
                      │                       │                 │
                      │                       │                 ├─> Phase 5
                      │                       │                 │   Features
                      │                       └─> Phase 4: ─────┘      │
                      │                           1Password            │
                      │                                               │
                      └───────────────────────────────────────────────┴─> Phase 6
                                                                          Platform
```

---

*These diagrams provide visual understanding of the system architecture and how all phases integrate together.*