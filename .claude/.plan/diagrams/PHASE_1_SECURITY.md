# Phase 1: Security Foundation - Visual Diagrams

## Atomic File Operations Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Atomic Write Operation                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Step 1: Create Temp File          Step 2: Write Content       │
│  ────────────────────────          ─────────────────────       │
│                                                                 │
│  /home/user/.config/               /home/user/.config/         │
│  ├── git-setup/                    ├── git-setup/             │
│  │   ├── profile.toml              │   ├── profile.toml       │
│  │   └── .tmp.a3f2d1 (NEW)         │   └── .tmp.a3f2d1        │
│                                            └─> "new content"    │
│                                                                 │
│  Step 3: Sync to Disk              Step 4: Atomic Rename       │
│  ────────────────────              ─────────────────────       │
│                                                                 │
│  fsync(.tmp.a3f2d1)                rename(.tmp.a3f2d1,         │
│  └─> Ensures data on disk               profile.toml)          │
│                                     └─> Atomic operation!       │
│                                                                 │
│  Result: Either complete success or no change at all           │
└─────────────────────────────────────────────────────────────────┘
```

## Memory Safety with SensitiveString

```
┌─────────────────────────────────────────────────────────────────┐
│                 SensitiveString Lifecycle                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Creation                          Usage                        │
│  ────────                          ─────                        │
│                                                                 │
│  Password Input                    Operations                   │
│       │                                │                        │
│       ▼                                ▼                        │
│  ┌─────────────┐                 ┌─────────────┐              │
│  │ Allocate    │                 │   Access    │              │
│  │ SecureVec   │────────────────>│   via       │              │
│  │ (mlock)     │                 │   Deref     │              │
│  └─────────────┘                 └─────────────┘              │
│       │                                │                        │
│       │                                │                        │
│       ▼                                ▼                        │
│  Memory locked                    No accidental                │
│  (won't swap)                     exposure in:                 │
│                                   • Debug output               │
│                                   • Logs                       │
│                                   • Error messages             │
│                                                                 │
│  Destruction                      Security                     │
│  ───────────                      ────────                     │
│                                                                 │
│  Drop Implementation              Before: "MyS3cr3t!"          │
│       │                          After:  "\0\0\0\0\0\0\0"     │
│       ▼                                                        │
│  1. Overwrite with zeros                                       │
│  2. Release mlock                                              │
│  3. Deallocate                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Path Traversal Protection

```
┌─────────────────────────────────────────────────────────────────┐
│                  Path Validation Pipeline                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Input: "../../../etc/passwd"                             │
│                    │                                            │
│                    ▼                                            │
│  ┌─────────────────────────────┐                              │
│  │   1. Canonicalization        │                              │
│  │   path.canonicalize()        │ ──> Error: Outside bounds    │
│  └─────────────────────────────┘                              │
│                    │                                            │
│                    ▼                                            │
│  ┌─────────────────────────────┐                              │
│  │   2. Component Check         │                              │
│  │   No ".." allowed            │ ──> Rejected                 │
│  └─────────────────────────────┘                              │
│                    │                                            │
│                    ▼                                            │
│  ┌─────────────────────────────┐                              │
│  │   3. Prefix Validation       │                              │
│  │   Must start with base_dir   │ ──> Enforced                │
│  └─────────────────────────────┘                              │
│                    │                                            │
│                    ▼                                            │
│         Safe Path Only!                                         │
│                                                                 │
│  Example Results:                                              │
│  ───────────────                                               │
│  ❌ "../../../etc/passwd"     → Rejected                       │
│  ❌ "/etc/passwd"             → Outside base                   │
│  ❌ "profiles/../../../bad"   → Contains ..                    │
│  ✅ "profiles/work.toml"      → Accepted                       │
│  ✅ "config.toml"             → Accepted                       │
└─────────────────────────────────────────────────────────────────┘
```

## Concurrent Access Handling

```
┌─────────────────────────────────────────────────────────────────┐
│              Concurrent Write Protection                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Process A                    Process B                         │
│  ─────────                    ─────────                         │
│                                                                 │
│  T1: Read profile.toml        T2: Read profile.toml           │
│      name = "work"                name = "work"                │
│      email = "old@example"        email = "old@example"        │
│                                                                 │
│  T3: Modify in memory         T4: Modify in memory             │
│      email = "new@example"        theme = "dark"               │
│                                                                 │
│  T5: Write atomic                                              │
│      → .tmp.abc123                                             │
│      → fsync()                                                 │
│      → rename() ✓                                              │
│                               T6: Write atomic                 │
│                                   → .tmp.def456                │
│                                   → fsync()                    │
│                                   → rename() ✓                 │
│                                                                 │
│  Result with atomic ops:      Result without atomic:           │
│  ──────────────────────      ─────────────────────           │
│  Last write wins cleanly      Corrupted file possible         │
│  File always valid            Partial writes                   │
│  No data loss                 Data loss likely                │
└─────────────────────────────────────────────────────────────────┘
```

## Security Error Handling

```
┌─────────────────────────────────────────────────────────────────┐
│                 Security-Aware Error Messages                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Error Occurs                  Security Filter                  │
│  ────────────                  ───────────────                  │
│                                                                 │
│  Credential Error:             What NOT to show:               │
│  • Bad password               • The actual password            │
│  • Wrong API key              • The API key value              │
│  • Invalid token              • Token contents                 │
│                                                                 │
│  ┌────────────────┐           ┌────────────────────┐          │
│  │ Original Error │           │ Sanitized for User │          │
│  ├────────────────┤           ├────────────────────┤          │
│  │ InvalidPassword│ ────────> │ "Authentication    │          │
│  │ "MyS3cr3t!"    │           │  failed. Please    │          │
│  └────────────────┘           │  check credentials"│          │
│                               └────────────────────┘          │
│                                                                 │
│  Path Error:                   Safe Error:                     │
│  • /home/alice/...            • "Permission denied"            │
│  • C:\Users\Bob\...           • "File not found"               │
│                               • No absolute paths!             │
│                                                                 │
│  Best Practices:                                               │
│  ───────────────                                               │
│  ✅ Log detailed errors internally (securely)                  │
│  ✅ Show generic messages to users                             │
│  ✅ Provide error codes for support                            │
│  ❌ Never expose system paths                                  │
│  ❌ Never show credentials                                     │
└─────────────────────────────────────────────────────────────────┘
```

## Complete Security Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Security-First Operations                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Request                                                   │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │
│  │   Input     │────>│ Validation  │────>│   Sanitize  │     │
│  │   Layer     │     │   Layer     │     │    Layer    │     │
│  └─────────────┘     └─────────────┘     └─────────────┘     │
│                                                   │             │
│                                                   ▼             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │
│  │   Error     │<────│  Operation  │<────│   Authorize │     │
│  │  Handling   │     │   Layer     │     │    Layer    │     │
│  └─────────────┘     └─────────────┘     └─────────────┘     │
│         │                    │                                  │
│         │                    ▼                                  │
│         │            ┌─────────────┐                          │
│         │            │   Atomic    │                          │
│         │            │    Write    │                          │
│         │            └─────────────┘                          │
│         │                    │                                  │
│         └────────────────────┴──────> Secure Result           │
│                                                                 │
│  Every step designed to fail safely!                           │
└─────────────────────────────────────────────────────────────────┘
```

---

*These diagrams illustrate the security mechanisms that form the foundation of git-setup-rs.*