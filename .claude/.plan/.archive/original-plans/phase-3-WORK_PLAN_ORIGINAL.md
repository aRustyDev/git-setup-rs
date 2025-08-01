# Phase 3: User Interfaces - Work Plan

## Prerequisites

Phase 3 builds upon the profile management system from Phase 2.

**Required from Previous Phases**:
- Profile Manager API (Phase 2)
- Git configuration engine (Phase 2)
- Secure file operations (Phase 1)
- Configuration system (Phase 1)

**Required Knowledge**:
- **Rust TUI Development**: Ratatui, event loops, state management (*critical*)
- **CLI Design**: Clap v4, subcommands, argument parsing (*critical*)
- **Terminal Programming**: Cross-platform terminal handling (*required*)
- **Async Rust**: Tokio basics for event handling (*required*)

💡 **Junior Dev Resources**:
- 📚 [Ratatui Book](https://ratatui.rs/introduction.html) - Complete TUI tutorial
- 🎥 [Building TUIs in Rust](https://www.youtube.com/watch?v=ZYjxPJuIhLs) - 45 min tutorial
- 📖 [Clap Tutorial](https://docs.rs/clap/latest/clap/_tutorial/index.html) - Official guide
- 🔧 [TUI Examples](https://github.com/ratatui-org/ratatui/tree/main/examples) - Working code
- 💻 [Async Rust Book](https://rust-lang.github.io/async-book/) - Chapters 1-4

## Quick Reference - Essential Resources

### TUI/CLI Documentation
- [Ratatui Book](https://ratatui.rs) - Complete TUI guide
- [Ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [Clap Documentation](https://docs.rs/clap/4) - CLI framework
- [Crossterm](https://docs.rs/crossterm) - Terminal manipulation

### Project Resources
- **[SPEC.md](../../spec/SPEC.md)** - See FR-005 Terminal User Interface
- **[Phase 2 API](../phase-2/)** - ProfileManager trait and types
- **[UI Mockups](../../design/tui-mockups/)** - TUI screen designs

### Commands
- `cargo run -- help` - Test CLI interface
- `cargo run -- tui` - Launch TUI
- `cargo test ui::` - Run UI tests
- `script -q /dev/null cargo run -- tui` - TUI in CI environment

## Overview

Phase 3 delivers the user-facing interfaces for git-setup-rs, providing both a powerful CLI for automation and an intuitive TUI for interactive use. This phase transforms the backend capabilities into accessible tools.

**Key Deliverables**:
- Complete CLI with all profile operations
- Full-featured TUI with profile wizard
- Fuzzy search integration with Nucleo
- Context-sensitive help system
- Comprehensive keyboard navigation

**Checkpoint Strategy**: 5 checkpoints for UI components

**Time Estimate**: 3 weeks (120 hours)

## Development Methodology: Test-Driven Development (TDD)

UI testing requires special approaches:
1. **CLI Testing** - Test command parsing and output
2. **TUI Testing** - Test state machines and rendering
3. **Integration Testing** - Test with real ProfileManager
4. **Manual Testing** - Terminal compatibility matrix

## Done Criteria Checklist

Phase 3 is complete when:
- [ ] CLI supports all profile operations
- [ ] TUI launches in <100ms
- [ ] Profile wizard guides through 6 steps
- [ ] Fuzzy search responds in <10ms/keystroke
- [ ] Vim keybindings work throughout
- [ ] Help accessible from any screen
- [ ] Works on macOS, Linux, Windows terminals
- [ ] Test coverage ≥80% (excluding UI rendering)
- [ ] All 5 checkpoints reviewed and approved

## Work Breakdown with Review Checkpoints

### 3.1 CLI Implementation (25 hours)

**Complexity**: Medium - Well-defined patterns
**Files**: `src/cli/mod.rs`, `src/cli/commands/*.rs`, `src/cli/output.rs`

#### Task 3.1.1: CLI Architecture Design (5 hours)

💡 **Junior Dev Concept**: CLI Design with Clap
**What it is**: Clap is a command-line argument parser that generates help text and handles parsing
**Why Clap v4**: Derive macros make it feel like magic - you write structs, get a CLI
**Key pattern**: Subcommands are enums, arguments are struct fields

Design the command structure using Clap v4:

```rust
#[derive(Parser)]
#[command(name = "git-setup")]
#[command(about = "Secure Git profile management")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(global = true, long)]
    pub config: Option<PathBuf>,
    
    #[arg(global = true, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all profiles
    List {
        #[arg(long)]
        format: Option<OutputFormat>,
    },
    /// Switch to a profile
    Switch {
        /// Profile name or fuzzy search
        profile: String,
        #[arg(long)]
        global: bool,
    },
    /// Create a new profile
    New {
        /// Profile name
        name: String,
        #[arg(long)]
        template: Option<String>,
    },
    /// Launch interactive TUI
    Tui,
    /// Health check
    Doctor,
}
```

#### Task 3.1.2: Command Implementation (12 hours)

Implement each command with proper error handling:

**Key Commands**:
- `list` - Table, JSON, YAML output formats
- `switch` - With confirmation and rollback
- `new` - Interactive or from template
- `edit` - Open in $EDITOR
- `delete` - With confirmation
- `import` - From file or 1Password
- `export` - Single or all profiles
- `validate` - Check profile validity
- `doctor` - System health check

**Output Formatting**:
- Colored output with `termcolor`
- Progress bars for long operations
- Structured errors with hints

**Step-by-Step Implementation**:

1. **Start with the simplest command** (2 hours)
   ```rust
   use clap::{Parser, Subcommand};
   use crate::profile::ProfileManager;
   
   /// List all profiles
   pub async fn cmd_list(manager: &ProfileManager, format: OutputFormat) -> Result<()> {
       let profiles = manager.list().await?;
       
       match format {
           OutputFormat::Table => print_table(&profiles),
           OutputFormat::Json => {
               let json = serde_json::to_string_pretty(&profiles)?;
               println!("{}", json);
           }
           OutputFormat::Yaml => {
               let yaml = serde_yaml::to_string(&profiles)?;
               println!("{}", yaml);
           }
       }
       
       Ok(())
   }
   ```
   
   💡 **Start Simple**: Get one command fully working before adding complexity

2. **Add user interaction for dangerous operations** (3 hours)
   ```rust
   pub async fn cmd_delete(manager: &mut ProfileManager, name: &str, force: bool) -> Result<()> {
       // Check profile exists
       let profile = manager.read(name).await
           .map_err(|_| CliError::ProfileNotFound(name.to_string()))?;
       
       // Confirm unless --force
       if !force {
           println!("Delete profile '{}'?", name);
           println!("  Email: {}", profile.user.email);
           print!("Type 'yes' to confirm: ");
           
           std::io::stdout().flush()?;
           let mut input = String::new();
           std::io::stdin().read_line(&mut input)?;
           
           if input.trim() != "yes" {
               println!("Cancelled.");
               return Ok(());
           }
       }
       
       manager.delete(name).await?;
       println!("✅ Profile '{}' deleted", name);
       Ok(())
   }
   ```
   
   ⚠️ **Common Mistake**: Forgetting to flush stdout before reading input
   ✅ **Always**: Flush when mixing print! and input

3. **Implement progress feedback** (3 hours)
   ```rust
   use indicatif::{ProgressBar, ProgressStyle};
   
   pub async fn cmd_import(manager: &mut ProfileManager, path: &Path) -> Result<()> {
       let pb = ProgressBar::new_spinner();
       pb.set_style(
           ProgressStyle::default_spinner()
               .template("{spinner:.green} {msg}")
               .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
       );
       
       pb.set_message("Reading file...");
       let content = tokio::fs::read_to_string(path).await?;
       
       pb.set_message("Parsing profile...");
       let profile = Profile::from_toml(&content)?;
       
       pb.set_message("Validating...");
       profile.validate()?;
       
       pb.set_message("Saving...");
       manager.create(profile).await?;
       
       pb.finish_with_message("✅ Import complete");
       Ok(())
   }
   ```

4. **Handle errors gracefully** (4 hours)
   ```rust
   pub fn print_error(error: &CliError) {
       use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
       
       let mut stderr = StandardStream::stderr(ColorChoice::Always);
       
       // Red "Error:" prefix
       stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
       eprint!("Error: ");
       stderr.reset().ok();
       
       // Error message
       eprintln!("{}", error);
       
       // Helpful hint in dim text
       if let Some(hint) = error.hint() {
           stderr.set_color(ColorSpec::new().set_dimmed(true)).ok();
           eprintln!("\nHint: {}", hint);
           stderr.reset().ok();
       }
   }
   ```

#### Task 3.1.3: CLI Testing Framework (6 hours)

Build comprehensive CLI tests:
- Command parsing tests
- Output format verification
- Error message validation
- Integration with ProfileManager mock

#### Task 3.1.4: Shell Completions (2 hours)

Generate completions for common shells:
- Bash completion
- Zsh completion
- Fish completion
- PowerShell completion

---

## 🛑 CHECKPOINT 1: CLI Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** past this checkpoint without completing the review process.

### Pre-Checkpoint Checklist

- [ ] All CLI commands implemented and tested
- [ ] Help text reviewed for clarity and completeness
- [ ] Error messages are helpful with recovery hints
- [ ] Output formats (table, JSON, YAML) working
- [ ] Shell completions generated for all shells
- [ ] Integration tests with real ProfileManager
- [ ] Performance: Commands respond in <100ms

### CLI Testing Commands

```bash
# Test all commands
./scripts/test-all-cli-commands.sh

# Verify help text
cargo run -- --help
cargo run -- help switch

# Test output formats
cargo run -- list --format json | jq .
cargo run -- list --format yaml | yq .

# Test error handling
cargo run -- switch nonexistent-profile
```

### Review Requirements

#### Usability Review (UX Designer)
- [ ] Commands follow Unix conventions
- [ ] Help text is clear for new users
- [ ] Error messages guide to solution
- [ ] Progress feedback for long operations

#### Technical Review (Tech Lead)
- [ ] Consistent error handling
- [ ] Proper async/await usage
- [ ] No blocking I/O in async contexts
- [ ] Memory efficient for large profile lists

### Consequences of Skipping

- Poor CLI UX affects all users
- Inconsistent commands confuse users
- Missing error handling frustrates debugging
- Performance issues compound in scripts

---

### 3.2 TUI Framework Setup (20 hours)

**Complexity**: High - Complex state management
**Files**: `src/tui/mod.rs`, `src/tui/app.rs`, `src/tui/state.rs`

#### Task 3.2.1: TUI Application Architecture (6 hours)

💡 **Junior Dev Concept**: TUI Architecture
**What it is**: Terminal User Interfaces are like GUIs but run in the terminal
**Key concepts**: 
- Event loop: Check for key presses, update state, redraw screen, repeat
- State management: Track what screen we're on, what's selected, etc.
- Rendering: Draw UI elements at specific terminal coordinates

Design the TUI application structure:

```rust
pub struct App {
    pub state: AppState,
    pub profile_manager: Arc<Mutex<dyn ProfileManager>>,
    pub input_mode: InputMode,
    pub messages: Vec<Message>,
}

pub enum AppState {
    ProfileList(ProfileListState),
    ProfileDetail(ProfileDetailState),
    ProfileWizard(WizardState),
    Settings(SettingsState),
    Help(HelpState),
}

pub enum InputMode {
    Normal,
    Insert,
    Search,
}

pub struct WizardState {
    pub step: WizardStep,
    pub profile: ProfileBuilder,
    pub validation_errors: Vec<ValidationError>,
}
```

#### Task 3.2.2: Event Handling System (8 hours)

💡 **Junior Dev Concept**: Event-Driven Programming
**What it is**: Your code responds to events (key presses, mouse clicks) rather than running sequentially
**Why in TUIs**: Users can press any key at any time - we need to handle it gracefully
**Key pattern**: Big match statement that routes events to handlers

Implement robust event handling:

```rust
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

impl App {
    pub fn handle_event(&mut self, event: Event) -> Result<Action> {
        match self.state {
            AppState::ProfileList(_) => self.handle_list_event(event),
            AppState::ProfileWizard(_) => self.handle_wizard_event(event),
            // ...
        }
    }
}
```

**Key Requirements**:
- Non-blocking event loop
- Graceful error handling
- State persistence across mode changes
- Undo/redo support for critical operations

**Step-by-Step Implementation**:

1. **Set up the event system** (2 hours)
   ```rust
   use crossterm::event::{self, Event as CEvent, KeyCode};
   use std::time::Duration;
   
   pub struct EventHandler {
       rx: mpsc::Receiver<Event>,
       tick_rate: Duration,
   }
   
   impl EventHandler {
       pub fn new(tick_rate: Duration) -> (Self, mpsc::Sender<Event>) {
           let (tx, rx) = mpsc::channel();
           let tx_clone = tx.clone();
           
           // Spawn event collection thread
           thread::spawn(move || {
               loop {
                   // Poll for events
                   if event::poll(tick_rate).unwrap() {
                       if let CEvent::Key(key) = event::read().unwrap() {
                           tx_clone.send(Event::Key(key)).unwrap();
                       }
                   } else {
                       // Send tick for animations/timers
                       tx_clone.send(Event::Tick).unwrap();
                   }
               }
           });
           
           (Self { rx, tick_rate }, tx)
       }
   }
   ```
   
   ⚠️ **Common Bug**: Blocking on event::read() freezes the UI
   ✅ **Solution**: Always poll first with a timeout

2. **Implement state-specific handlers** (3 hours)
   ```rust
   impl App {
       fn handle_list_event(&mut self, event: Event) -> Result<Action> {
           match event {
               Event::Key(key) => match key.code {
                   KeyCode::Char('q') => return Ok(Action::Quit),
                   KeyCode::Char('n') => {
                       self.state = AppState::ProfileWizard(WizardState::new());
                   }
                   KeyCode::Up | KeyCode::Char('k') => {
                       self.profile_list.previous();
                   }
                   KeyCode::Down | KeyCode::Char('j') => {
                       self.profile_list.next();
                   }
                   KeyCode::Enter => {
                       if let Some(profile) = self.profile_list.selected() {
                           return Ok(Action::SwitchProfile(profile));
                       }
                   }
                   _ => {}
               },
               Event::Tick => {
                   // Update any animations or time-based UI
               }
               _ => {}
           }
           Ok(Action::None)
       }
   }
   ```
   
   💡 **Vim Users**: Support both arrows and hjkl navigation

3. **Add error recovery** (2 hours)
   ```rust
   pub fn handle_event_safe(&mut self, event: Event) -> Action {
       match self.handle_event(event) {
           Ok(action) => action,
           Err(e) => {
               // Show error in status bar
               self.set_error_message(format!("Error: {}", e));
               Action::None
           }
       }
   }
   ```

4. **Implement undo/redo** (1 hour)
   ```rust
   pub struct UndoStack<T> {
       past: Vec<T>,
       future: Vec<T>,
       limit: usize,
   }
   
   impl<T: Clone> UndoStack<T> {
       pub fn push(&mut self, state: T) {
           self.past.push(state);
           self.future.clear();  // New action clears redo
           
           if self.past.len() > self.limit {
               self.past.remove(0);
           }
       }
       
       pub fn undo(&mut self, current: T) -> Option<T> {
           if let Some(past) = self.past.pop() {
               self.future.push(current);
               Some(past)
           } else {
               None
           }
       }
   }
   ```

#### Task 3.2.3: Terminal Setup & Restoration (4 hours)

💡 **Junior Dev Concept**: Terminal Raw Mode
**Normal mode**: Terminal processes input (Ctrl+C, Enter, etc.)
**Raw mode**: Your app gets every keystroke directly
**Critical**: MUST restore terminal on exit or terminal stays broken!

Handle terminal lifecycle properly:
- Enable raw mode
- Alternative screen buffer
- Mouse support (optional)
- Proper cleanup on panic
- Save/restore terminal state

**Step-by-Step Implementation**:

1. **Create safe terminal wrapper** (2 hours)
   ```rust
   use crossterm::{
       terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
       execute,
       cursor,
   };
   use std::panic;
   
   pub struct Terminal {
       backend: CrosstermBackend<Stdout>,
   }
   
   impl Terminal {
       pub fn new() -> Result<Self> {
           // Save current panic hook
           let original_hook = panic::take_hook();
           
           // Set panic hook to restore terminal
           panic::set_hook(Box::new(move |panic_info| {
               // Restore terminal before printing panic
               Self::restore_terminal().ok();
               original_hook(panic_info);
           }));
           
           // Setup terminal
           terminal::enable_raw_mode()?;
           let mut stdout = io::stdout();
           execute!(
               stdout,
               EnterAlternateScreen,
               cursor::Hide
           )?;
           
           Ok(Self {
               backend: CrosstermBackend::new(stdout),
           })
       }
       
       fn restore_terminal() -> Result<()> {
           terminal::disable_raw_mode()?;
           execute!(
               io::stdout(),
               LeaveAlternateScreen,
               cursor::Show
           )?;
           Ok(())
       }
   }
   
   impl Drop for Terminal {
       fn drop(&mut self) {
           Self::restore_terminal().ok();
       }
   }
   ```
   
   ⚠️ **Critical**: Panic hook ensures terminal restored even on crash

2. **Add mouse support conditionally** (1 hour)
   ```rust
   pub fn enable_mouse(&mut self) -> Result<()> {
       execute!(self.backend, event::EnableMouseCapture)?;
       Ok(())
   }
   
   pub fn disable_mouse(&mut self) -> Result<()> {
       execute!(self.backend, event::DisableMouseCapture)?;
       Ok(())
   }
   ```

3. **Handle terminal resize** (1 hour)
   ```rust
   pub fn check_size(&self) -> Result<(u16, u16)> {
       let size = terminal::size()?;
       if size.0 < 80 || size.1 < 24 {
           return Err(TuiError::TerminalTooSmall { 
               width: size.0, 
               height: size.1 
           });
       }
       Ok(size)
   }
   ```

#### Task 3.2.4: Layout System (2 hours)

Create responsive layouts:
- Adapt to terminal size
- Minimum size requirements
- Graceful degradation

---

## 🛑 CHECKPOINT 2: TUI Framework Ready

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without framework review and approval.

### Pre-Checkpoint Checklist

- [ ] Event loop non-blocking and responsive
- [ ] Terminal restoration tested (including panic)
- [ ] State transitions working correctly
- [ ] Basic keyboard navigation functional
- [ ] No terminal artifacts on resize
- [ ] Mouse support optional but working
- [ ] Memory usage stable over time

### Framework Testing

```bash
# Test terminal restoration
cargo run --example tui_framework
# Press Ctrl+C, verify terminal restored

# Test panic recovery
cargo run --example tui_panic_test
# Verify terminal usable after panic

# Test resize handling
# Run TUI and resize terminal window
```

### Review Criteria

#### Architecture (Senior Dev)
- [ ] Clean separation of concerns
- [ ] Event handling is extensible
- [ ] State management is predictable
- [ ] No tight coupling between components

#### Reliability (QA)
- [ ] Terminal always restored properly
- [ ] Handles all key combinations
- [ ] Graceful degradation on small terminals
- [ ] No memory leaks in event loop

### Consequences of Skipping

- Broken terminals frustrate users
- Event handling bugs multiply with features
- State management issues hard to fix later
- Poor foundation affects all TUI screens

---

### 3.3 TUI Screens Implementation (35 hours)

**Complexity**: High - Multiple interactive screens
**Files**: `src/tui/screens/*.rs`, `src/tui/widgets/*.rs`

#### Task 3.3.1: Profile List Screen (8 hours)

💡 **Junior Dev Concept**: TUI Screen Components
**What it is**: Each screen is a combination of:
- State (what's selected, what data to show)
- Render function (how to draw it)
- Event handler (how to respond to input)
**Pattern**: Keep each screen self-contained for easier testing

Main screen showing all profiles:

```rust
pub struct ProfileListScreen {
    profiles: Vec<ProfileSummary>,
    selected: usize,
    filter: String,
    sort_order: SortOrder,
}
```

**Features**:
- Scrollable list with selection
- Status indicators (active, valid, signing)
- Quick actions (Enter: switch, e: edit, d: delete)
- Sort by name, date, usage
- Filter/search bar

#### Task 3.3.2: Profile Creation Wizard (12 hours)

💡 **Junior Dev Concept**: Multi-Step Wizards
**What it is**: Guide users through complex tasks one step at a time
**Why effective**: Reduces cognitive load, ensures nothing missed
**Key UX**: Always show progress, allow going back, validate before proceeding

6-step wizard for profile creation:

1. **Basic Info**: Name and description
2. **User Details**: Name and email with validation
3. **Signing Method**: None, SSH, GPG, x509, gitsign
4. **Signing Key**: Browse available keys
5. **Additional Config**: Proxy, custom fields
6. **Review & Create**: Summary with validation

**Per Step Requirements**:
- Real-time validation
- Help text
- Navigation (next, previous, cancel)
- Progress indicator

**Step-by-Step Implementation**:

1. **Design the wizard state machine** (2 hours)
   ```rust
   pub struct WizardState {
       current_step: WizardStep,
       profile_builder: ProfileBuilder,
       validation_errors: Vec<ValidationError>,
       can_proceed: bool,
   }
   
   #[derive(Clone, Copy, PartialEq)]
   pub enum WizardStep {
       BasicInfo,
       UserDetails,
       SigningMethod,
       SigningKey,
       AdditionalConfig,
       ReviewCreate,
   }
   
   impl WizardStep {
       pub fn next(self) -> Option<Self> {
           match self {
               Self::BasicInfo => Some(Self::UserDetails),
               Self::UserDetails => Some(Self::SigningMethod),
               Self::SigningMethod => Some(Self::SigningKey),
               Self::SigningKey => Some(Self::AdditionalConfig),
               Self::AdditionalConfig => Some(Self::ReviewCreate),
               Self::ReviewCreate => None,
           }
       }
       
       pub fn progress(self) -> (usize, usize) {
           (self as usize + 1, 6)
       }
   }
   ```

2. **Implement step rendering** (4 hours)
   ```rust
   impl WizardState {
       pub fn render(&mut self, f: &mut Frame, area: Rect) {
           // Split area into sections
           let chunks = Layout::default()
               .direction(Direction::Vertical)
               .constraints([
                   Constraint::Length(3),  // Progress bar
                   Constraint::Min(10),    // Content
                   Constraint::Length(3),  // Navigation help
               ])
               .split(area);
           
           // Render progress bar
           self.render_progress(f, chunks[0]);
           
           // Render current step
           match self.current_step {
               WizardStep::BasicInfo => self.render_basic_info(f, chunks[1]),
               WizardStep::UserDetails => self.render_user_details(f, chunks[1]),
               // ...
           }
           
           // Render navigation help
           self.render_navigation(f, chunks[2]);
       }
       
       fn render_progress(&self, f: &mut Frame, area: Rect) {
           let (current, total) = self.current_step.progress();
           let progress = (current as f64 / total as f64 * 100.0) as u16;
           
           let gauge = Gauge::default()
               .block(Block::default().borders(Borders::ALL).title("Progress"))
               .gauge_style(Style::default().fg(Color::Green))
               .percent(progress)
               .label(format!("{}/{}: {}", current, total, self.current_step));
           
           f.render_widget(gauge, area);
       }
   }
   ```
   
   💡 **Layout Tip**: Use Constraint::Min for content that needs space

3. **Add real-time validation** (3 hours)
   ```rust
   fn validate_current_step(&mut self) {
       self.validation_errors.clear();
       
       match self.current_step {
           WizardStep::UserDetails => {
               // Validate email
               if !is_valid_email(&self.profile_builder.email) {
                   self.validation_errors.push(
                       ValidationError::new("email", "Invalid email format")
                   );
               }
               
               // Validate name not empty
               if self.profile_builder.name.trim().is_empty() {
                   self.validation_errors.push(
                       ValidationError::new("name", "Name cannot be empty")
                   );
               }
           }
           // ... other steps
       }
       
       self.can_proceed = self.validation_errors.is_empty();
   }
   ```

4. **Handle navigation and input** (3 hours)
   ```rust
   pub fn handle_input(&mut self, key: KeyCode) -> Result<WizardAction> {
       match key {
           KeyCode::Tab => {
               // Move to next field in current step
               self.next_field();
           }
           KeyCode::BackTab => {
               // Move to previous field
               self.previous_field();
           }
           KeyCode::Right | KeyCode::Enter => {
               // Try to proceed to next step
               if self.can_proceed {
                   if let Some(next) = self.current_step.next() {
                       self.current_step = next;
                   } else {
                       // On last step, create profile
                       return Ok(WizardAction::Complete(self.build_profile()?));
                   }
               }
           }
           KeyCode::Left => {
               // Go back to previous step
               if let Some(prev) = self.current_step.previous() {
                   self.current_step = prev;
               }
           }
           KeyCode::Esc => {
               return Ok(WizardAction::Cancel);
           }
           KeyCode::Char(c) => {
               // Input character to current field
               self.handle_char_input(c);
               self.validate_current_step();
           }
           _ => {}
       }
       
       Ok(WizardAction::Continue)
   }
   ```
   
   ⚠️ **UX Rule**: Always validate on input, not just on "next"

#### Task 3.3.3: Profile Detail/Edit Screen (8 hours)

View and edit existing profiles:
- Syntax highlighted TOML display
- Inline editing with validation
- Diff view for changes
- Test configuration option

#### Task 3.3.4: Help System (4 hours)

Context-sensitive help:
- Global keybindings reference
- Screen-specific commands
- Searchable help topics
- Tips and examples

#### Task 3.3.5: Settings Screen (3 hours)

Application settings:
- Default profile location
- UI preferences (colors, shortcuts)
- Integration settings
- Import/export app config

---

## 🛑 CHECKPOINT 3: Core TUI Screens Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without UX review and testing.

### Pre-Checkpoint Checklist

- [ ] All screens implemented and navigable
- [ ] Wizard completes profile creation successfully
- [ ] Validation provides helpful feedback
- [ ] Keyboard shortcuts consistent across screens
- [ ] Help available on every screen (? key)
- [ ] Screen transitions smooth (<16ms)
- [ ] Accessibility: Screen reader friendly output

### User Testing Protocol

```bash
# Test wizard flow
cargo run -- tui
# Press 'n' for new profile
# Complete all wizard steps
# Verify profile created

# Test navigation
# Use arrow keys, vim keys, tab
# Verify all methods work

# Test help system
# Press '?' on each screen
# Verify context-appropriate help
```

### Review Requirements

#### UX Review (Designer + Users)
- [ ] Wizard flow intuitive
- [ ] Error messages helpful
- [ ] Progress clearly shown
- [ ] No dead ends in navigation

#### Technical Review (Senior Dev)
- [ ] State management clean
- [ ] Rendering performance good
- [ ] Memory usage stable
- [ ] Code maintainable

#### Accessibility Review
- [ ] Keyboard-only navigation works
- [ ] High contrast mode support
- [ ] Screen reader compatible output

### Consequences of Skipping

- Users get lost in the interface
- Incomplete profiles created
- Poor accessibility excludes users
- Rework requires state refactoring

---

### 3.4 Fuzzy Search Integration (20 hours)

**Complexity**: Medium - Performance critical
**Files**: `src/tui/search.rs`, `src/search/mod.rs`

#### Task 3.4.1: Nucleo Integration (6 hours)

💡 **Junior Dev Concept**: Fuzzy Search
**What it is**: Search that finds matches even with typos ("wrk" finds "work")
**Why Nucleo**: It's FAST - can search 100k items in milliseconds
**How it works**: Scores each item based on character matches and positions

Integrate Nucleo fuzzy finder:

```rust
use nucleo::{Nucleo, Config};

pub struct FuzzySearch {
    nucleo: Nucleo<ProfileItem>,
    results: Vec<SearchResult>,
    query: String,
}

impl FuzzySearch {
    pub fn update_query(&mut self, query: &str) -> Result<()> {
        self.query = query.to_string();
        self.results = self.nucleo
            .search(query, Config::DEFAULT)
            .take(20)
            .collect();
        Ok(())
    }
}
```

#### Task 3.4.2: Search UI Component (8 hours)

Build reusable search widget:
- Input field with cursor
- Results list with highlighting
- Keyboard navigation
- Preview pane (optional)
- Performance indicators

#### Task 3.4.3: Search Contexts (4 hours)

Implement search in multiple contexts:
- Profile selection
- SSH key selection
- Command palette
- Help search

#### Task 3.4.4: Search Performance (2 hours)

Optimize for <10ms response:
- Incremental search
- Result caching
- Background indexing
- Debouncing

**Performance Optimization Steps**:

1. **Implement debouncing** (30 minutes)
   ```rust
   use tokio::time::{sleep, Duration, Instant};
   
   pub struct DebouncedSearch {
       last_query: String,
       last_update: Instant,
       debounce_ms: u64,
   }
   
   impl DebouncedSearch {
       pub async fn search(&mut self, query: &str) -> Option<Vec<SearchResult>> {
           if query == self.last_query {
               return None;  // No change
           }
           
           let now = Instant::now();
           let elapsed = now.duration_since(self.last_update);
           
           if elapsed < Duration::from_millis(self.debounce_ms) {
               // Wait for user to stop typing
               sleep(Duration::from_millis(self.debounce_ms) - elapsed).await;
           }
           
           self.last_query = query.to_string();
           self.last_update = Instant::now();
           
           Some(self.perform_search(query).await)
       }
   }
   ```
   
   💡 **Why debounce**: Prevents searching on every keystroke

2. **Add result caching** (30 minutes)
   ```rust
   use lru::LruCache;
   
   pub struct SearchCache {
       cache: LruCache<String, Vec<SearchResult>>,
   }
   
   impl SearchCache {
       pub fn get_or_compute<F>(&mut self, query: &str, compute: F) -> Vec<SearchResult>
       where
           F: FnOnce() -> Vec<SearchResult>,
       {
           if let Some(results) = self.cache.get(query) {
               return results.clone();
           }
           
           let results = compute();
           self.cache.put(query.to_string(), results.clone());
           results
       }
   }
   ```

3. **Profile and optimize** (1 hour)
   ```rust
   #[cfg(test)]
   mod bench {
       use super::*;
       use criterion::{black_box, criterion_group, Criterion};
       
       fn bench_search(c: &mut Criterion) {
           let items: Vec<_> = (0..10000)
               .map(|i| ProfileItem {
                   name: format!("profile_{}", i),
                   email: format!("user{}@example.com", i),
               })
               .collect();
           
           let mut search = FuzzySearch::new(items);
           
           c.bench_function("search_10k_items", |b| {
               b.iter(|| {
                   search.update_query(black_box("prof_42"))
               });
           });
       }
   }
   ```

---

## 🛑 CHECKPOINT 4: Fuzzy Search Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without performance verification.

### Pre-Checkpoint Checklist

- [ ] Search responds in <10ms for 1000 items
- [ ] Results ranked by relevance
- [ ] Match highlighting correct
- [ ] Keyboard navigation smooth
- [ ] Memory usage stable during search
- [ ] Works with Unicode (émails, 中文, etc.)

### Performance Testing

```bash
# Run search benchmarks
cargo bench --bench search_performance

# Test with large dataset
cargo run --example search_stress_test

# Profile memory usage
heaptrack cargo run --example search_demo
```

### Review Criteria

#### Performance (Tech Lead)
- [ ] <10ms response time verified
- [ ] Memory usage acceptable
- [ ] No UI freezing during search
- [ ] Handles 10k+ items

#### Usability (UX)
- [ ] Results make sense to users
- [ ] Highlighting helps find matches
- [ ] Navigation intuitive
- [ ] Works for common typos

### Consequences of Skipping

- Slow search frustrates users
- Poor relevance makes search useless
- Memory leaks in production
- Performance degrades with scale

---

### 3.5 Polish and Integration (20 hours)

**Complexity**: Medium - Refinement phase
**Files**: `src/tui/theme.rs`, `src/tui/config.rs`, tests

#### Task 3.5.1: Keybinding System (6 hours)

💡 **Junior Dev Concept**: Configurable Keybindings
**What it is**: Let users choose their keyboard shortcuts
**Why important**: Vim users want hjkl, Emacs users want Ctrl+n/p
**Implementation**: Map from key to action, not hardcode keys

Implement customizable keybindings:
- Vim-style navigation (hjkl)
- Emacs alternatives
- Custom binding support
- Keybinding help overlay

**Step-by-Step Implementation**:

1. **Design keybinding system** (2 hours)
   ```rust
   use std::collections::HashMap;
   
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub enum Action {
       MoveUp,
       MoveDown,
       MoveLeft,
       MoveRight,
       Select,
       Back,
       Quit,
       Help,
       Search,
       // ... more actions
   }
   
   pub struct KeyBindings {
       bindings: HashMap<KeyEvent, Action>,
       presets: HashMap<String, HashMap<KeyEvent, Action>>,
   }
   
   impl KeyBindings {
       pub fn with_preset(preset: &str) -> Self {
           let mut bindings = Self::default();
           
           match preset {
               "vim" => bindings.load_vim_preset(),
               "emacs" => bindings.load_emacs_preset(),
               _ => bindings.load_default_preset(),
           }
           
           bindings
       }
       
       fn load_vim_preset(&mut self) {
           self.bind(key!('h'), Action::MoveLeft);
           self.bind(key!('j'), Action::MoveDown);
           self.bind(key!('k'), Action::MoveUp);
           self.bind(key!('l'), Action::MoveRight);
           self.bind(key!('g'), Action::Top);
           self.bind(key!('G'), Action::Bottom);
           // ... more vim bindings
       }
   }
   ```
   
   💡 **Design Pattern**: Strategy pattern for different keybinding sets

2. **Create help overlay** (2 hours)
   ```rust
   pub struct HelpOverlay {
       visible: bool,
       context: HelpContext,
   }
   
   impl HelpOverlay {
       pub fn render(&self, f: &mut Frame, area: Rect, bindings: &KeyBindings) {
           if !self.visible {
               return;
           }
           
           // Semi-transparent overlay
           let overlay_area = centered_rect(80, 80, area);
           f.render_widget(Clear, overlay_area);  // Clear background
           
           // Group bindings by category
           let categories = self.categorize_bindings(bindings);
           
           let help_text: Vec<Line> = categories.iter()
               .flat_map(|(category, bindings)| {
                   let mut lines = vec![Line::from(category.to_string()).bold()];
                   
                   for (key, action) in bindings {
                       lines.push(Line::from(vec![
                           Span::styled(
                               format!("{:>10}", key_to_string(key)),
                               Style::default().fg(Color::Cyan)
                           ),
                           Span::raw(" - "),
                           Span::raw(action_description(action)),
                       ]));
                   }
                   
                   lines.push(Line::default());  // Empty line
                   lines
               })
               .collect();
           
           let help = Paragraph::new(help_text)
               .block(Block::default()
                   .borders(Borders::ALL)
                   .title(" Help (press ? to close) ")
               )
               .wrap(Wrap { trim: true });
           
           f.render_widget(help, overlay_area);
       }
   }
   ```

3. **Support custom bindings** (2 hours)
   ```rust
   impl KeyBindings {
       pub fn from_config(config: &KeybindingConfig) -> Result<Self> {
           let mut bindings = Self::with_preset(&config.preset);
           
           // Apply custom overrides
           for (key_str, action_str) in &config.custom {
               let key = parse_key(key_str)?;
               let action = parse_action(action_str)?;
               bindings.bind(key, action);
           }
           
           // Validate no conflicts
           bindings.validate()?;
           
           Ok(bindings)
       }
       
       fn validate(&self) -> Result<()> {
           // Check for essential bindings
           let essential = [Action::Quit, Action::Help, Action::Select];
           
           for action in &essential {
               if !self.bindings.values().any(|a| a == action) {
                   return Err(ConfigError::MissingBinding(*action));
               }
           }
           
           Ok(())
       }
   }
   ```

#### Task 3.5.2: Theme System (4 hours)

Support multiple color themes:
- Default (Gruvbox-inspired)
- Light theme
- High contrast
- Custom theme support

#### Task 3.5.3: Animation & Feedback (4 hours)

Polish interactions:
- Smooth scrolling
- Loading spinners
- Success/error notifications
- Progress bars for operations

#### Task 3.5.4: Terminal Compatibility Testing (4 hours)

💡 **Junior Dev Concept**: Terminal Compatibility
**The problem**: Different terminals support different features
**Examples**: Not all support true color, mouse, or Unicode
**Solution**: Detect capabilities and gracefully degrade

Test on multiple terminals:
- **macOS**: Terminal.app, iTerm2, Alacritty
- **Linux**: gnome-terminal, konsole, xterm
- **Windows**: Windows Terminal, ConEmu
- **Cross-platform**: VS Code terminal, tmux

**Testing Protocol**:

1. **Create compatibility test suite** (1 hour)
   ```rust
   pub struct TerminalCapabilities {
       pub true_color: bool,
       pub unicode: bool,
       pub mouse: bool,
       pub size: (u16, u16),
   }
   
   impl TerminalCapabilities {
       pub fn detect() -> Self {
           Self {
               true_color: Self::supports_true_color(),
               unicode: Self::supports_unicode(),
               mouse: Self::supports_mouse(),
               size: terminal::size().unwrap_or((80, 24)),
           }
       }
       
       fn supports_true_color() -> bool {
           // Check COLORTERM env var
           std::env::var("COLORTERM")
               .map(|v| v == "truecolor" || v == "24bit")
               .unwrap_or(false)
       }
       
       fn supports_unicode() -> bool {
           // Check LANG/LC_ALL for UTF-8
           std::env::var("LANG")
               .or_else(|_| std::env::var("LC_ALL"))
               .map(|v| v.contains("UTF-8") || v.contains("utf8"))
               .unwrap_or(false)
       }
   }
   ```

2. **Test on each terminal** (2 hours)
   ```bash
   #!/bin/bash
   # test-terminals.sh
   
   echo "Testing Terminal Compatibility"
   echo "============================="
   
   # Build test binary
   cargo build --example terminal_test
   
   # Test in different terminals
   terminals=(
       "Terminal.app"
       "iTerm.app"
       "Alacritty"
       "gnome-terminal"
       "konsole"
   )
   
   for term in "${terminals[@]}"; do
       echo "\nTesting $term..."
       # Run test and capture output
       # Manual step: Actually run in each terminal
   done
   ```

3. **Document compatibility** (1 hour)
   ```markdown
   # Terminal Compatibility Matrix
   
   | Terminal | True Color | Unicode | Mouse | Min Size | Notes |
   |----------|------------|---------|-------|----------|-------|
   | Terminal.app | ❌ | ✅ | ❌ | 80x24 | Use 256 colors |
   | iTerm2 | ✅ | ✅ | ✅ | 80x24 | Full support |
   | Windows Terminal | ✅ | ✅ | ✅ | 80x24 | Full support |
   | ConEmu | ⚠️ | ✅ | ✅ | 80x24 | True color varies |
   | tmux | ⚠️ | ✅ | ⚠️ | 80x24 | Depends on outer terminal |
   
   ## Fallback Strategies
   
   ### Colors
   - True color → 256 colors → 16 colors → no color
   - Test: `echo $COLORTERM` and `tput colors`
   
   ### Unicode
   - Full Unicode → ASCII art → plain text
   - Test: `echo $LANG` contains UTF-8
   
   ### Mouse
   - Mouse support → keyboard only
   - Test: Try enabling and check for errors
   ```

#### Task 3.5.5: Documentation (2 hours)

User-facing documentation:
- README with screenshots
- Keyboard shortcuts reference
- Video demo/tutorial
- Troubleshooting guide

---

## 🛑 CHECKPOINT 5: Phase 3 Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** to Phase 4 without final review.

### Pre-Checkpoint Checklist

- [ ] CLI passes all integration tests
- [ ] TUI works on all target terminals
- [ ] Search performance verified <10ms
- [ ] Keybindings customizable
- [ ] Theme system working
- [ ] Help system comprehensive
- [ ] Terminal compatibility documented
- [ ] User testing completed

### Final Testing Protocol

```bash
# Full CLI test suite
./scripts/test-cli-complete.sh

# TUI on all platforms
./scripts/test-tui-platforms.sh

# Performance benchmarks
cargo bench --features ui

# User acceptance tests
./scripts/run-uat.sh
```

### Phase 3 Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Test Coverage | ≥80% | ___ | ⬜ |
| TUI Launch Time | <100ms | ___ | ⬜ |
| Search Response | <10ms | ___ | ⬜ |
| Terminal Support | 100% | ___ | ⬜ |
| User Satisfaction | >4/5 | ___ | ⬜ |

### User Testing Results

**Test Group**: 5 developers (2 junior, 2 mid, 1 senior)

| Task | Success Rate | Avg Time | Issues Found |
|------|--------------|----------|-------------|
| Create profile via CLI | ___% | ___s | ___ |
| Create profile via TUI | ___% | ___s | ___ |
| Find profile with search | ___% | ___s | ___ |
| Switch profiles | ___% | ___s | ___ |
| Access help | ___% | ___s | ___ |

### Sign-offs Required

- [ ] **UX Designer**: Interface approved
- [ ] **Tech Lead**: Code quality acceptable
- [ ] **QA Lead**: Testing complete
- [ ] **Product Owner**: Features match spec
- [ ] **Accessibility**: WCAG 2.1 AA compliant

### Handoff to Phase 4

**What Phase 4 needs**:
1. Stable TUI framework for 1Password integration
2. Search system for browsing vaults/items
3. Async UI patterns for network operations
4. Error display patterns

---

## Testing Strategy

### CLI Testing
```rust
#[test]
fn test_profile_list_json_output() {
    let mut cmd = Command::cargo_bin("git-setup").unwrap();
    cmd.arg("list").arg("--format").arg("json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"profiles\""));
}
```

### TUI Testing
- State machine tests (unit)
- Widget rendering tests (snapshot)
- Event handling tests (unit)
- Integration tests with mock backend

### Manual Testing Matrix
| Terminal | macOS | Linux | Windows |
|----------|-------|-------|---------|
| Native | ✓ | ✓ | ✓ |
| VS Code | ✓ | ✓ | ✓ |
| tmux | ✓ | ✓ | - |
| SSH | ✓ | ✓ | ✓ |

## Common Issues & Solutions

### Issue: TUI Rendering Glitches
**Symptom**: Artifacts, incorrect colors, broken borders
**Likely Cause**: Terminal doesn't support required features
**Solution**:
```rust
// Detect and fallback
let caps = TerminalCapabilities::detect();
let theme = if caps.true_color {
    Theme::default()
} else {
    Theme::basic_16_colors()
};
```

### Issue: Slow Fuzzy Search
**Symptom**: Typing lag in search box
**Cause**: Searching too many items without optimization
**Solution**:
```rust
// Limit results and add debouncing
const MAX_RESULTS: usize = 50;
const DEBOUNCE_MS: u64 = 50;

// Only show top results
let results: Vec<_> = search.find(query)
    .take(MAX_RESULTS)
    .collect();
```

### Issue: Keybinding Conflicts
**Symptom**: Ctrl+S doesn't work (terminal intercepts)
**Cause**: Terminal emulator captures certain keys
**Solution**:
```rust
// Provide alternatives
bindings.bind(key!(Ctrl-'s'), Action::Save);
bindings.bind(key!('S'), Action::Save);  // Shift+S as alternative
```

### Issue: Unicode Rendering Issues
**Symptom**: Boxes instead of icons/borders
**Cause**: Terminal font missing glyphs
**Solution**:
```rust
pub fn get_icons(unicode_safe: bool) -> Icons {
    if unicode_safe {
        Icons {
            check: "✓",
            cross: "✗",
            arrow: "→",
        }
    } else {
        Icons {
            check: "[OK]",
            cross: "[X]",
            arrow: ">>",
        }
    }
}
```

### Issue: Panic Leaves Terminal Broken
**Symptom**: Terminal doesn't accept input after crash
**Cause**: Raw mode not disabled on panic
**Solution**: Already implemented in terminal wrapper!
```bash
# If it still happens:
reset  # or
stty sane
```

## Performance Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| TUI Launch | <100ms | <200ms |
| Screen Switch | <16ms | <33ms |
| Fuzzy Search | <10ms | <20ms |
| List Render (1000) | <50ms | <100ms |
| Wizard Step | <16ms | <33ms |

## Security Considerations

- No credentials displayed in UI
- Mask sensitive values
- Clear clipboard after copy
- No logging of sensitive data
- Secure tempfile for editing

## Junior Developer Tips

### Getting Started with Phase 3

1. **Learn the basics first**:
   - Complete Ratatui tutorial (2-3 hours)
   - Try modifying examples before writing from scratch
   - Understand event loops (critical!)

2. **Development workflow**:
   - Use `cargo watch -x 'run -- tui'` for hot reload
   - Test one key at a time when adding bindings
   - Keep terminal reset command handy

3. **Debugging TUI apps**:
   ```rust
   // Log to file since stdout is used by TUI
   use log::debug;
   use simplelog::*;
   
   WriteLogger::init(
       LevelFilter::Debug,
       Config::default(),
       File::create("tui-debug.log").unwrap(),
   ).unwrap();
   
   debug!("Current state: {:?}", app.state);
   ```

4. **Common TUI pitfalls**:
   - Forgetting to handle terminal resize
   - Not restoring terminal on exit
   - Blocking the event loop
   - Hardcoding colors (use theme!)

### Pair Programming Suggestions

- **CLI Architecture**: Design session recommended
- **Event Loop Setup**: Critical to get right
- **Wizard State Machine**: Complex state management
- **Performance Tuning**: Learn profiling tools

## Next Phase Preview

Phase 4 (1Password Integration) will:
- Add 1Password vault browsing in TUI
- Enable SSH key selection from 1Password
- Support biometric authentication
- Handle async operations in UI

**What Phase 4 needs from Phase 3**:
- Async-friendly TUI architecture
- List widgets for browsing items
- Loading states for network operations
- Secure input handling for passwords

---

*Last updated: 2025-07-30*