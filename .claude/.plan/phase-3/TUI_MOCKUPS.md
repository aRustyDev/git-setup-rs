# TUI Mockups and Visual Design Guide

## Overview

This document provides detailed mockups for all TUI screens in git-setup-rs, including layout specifications, color schemes, and interaction patterns.

## Design Principles

1. **Clarity**: Information hierarchy is clear
2. **Efficiency**: Common tasks require minimal keystrokes
3. **Consistency**: Similar elements behave similarly
4. **Accessibility**: Works with screen readers, high contrast modes
5. **Responsive**: Adapts to terminal size

## Color Palette

```
Primary Colors:
- Cyan:     #00CED1 - Headers, borders, focus
- Green:    #00FF00 - Success, active items
- Yellow:   #FFD700 - Warnings, search highlights
- Red:      #FF6B6B - Errors, destructive actions

Secondary Colors:
- White:    #FFFFFF - Primary text
- Gray:     #808080 - Secondary text, disabled
- DarkGray: #404040 - Borders, separators
- Blue:     #4169E1 - Links, references
```

## Main Screen Mockups

### 1. Main Menu (Profile List)

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────┐
│ Profiles                                          [/] Search  [?] Help       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ▸ work                        alice@company.com         ✓ Active          │
│    personal                    alice@personal.com        SSH signing       │
│    client-acme                 alice@acme.com           GPG signing       │
│    open-source                 alice.oss@gmail.com      No signing        │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] View  [n] New  [e] Edit  [d] Delete  [q] Quit       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Profile Details View

```
┌─ Profile: work ─────────────────────────────────────────────────────────────┐
│                                                              [←] Back       │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Git Configuration ─────────────────────────────────────────────────────┐ │
│ │ Name:     Alice Developer                                              │ │
│ │ Email:    alice@company.com                                            │ │
│ │ Editor:   nvim                                                         │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│ ┌─ Signing Configuration ─────────────────────────────────────────────────┐ │
│ │ Method:   SSH                                                          │ │
│ │ Key:      ~/.ssh/id_ed25519.pub                                       │ │
│ │ Format:   ssh                                                          │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│ ┌─ Detection Rules ───────────────────────────────────────────────────────┐ │
│ │ Path:     ~/work/**                                                    │ │
│ │ Remote:   github.com/company/*                                         │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ [e] Edit  [u] Use This Profile  [Esc] Back                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3. Profile Edit Screen

```
┌─ Edit Profile: work ────────────────────────────────────────────────────────┐
│                                                     [Ctrl+S] Save  [Esc] Cancel │
├─────────────────────────────────────────────────────────────────────────────┤
│ Git Configuration                                                           │
│ ─────────────────                                                          │
│ Name:     [Alice Developer                    ]                           │
│ Email:    [alice@company.com                  ]                           │
│ Editor:   [nvim                               ] [Tab] Next field          │
│                                                                             │
│ Signing Configuration                                                       │
│ ─────────────────────                                                      │
│ Method:   ( ) None  (•) SSH  ( ) GPG  ( ) X.509  ( ) Sigstore            │
│                                                                             │
│ SSH Key:  [~/.ssh/id_ed25519                  ] [b] Browse               │
│           ✓ Key exists and is valid                                        │
│                                                                             │
│ Detection Rules                                                             │
│ ───────────────                                                            │
│ Path Pattern:    [~/work/**                   ]                           │
│ Remote Pattern:  [github.com/company/*        ]                           │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Tab] Next Field  [Shift+Tab] Previous  [Ctrl+S] Save  [Esc] Cancel       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4. New Profile Wizard

```
┌─ Create New Profile ────────────────────────────────────────────────────────┐
│ Step 1 of 5: Basic Information                              [Esc] Cancel    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Profile Name                                                              │
│   ────────────                                                             │
│   This is how you'll identify this profile                                 │
│                                                                             │
│   Name: [client-bigcorp                       ]                           │
│         └─ Use lowercase with hyphens                                      │
│                                                                             │
│   Description (optional)                                                    │
│   ─────────────────────                                                   │
│   [Configuration for BigCorp client projects  ]                           │
│   [                                           ]                           │
│                                                                             │
│   ⚠️  Profile names cannot be changed after creation                        │
│                                                                             │
│                                                                             │
│                           ┌──────────┐ ┌────────┐                         │
│                           │ Previous │ │  Next  │                         │
│                           └──────────┘ └────────┘                         │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Progress: [■■□□□] Step 1 of 5                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5. Search Interface

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────┐
│ Search Profiles                                          [Esc] Close Search  │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ Search: work█                                                           │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│ Results (2 matches):                                                        │
│ ────────────────────                                                       │
│  ▸ work                        alice@company.com         Score: 100       │
│      ^^^^                                                                   │
│    client-workshop             alice@workshop.com        Score: 67        │
│             ^^^^                                                           │
│                                                                             │
│ Fuzzy search enabled - try: wrk, wok, wrkhp                               │
│                                                                             │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] Select  [Esc] Cancel                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6. Help Overlay

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────┐
│ Profiles                                                                    │
├─────────────────────────────────────────────────────────┐                  │
│                                                         │ Help - Main Menu │
│  ▸ work                        alice@company.com       ├──────────────────┤
│    personal                    alice@personal.com       │ Navigation       │
│    client-acme                 alice@acme.com          │ ──────────      │
│    open-source                 alice.oss@gmail.com     │ ↑/k  Previous   │
│                                                         │ ↓/j  Next       │
│                                                         │ g    First      │
│                                                         │ G    Last       │
│                                                         │                 │
│                                                         │ Actions         │
│                                                         │ ───────         │
│                                                         │ Enter View      │
│                                                         │ n     New       │
│                                                         │ e     Edit      │
│                                                         │ d     Delete    │
│                                                         │ u     Use       │
│                                                         │ /     Search    │
│                                                         │                 │
│                                                         │ Global          │
│                                                         │ ──────          │
│ [↑↓] Navigate  [Enter] View  [n] New  [e] Edit  [d] De │ F1/?  Help      │
└─────────────────────────────────────────────────────────┤ q     Quit      │
                                                          │ Esc   Close     │
                                                          └─────────────────┘
```

### 7. Confirmation Dialog

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────┐
│ Profiles                                                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ▸ work           ┌─ Confirm Delete ─────────────────────┐ ✓ Active       │
│    personal       │                                      │ SSH signing    │
│    client-acme    │ Delete profile "client-acme"?       │ GPG signing    │
│    open-source    │                                      │ No signing     │
│                   │ This action cannot be undone.       │                 │
│                   │                                      │                 │
│                   │        [Delete]  [Cancel]            │                 │
│                   │         (Enter)   (Esc)              │                 │
│                   └──────────────────────────────────────┘                 │
│                                                                             │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] View  [n] New  [e] Edit  [d] Delete  [q] Quit       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8. Loading/Progress States

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────┐
│ Syncing Profiles...                                      [Ctrl+C] Cancel    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│                     Synchronizing with remote...                            │
│                                                                             │
│                    ┌────────────────────────────┐                          │
│                    │████████████░░░░░░░░░░░░░░░│ 45%                      │
│                    └────────────────────────────┘                          │
│                                                                             │
│                    ⠼ Fetching profile: client-acme                         │
│                                                                             │
│                    Completed:                                               │
│                    ✓ work                                                   │
│                    ✓ personal                                               │
│                    ⠼ client-acme                                           │
│                    ⠀ open-source                                           │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Time elapsed: 0:03  Estimated remaining: 0:04                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9. Error State

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────┐
│ Profiles                                                  ⚠️ 2 Issues Found  │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Error ────────────────────────────────────────────────────────────────┐ │
│ │ ✗ Failed to load profile configuration                                 │ │
│ │                                                                         │ │
│ │ Details:                                                                │ │
│ │ Could not parse ~/.config/git-setup/profiles/work.toml                 │ │
│ │ Line 12: Expected string, found number                                 │ │
│ │                                                                         │ │
│ │ Suggestions:                                                            │ │
│ │ • Check the file syntax                                                │ │
│ │ • Restore from backup: work.toml.bak                                   │ │
│ │ • Run 'git-setup health' for diagnostics                              │ │
│ │                                                                         │ │
│ │                          [OK] (Enter)                                   │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│    personal                    alice@personal.com        SSH signing       │
│    client-acme                 alice@acme.com           GPG signing       │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] Acknowledge  [h] View Health Report                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 10. Multi-Select Mode

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────┐
│ Select Profiles to Export                           [Space] Toggle  [a] All │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  [✓] work                      alice@company.com         Selected         │
│  [ ] personal                  alice@personal.com                         │
│  [✓] client-acme              alice@acme.com            Selected         │
│  [ ] open-source              alice.oss@gmail.com                        │
│                                                                             │
│                                                                             │
│  Selected: 2 profiles                                                       │
│                                                                             │
│                                                                             │
│                          ┌──────────┐ ┌────────┐                          │
│                          │  Cancel  │ │ Export │                          │
│                          └──────────┘ └────────┘                          │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Space] Toggle  [a] Select All  [n] Select None  [Enter] Export           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Specifications

### 1. List Component

```rust
// Standard list with selection
List {
    style: ListStyle {
        normal_item: Style::default().fg(Color::White),
        selected_item: Style::default().fg(Color::Green).bg(Color::DarkGray),
        highlight_symbol: "▸ ",
        item_spacing: 0,
    }
}
```

### 2. Input Field

```rust
// Text input with validation
InputField {
    style: InputStyle {
        normal: Style::default().fg(Color::White),
        focused: Style::default().fg(Color::Yellow),
        error: Style::default().fg(Color::Red),
        placeholder: Style::default().fg(Color::DarkGray),
        cursor: CursorStyle::Block,
    }
}
```

### 3. Button

```rust
// Action buttons
Button {
    style: ButtonStyle {
        normal: Style::default().fg(Color::White),
        focused: Style::default().fg(Color::Black).bg(Color::Cyan),
        disabled: Style::default().fg(Color::DarkGray),
        destructive: Style::default().fg(Color::Red),
    }
}
```

### 4. Progress Bar

```rust
// Progress indication
ProgressBar {
    style: ProgressStyle {
        filled: '█',
        empty: '░',
        bar_style: Style::default().fg(Color::Green),
        percent_style: Style::default().fg(Color::White),
        message_style: Style::default().fg(Color::Gray),
    }
}
```

## Responsive Design

### Small Terminal (80x24)

```
┌─ Git Setup ──────────────────┐
│ Profiles         [/] [?]     │
├──────────────────────────────┤
│ ▸ work                       │
│   personal                   │
│   client-acme               │
│   open-source               │
│                             │
│                             │
├──────────────────────────────┤
│ [↑↓] Nav [Enter] View       │
└──────────────────────────────┘
```

### Medium Terminal (120x30)

Standard layouts as shown above.

### Large Terminal (160x50)

```
┌─ Git Setup ─────────────────────────────────────────────────────────────────────────────────┐
│ Profiles                                                      [/] Search  [?] Help           │
├────────────────────────────────────────────┬────────────────────────────────────────────────┤
│                                            │ Profile Details                                │
│  ▸ work              alice@company.com    ├────────────────────────────────────────────────┤
│    personal          alice@personal.com    │ Name:     Alice Developer                      │
│    client-acme       alice@acme.com       │ Email:    alice@company.com                    │
│    open-source       alice.oss@gmail.com  │ Signing:  SSH (ed25519)                        │
│                                            │                                                │
│                                            │ Detection Rules:                               │
│                                            │ • Path: ~/work/**                              │
│                                            │ • Remote: github.com/company/*                 │
│                                            │                                                │
│                                            │ Last Used: 2 hours ago                         │
│                                            │ Created:   2024-01-15                          │
└────────────────────────────────────────────┴────────────────────────────────────────────────┘
```

## Animation Specifications

### 1. Loading Spinner

```
Frames: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
Speed: 80ms per frame
```

### 2. Progress Indicators

```
Styles:
- Dots:    ["   ", ".  ", ".. ", "..."]
- Bars:    ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"]
- Pulse:   ["◯", "◉", "●", "◉"]
```

### 3. Transitions

```
Screen transitions: 150ms fade
Modal appearance: 100ms slide-in
Error shake: 50ms × 3
```

## Accessibility Features

### 1. Screen Reader Support

```
Role announcements:
- "Profile list, 4 items"
- "Selected: work profile"
- "Text input: Profile name"
- "Button: Save changes"
```

### 2. High Contrast Mode

```
When terminal reports high contrast:
- Increase border brightness
- Use bold for important text
- Avoid color-only indicators
- Add text indicators (*, !, ?)
```

### 3. Keyboard Navigation

```
Tab order:
1. Main content area
2. Action buttons
3. Navigation hints
4. Status bar

Focus indicators:
- Visible focus ring
- High contrast selection
- Clear active element
```

## Implementation Notes

1. **Layout Engine**: Use constraint-based layout for responsive design
2. **Widget Reuse**: Cache static widgets to improve performance
3. **Partial Rendering**: Only redraw changed areas
4. **Event Handling**: Implement proper focus management
5. **Error States**: Every interactive element needs error handling
6. **Loading States**: Show progress for operations >100ms
7. **Undo Support**: Critical operations should be undoable

## Testing Considerations

1. Test with various terminal sizes (80x24 minimum)
2. Test with different color schemes
3. Test keyboard-only navigation
4. Test with screen readers (when possible)
5. Test over slow SSH connections
6. Test with Unicode and emoji support disabled
7. Test rapid key input handling

This comprehensive mockup guide ensures consistent, accessible, and beautiful TUI implementation across all screens in git-setup-rs.