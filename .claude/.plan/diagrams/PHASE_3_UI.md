# Phase 3: User Interfaces - Visual Diagrams

## CLI Command Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                     CLI Command Hierarchy                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  git-setup <flags> <command> <subcommand> <args>               │
│      │        │        │          │         │                   │
│      │        │        │          │         └─> Arguments      │
│      │        │        │          └─> Subcommands              │
│      │        │        └─> Main commands                       │
│      │        └─> Global flags (--verbose, --quiet)            │
│      └─> Binary name                                           │
│                                                                 │
│  Command Tree:                                                  │
│  ─────────────                                                  │
│                                                                 │
│  git-setup                                                      │
│  ├── profile                    Profile Management              │
│  │   ├── list                   Show all profiles              │
│  │   ├── show <name>            Display profile details        │
│  │   ├── create                 Create new profile             │
│  │   ├── edit <name>            Modify existing profile        │
│  │   ├── delete <name>          Remove profile                 │
│  │   └── use <name>             Apply profile to repo          │
│  │                                                              │
│  ├── config                     Configuration                   │
│  │   ├── get <key>              Read config value              │
│  │   └── set <key> <value>      Write config value             │
│  │                                                              │
│  ├── health                     System Diagnostics             │
│  │   └── --all                  Show all checks                │
│  │                                                              │
│  └── tui                        Launch Interactive UI           │
└─────────────────────────────────────────────────────────────────┘
```

## TUI State Machine

```
┌─────────────────────────────────────────────────────────────────┐
│                       TUI State Transitions                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│     ┌──────────┐  'n'    ┌──────────┐  Tab    ┌──────────┐   │
│     │   Main   │ ──────> │  Create  │ ──────> │  Create  │   │
│     │  Screen  │         │  Wizard  │         │ (Step 2) │   │
│     └────┬─────┘         └──────────┘         └──────────┘   │
│          │ Enter                                      │        │
│          │                                            │        │
│          ▼                              Esc           ▼        │
│     ┌──────────┐  'e'    ┌──────────┐ <────── ┌──────────┐   │
│     │ Profile  │ ──────> │   Edit   │         │  Confirm │   │
│     │  View    │         │  Screen  │         │  Dialog  │   │
│     └────┬─────┘         └──────────┘         └──────────┘   │
│          │ 'd'                                                 │
│          │                                                     │
│          ▼                                                     │
│     ┌──────────┐  'y'    ┌──────────┐                        │
│     │  Delete  │ ──────> │ (Delete) │ ──> Back to Main       │
│     │ Confirm  │         │          │                        │
│     └──────────┘         └──────────┘                        │
│                                                                 │
│  State Properties:                                              │
│  ─────────────────                                             │
│  • Current Screen     • Input Buffer                           │
│  • Selected Index     • Status Message                         │
│  • Profile List       • Input Mode                            │
│  • Current Profile    • Should Quit                           │
└─────────────────────────────────────────────────────────────────┘
```

## TUI Layout Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                        TUI Layout Design                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                    Title Bar (Fixed)                     │  │
│  │  Git Setup v1.0.0              [q]uit [?]help          │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────┬───────────────────────────────────────┐  │
│  │                 │                                       │  │
│  │  Profile List   │         Content Area                 │  │
│  │                 │                                       │  │
│  │  > work        │  Profile: work                       │  │
│  │    personal    │  ─────────────                       │  │
│  │    client-a    │  Name:     John Doe                  │  │
│  │    client-b    │  Email:    john@company.com         │  │
│  │                 │  Signing:  SSH                       │  │
│  │  [n]ew         │  Extends:  base                      │  │
│  │  [d]elete      │                                       │  │
│  │  [/]search     │  Git Config:                         │  │
│  │                 │  • user.name = "John Doe"           │  │
│  │                 │  • user.email = "john@company.com"  │  │
│  │                 │  • commit.gpgsign = true            │  │
│  │                 │                                       │  │
│  └─────────────────┴───────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │ Status: Profile loaded successfully      │ 14:32:05     │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Layout Constraints:                                            │
│  ──────────────────                                            │
│  • Min terminal: 80x24                                         │
│  • List width: 20-30 chars                                     │
│  • Responsive to terminal resize                               │
└─────────────────────────────────────────────────────────────────┘
```

## Event Handling Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      Event Processing Pipeline                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Hardware Layer              Crossterm Layer                    │
│  ──────────────              ──────────────                    │
│                                                                 │
│  Keyboard ────┐              ┌─> KeyEvent                      │
│               │              │   • code: KeyCode               │
│               ├─> OS Event ──┤   • modifiers: KeyModifiers     │
│  Mouse ───────┘              │                                 │
│                              └─> MouseEvent                    │
│                                  • position: (x, y)            │
│                                  • button: MouseButton         │
│                                         │                       │
│                                         ▼                       │
│  Application Layer           Event Handler                      │
│  ─────────────────           ─────────────                      │
│                                                                 │
│  ┌──────────────┐           ┌──────────────┐                  │
│  │   Event      │ <──────── │   Crossterm  │                  │
│  │   Queue      │           │   Adapter    │                  │
│  └──────┬───────┘           └──────────────┘                  │
│         │                                                       │
│         ▼                                                       │
│  ┌──────────────┐           ┌──────────────┐                  │
│  │   Pattern    │ ────────> │    State     │                  │
│  │   Matching   │           │   Updates    │                  │
│  └──────────────┘           └──────┬───────┘                  │
│                                     │                           │
│                                     ▼                           │
│                              ┌──────────────┐                  │
│                              │   Render     │                  │
│                              │   Trigger    │                  │
│                              └──────────────┘                  │
│                                                                 │
│  Example Event Flow:                                            │
│  ──────────────────                                            │
│  User presses 'j' → OS → Crossterm → KeyEvent('j') →          │
│  → Match pattern → Update selected_index → Render new UI       │
└─────────────────────────────────────────────────────────────────┘
```

## Input Modes and Navigation

```
┌─────────────────────────────────────────────────────────────────┐
│                    Input Mode State Machine                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│     Normal Mode                    Insert Mode                  │
│     ───────────                    ───────────                  │
│                                                                 │
│     Navigation:          'i'       Text Input:                  │
│     • j/k  = up/down    ────>      • Type text                 │
│     • Enter = select               • Backspace                  │
│     • / = search mode              • Arrow keys                 │
│     • q = quit          <────      • Esc = back                 │
│                         Esc                                     │
│           │                                                     │
│           │ '/'                    Search Mode                  │
│           │                        ───────────                  │
│           └──────────────>         Filter list:                │
│                                    • Type query                 │
│                                    • Real-time                  │
│                          <────     • Esc = cancel               │
│                          Esc       • Enter = select             │
│                                                                 │
│  Visual Feedback:                                               │
│  ────────────────                                               │
│                                                                 │
│  Normal Mode:            Insert Mode:          Search Mode:     │
│  ┌─────────────┐        ┌─────────────┐      ┌─────────────┐  │
│  │ > Profile   │        │ Name: █     │      │ /work█      │  │
│  └─────────────┘        └─────────────┘      └─────────────┘  │
│   (selection)            (cursor)              (search)         │
└─────────────────────────────────────────────────────────────────┘
```

## Cross-Platform Considerations

```
┌─────────────────────────────────────────────────────────────────┐
│                 Cross-Platform Terminal Handling                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Windows Terminal           Unix/Linux Terminal                 │
│  ────────────────           ───────────────────                 │
│                                                                 │
│  Console API ──┐            TTY Interface ──┐                  │
│                │                            │                  │
│                ▼                            ▼                  │
│         ┌─────────────────────────────────────┐               │
│         │      Crossterm Abstraction          │               │
│         │  • Raw mode entry/exit              │               │
│         │  • Key event normalization          │               │
│         │  • Color support detection          │               │
│         │  • Unicode handling                 │               │
│         └─────────────────────────────────────┘               │
│                         │                                       │
│                         ▼                                       │
│                   Your TUI Code                                │
│                  (Same for all!)                               │
│                                                                 │
│  Platform Differences Handled:                                  │
│  ─────────────────────────────                                 │
│                                                                 │
│  Feature          Windows         Unix/Linux                   │
│  ────────         ───────         ──────────                   │
│  Clear Screen     cls             clear                        │
│  Colors           16/256          16/256/True                  │
│  Unicode          Code Page       UTF-8                        │
│  Line Endings     \r\n            \n                           │
│  Key Codes        Different       Standard                     │
│                                                                 │
│  Your Code Sees:  Same events on all platforms!               │
└─────────────────────────────────────────────────────────────────┘
```

## Error Display Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│                      Error Handling in UI                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Error Types                   Display Strategy                 │
│  ───────────                   ────────────────                 │
│                                                                 │
│  Recoverable:                  Status Bar Message               │
│  • File not found              ┌─────────────────────┐         │
│  • Invalid input               │ ⚠ File not found    │         │
│  • Network timeout             └─────────────────────┘         │
│                                                                 │
│  Critical:                     Modal Dialog                     │
│  • Disk full                   ┌─────────────────────┐         │
│  • Permission denied           │     Error!          │         │
│  • Corruption detected         │ Disk full. Cannot   │         │
│                                │ save profile.       │         │
│                                │                     │         │
│                                │ [OK]   [Quit]       │         │
│                                └─────────────────────┘         │
│                                                                 │
│  Fatal:                        Clean Exit                       │
│  • Panic recovery              1. Restore terminal              │
│  • Terminal lost               2. Print error to stderr         │
│  • Memory exhausted            3. Exit with code                │
│                                                                 │
│  Error Context Preservation:                                    │
│  ──────────────────────────                                    │
│                                                                 │
│  struct AppError {                                              │
│      kind: ErrorKind,      // What type                        │
│      message: String,      // User-friendly                    │
│      context: String,      // What we were doing               │
│      recovery: Vec<Action> // Possible fixes                   │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
```

---

*These diagrams illustrate the CLI and TUI architecture and implementation details.*