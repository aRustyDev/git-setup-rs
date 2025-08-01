# Phase 2: Profile Management - Visual Diagrams

## Profile Inheritance Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                    Profile Inheritance Tree                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                          base                                   │
│                      (company defaults)                         │
│                           │                                     │
│                ┌──────────┴──────────┐                         │
│                │                     │                         │
│              work                personal                      │
│         (work defaults)         (personal)                     │
│                │                                               │
│        ┌───────┼───────┐                                      │
│        │       │       │                                      │
│    client-a client-b project-x                                │
│                                                                 │
│  Inheritance Example:                                          │
│  ───────────────────                                           │
│                                                                 │
│  base.toml:               work.toml:                          │
│  ┌─────────────────┐      ┌─────────────────┐                │
│  │ [git]           │      │ extends = "base" │                │
│  │ company = "Inc" │      │ [git]           │                │
│  │ [signing]       │      │ email = "work@" │                │
│  │ method = "gpg"  │      └─────────────────┘                │
│  └─────────────────┘              │                           │
│          │                        │                           │
│          └────────┬───────────────┘                           │
│                   ▼                                            │
│         Merged Result:                                         │
│      ┌─────────────────┐                                      │
│      │ [git]           │                                      │
│      │ company = "Inc" │ (from base)                         │
│      │ email = "work@" │ (from work)                         │
│      │ [signing]       │                                      │
│      │ method = "gpg"  │ (from base)                         │
│      └─────────────────┘                                      │
└─────────────────────────────────────────────────────────────────┘
```

## Profile Storage Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                      Profile File System                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ~/.config/git-setup/                                          │
│  │                                                              │
│  ├── config.toml              Global Configuration             │
│  │   ├─ default_profile        "work"                         │
│  │   └─ theme                  "dark"                         │
│  │                                                              │
│  ├── profiles/                 Profile Storage                 │
│  │   ├── base.toml            ┌─────────────────┐            │
│  │   │                        │ name = "base"   │            │
│  │   │                        │ [git]           │            │
│  │   │                        │ company = "..."  │            │
│  │   │                        └─────────────────┘            │
│  │   │                                                         │
│  │   ├── work.toml            ┌─────────────────┐            │
│  │   │                        │ name = "work"   │            │
│  │   │                        │ extends = "base"│            │
│  │   │                        │ [git]           │            │
│  │   │                        │ email = "..."   │            │
│  │   │                        └─────────────────┘            │
│  │   │                                                         │
│  │   └── personal.toml        Independent Profile             │
│  │                                                              │
│  └── cache/                    Runtime Cache                   │
│      └── last_used.json        Quick Access                    │
│                                                                 │
│  Platform Paths:                                               │
│  ───────────────                                               │
│  • Linux/Mac: ~/.config/git-setup/                            │
│  • Windows:   C:\Users\Name\AppData\Roaming\git-setup\        │
└─────────────────────────────────────────────────────────────────┘
```

## Profile Manager Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     ProfileManager Flow                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  API Layer                     Implementation                   │
│  ─────────                     ──────────────                   │
│                                                                 │
│  list() ─────────┐            ┌─> FileSystemStore              │
│                  │            │   └─> Read directory           │
│  get(name) ──────┼────────────┤                                │
│                  │            │                                │
│  create(profile) ┤            ├─> Validator                    │
│                  │            │   └─> Check rules              │
│  update(profile) ┤            │                                │
│                  │            ├─> InheritanceResolver          │
│  delete(name) ───┤            │   └─> Merge parents            │
│                  │            │                                │
│  apply(name) ────┘            └─> GitConfigurator              │
│                                   └─> Set git config           │
│                                                                 │
│  State Management:                                              │
│  ─────────────────                                             │
│                                                                 │
│  ┌────────────┐  Load   ┌────────────┐  Resolve  ┌──────────┐ │
│  │   File     │ ──────> │  Profile   │ ───────> │ Complete │ │
│  │  Storage   │         │   Data     │          │ Profile  │ │
│  └────────────┘         └────────────┘          └──────────┘ │
│                                                        │        │
│                                                        ▼        │
│  ┌────────────┐  Write  ┌────────────┐  Apply   ┌──────────┐ │
│  │   File     │ <────── │   Cache    │ <────── │   Git    │ │
│  │  System    │         │  Updated   │         │  Config  │ │
│  └────────────┘         └────────────┘         └──────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Profile Application Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Profile Application Process                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Load Profile              2. Resolve Inheritance           │
│  ───────────────              ─────────────────────           │
│                                                                 │
│  work.toml                    work + base + defaults           │
│  ┌──────────┐                 ┌──────────────────┐            │
│  │ extends = │ ───────────────>│ Complete Profile │            │
│  │  "base"  │                 │ All fields filled│            │
│  └──────────┘                 └──────────────────┘            │
│                                        │                        │
│                                        ▼                        │
│  3. Apply to Git              4. Verify Applied                │
│  ───────────────              ────────────────                 │
│                                                                 │
│  Git Config Commands:          Read Back & Confirm:            │
│  • git config user.name       • git config --get user.name     │
│  • git config user.email      • git config --get user.email    │
│  • git config commit.gpgsign  • Matches profile values?        │
│                                                                 │
│  Scope Options:                                                │
│  ──────────────                                                │
│                                                                 │
│  --local  (default)            Current Repository Only         │
│     ↓                          .git/config                     │
│  ┌─────┐                                                       │
│  │ Repo │                                                      │
│  └─────┘                                                       │
│                                                                 │
│  --global                      All Repositories                │
│     ↓                          ~/.gitconfig                    │
│  ┌─────┬─────┬─────┐                                          │
│  │ All │ Git │ Repos│                                          │
│  └─────┴─────┴─────┘                                          │
└─────────────────────────────────────────────────────────────────┘
```

## Profile Validation Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                    Profile Validation Steps                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Input Profile                                                  │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────┐                                           │
│  │ 1. Name Check   │     Rules:                               │
│  │                 │     • Not empty                           │
│  │ "my-profile"    │     • Valid chars: a-z, 0-9, -, _        │
│  └────────┬────────┘     • Not reserved: "default", "all"     │
│           │ ✓                                                   │
│           ▼                                                     │
│  ┌─────────────────┐                                           │
│  │ 2. Email Valid  │     Rules:                               │
│  │                 │     • Valid format: *@*.*                 │
│  │ "user@example"  │     • Not empty                          │
│  └────────┬────────┘                                           │
│           │ ✓                                                   │
│           ▼                                                     │
│  ┌─────────────────┐                                           │
│  │ 3. Parent Check │     Rules:                               │
│  │                 │     • Parent exists                       │
│  │ extends="base"  │     • No circular dependencies            │
│  └────────┬────────┘     • Max depth: 5                       │
│           │ ✓                                                   │
│           ▼                                                     │
│  ┌─────────────────┐                                           │
│  │ 4. Fields Valid │     Rules:                               │
│  │                 │     • Known field names                   │
│  │ All settings    │     • Correct types                      │
│  └────────┬────────┘     • Valid values                       │
│           │ ✓                                                   │
│           ▼                                                     │
│      Valid Profile                                              │
│                                                                 │
│  Common Validation Errors:                                      │
│  ────────────────────────                                      │
│  ❌ "my profile" → Invalid name (space)                        │
│  ❌ "user@"      → Invalid email                               │
│  ❌ extends="xyz" → Parent not found                           │
│  ❌ gpgsign=yes  → Should be boolean                           │
└─────────────────────────────────────────────────────────────────┘
```

## Profile Format Examples

```
┌─────────────────────────────────────────────────────────────────┐
│                    Supported Profile Formats                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TOML Format (Recommended)          YAML Format                │
│  ─────────────────────────          ────────────               │
│                                                                 │
│  # work.toml                        # work.yaml                │
│  name = "work"                      name: work                 │
│  extends = "base"                   extends: base              │
│  description = """                  description: |             │
│  Work profile for                     Work profile for         │
│  company projects                     company projects         │
│  """                                                           │
│                                                                 │
│  [git]                              git:                       │
│  user_name = "John Doe"               user_name: John Doe     │
│  user_email = "john@company.com"      user_email: john@company│
│                                                                 │
│  [signing]                          signing:                   │
│  enabled = true                       enabled: true            │
│  method = "ssh"                       method: ssh              │
│  key = "~/.ssh/id_ed25519"           key: ~/.ssh/id_ed25519   │
│                                                                 │
│  JSON Format                                                    │
│  ───────────                                                    │
│                                                                 │
│  {                                                              │
│    "name": "work",                                             │
│    "extends": "base",                                          │
│    "git": {                                                    │
│      "user_name": "John Doe",                                  │
│      "user_email": "john@company.com"                         │
│    }                                                           │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
```

---

*These diagrams illustrate the profile management system's architecture and data flow.*