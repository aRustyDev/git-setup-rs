# Phase 3: User Interfaces - Work Plan

## Prerequisites

Before starting Phase 3, ensure you're comfortable with terminal applications and event-driven programming.

**Required from Previous Phases**:
- ✅ Profile management API complete (Phase 2)
- ✅ CRUD operations working
- ✅ Git integration functional
- ✅ Error handling patterns established

**Required Knowledge**:
- **Rust Basics**: Ownership, traits, error handling (*critical*)
- **Terminal Concepts**: How terminals display text (*required*)
- **Event Loops**: Basic understanding of event-driven apps (*helpful*)
- **Command Patterns**: CLI design principles (*helpful*)

💡 **Junior Dev Resources**:
- 📚 [Command Line Apps in Rust](https://rust-cli.github.io/book/) - Start here! (2 hours)
- 📖 [TUI Development Guide](https://ratatui.rs/tutorial/) - Comprehensive intro
- 📖 [Ratatui Tutorial](https://ratatui.rs/tutorial/) - Step-by-step guide
- 🔧 Practice: Complete `examples/tui/simple_menu.rs` first
- 📝 [Terminal Concepts](../../resources/terminal-guide.md) - How terminals work
- 🎯 [Event Handling Examples](EVENT_HANDLING_EXAMPLES.md) - **Essential** for keyboard/mouse handling
- 🖼️ [TUI Mockups](TUI_MOCKUPS.md) - Visual guide for all screens
- ⚡ [Async/Await Examples](../ASYNC_AWAIT_EXAMPLES.md) - For responsive UI patterns

## Quick Reference - Essential Resources

### Key Concepts to Understand
```
CLI (Command Line Interface)     TUI (Terminal User Interface)
─────────────────────────       ────────────────────────────
$ git-setup profile list        ┌─ Git Setup ─────────┐
work                            │ > work              │
personal                        │   personal          │
client-a                        │   client-a          │
                               └─────────────────────┘
        ↑                                  ↑
   Text output                    Interactive interface
```

### Commands You'll Use
```bash
# Test CLI commands
cargo run -- --help
cargo run -- profile list
cargo run -- profile show work

# Launch TUI
cargo run -- tui

# Run UI tests
cargo test ui:: -- --nocapture
```

## Overview

Phase 3 creates the user interfaces that make git-setup-rs accessible. We'll build both a CLI for scripting and a TUI for interactive use.

**What You'll Build**:
1. CLI with subcommands for all operations
2. Interactive TUI with menus and forms
3. Keyboard navigation system
4. Real-time search functionality
5. Context-sensitive help

**Success Looks Like**:
- User types `git-setup profile list` → sees their profiles
- User launches TUI → navigates with arrow keys
- User searches for profile → finds it instantly
- Everything works on Windows, Mac, and Linux

**Time Estimate**: 3 weeks (120 hours)
- Week 1: CLI implementation (40h)
- Week 2: Basic TUI (40h)
- Week 3: Advanced TUI features (40h)

## Done Criteria Checklist

Before Phase 4, ensure:
- [ ] CLI commands all working
- [ ] TUI navigation smooth
- [ ] Search responds instantly
- [ ] Help available everywhere
- [ ] Cross-platform tested
- [ ] No panic on resize
- [ ] Graceful error handling
- [ ] Documentation complete

## Week 1: CLI Implementation

### 3.1 Command Line Interface (32 hours)

#### Task 3.1.1: CLI Structure with Clap (8 hours)

💡 **Junior Dev Concept**: Command-Line Parsing with Clap
**What it is**: A library that turns Rust structs into command-line interfaces
**Why we use it**: Automatic help generation, validation, and great error messages
**Real Example**: `git-setup profile create --name work --email work@company.com`

**Prerequisites**:
- [ ] Read: [Clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [ ] Understand: Rust enums and pattern matching
- [ ] Try: `examples/cli/basic_clap.rs`

**Visual Command Structure**:
```
git-setup
├── profile
│   ├── list                 # Show all profiles
│   ├── show <name>          # Show specific profile
│   ├── create               # Create new profile
│   ├── edit <name>          # Edit existing
│   ├── delete <name>        # Remove profile
│   └── use <name>           # Apply profile
├── config
│   ├── get <key>            # Get config value
│   └── set <key> <value>    # Set config value
├── health                   # Run diagnostics
└── tui                      # Launch TUI
```

**Step-by-Step Implementation**:

1. **Create Main CLI Structure** (2 hours)
   ```rust
   // src/cli/mod.rs
   
   use clap::{Parser, Subcommand};
   
   /// Git profile management made easy
   #[derive(Parser, Debug)]
   #[command(author, version, about, long_about = None)]
   #[command(propagate_version = true)]
   pub struct Cli {
       /// Increase logging verbosity
       #[arg(short, long, action = clap::ArgAction::Count)]
       pub verbose: u8,
       
       /// Suppress all output
       #[arg(short, long, conflicts_with = "verbose")]
       pub quiet: bool,
       
       /// Output format
       #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
       pub format: OutputFormat,
       
       #[command(subcommand)]
       pub command: Commands,
   }
   
   #[derive(Subcommand, Debug)]
   pub enum Commands {
       /// Manage Git profiles
       Profile(ProfileCommand),
       
       /// Manage configuration
       Config(ConfigCommand),
       
       /// Check system health
       Health {
           /// Show all checks, not just failures
           #[arg(short, long)]
           all: bool,
       },
       
       /// Launch interactive TUI
       Tui,
   }
   
   #[derive(Clone, Debug, ValueEnum)]
   pub enum OutputFormat {
       /// Human-readable text
       Text,
       /// JSON output
       Json,
       /// YAML output
       Yaml,
       /// Tab-separated values
       Tsv,
   }
   ```
   
   💡 **Tip**: `#[command(propagate_version = true)]` shows version in all subcommands

2. **Implement Profile Subcommands** (3 hours)
   ```rust
   // src/cli/commands/profile.rs
   
   use clap::{Args, Subcommand};
   
   #[derive(Args, Debug)]
   pub struct ProfileCommand {
       #[command(subcommand)]
       pub command: ProfileSubcommands,
   }
   
   #[derive(Subcommand, Debug)]
   pub enum ProfileSubcommands {
       /// List all profiles
       List {
           /// Show detailed information
           #[arg(short, long)]
           long: bool,
           
           /// Filter by pattern
           #[arg(short, long)]
           filter: Option<String>,
       },
       
       /// Show profile details
       Show {
           /// Profile name
           name: String,
           
           /// Show with decrypted secrets (careful!)
           #[arg(long)]
           show_secrets: bool,
       },
       
       /// Create a new profile
       Create {
           /// Profile name
           #[arg(short, long)]
           name: String,
           
           /// Git user name
           #[arg(long)]
           user_name: String,
           
           /// Git user email
           #[arg(long)]
           user_email: String,
           
           /// Profile description
           #[arg(short, long)]
           description: Option<String>,
           
           /// Parent profile to extend
           #[arg(short, long)]
           extends: Option<String>,
           
           /// Start interactive wizard instead
           #[arg(short, long)]
           interactive: bool,
       },
       
       /// Edit existing profile
       Edit {
           /// Profile to edit
           name: String,
           
           /// Open in $EDITOR
           #[arg(short, long)]
           editor: bool,
       },
       
       /// Delete a profile
       Delete {
           /// Profile to delete
           name: String,
           
           /// Skip confirmation
           #[arg(short, long)]
           force: bool,
       },
       
       /// Apply profile to current repository
       Use {
           /// Profile to apply
           name: String,
           
           /// Apply globally instead of locally
           #[arg(short, long)]
           global: bool,
       },
   }
   ```
   
   ⚠️ **Common Mistake**: Forgetting to handle missing profiles
   ✅ **Instead**: Always check if profile exists before operations

3. **Create Command Handlers** (3 hours)
   ```rust
   // src/cli/handlers/profile.rs
   
   use crate::profile::{ProfileManager, ProfileError};
   use crate::cli::{OutputFormat, ProfileSubcommands};
   
   pub struct ProfileHandler {
       manager: Box<dyn ProfileManager>,
       format: OutputFormat,
   }
   
   impl ProfileHandler {
       pub fn new(manager: Box<dyn ProfileManager>, format: OutputFormat) -> Self {
           Self { manager, format }
       }
       
       pub async fn handle(&self, command: ProfileSubcommands) -> Result<(), CliError> {
           match command {
               ProfileSubcommands::List { long, filter } => {
                   self.handle_list(long, filter).await
               }
               ProfileSubcommands::Show { name, show_secrets } => {
                   self.handle_show(&name, show_secrets).await
               }
               ProfileSubcommands::Create { name, user_name, user_email, .. } => {
                   self.handle_create(name, user_name, user_email).await
               }
               // ... other commands
           }
       }
       
       async fn handle_list(&self, long: bool, filter: Option<String>) -> Result<(), CliError> {
           let profiles = self.manager.list().await
               .map_err(|e| CliError::Profile(e))?;
           
           // Apply filter if provided
           let profiles: Vec<_> = if let Some(pattern) = filter {
               profiles.into_iter()
                   .filter(|p| p.contains(&pattern))
                   .collect()
           } else {
               profiles
           };
           
           // Format output based on selected format
           match self.format {
               OutputFormat::Text => {
                   if profiles.is_empty() {
                       println!("No profiles found");
                   } else if long {
                       // Detailed view
                       for name in &profiles {
                           let profile = self.manager.get(name).await?;
                           println!("{}", profile.display_long());
                       }
                   } else {
                       // Simple list
                       for name in profiles {
                           println!("{}", name);
                       }
                   }
               }
               OutputFormat::Json => {
                   let json = serde_json::to_string_pretty(&profiles)?;
                   println!("{}", json);
               }
               // ... other formats
           }
           
           Ok(())
       }
   }
   ```

**Testing Your CLI**:
```bash
# Test command parsing
cargo test cli::tests::test_parse_commands

# Try real commands
cargo run -- profile list
cargo run -- profile create --name test --user-name "Test" --user-email "test@example.com"
cargo run -- profile show test --format json

# Test help generation
cargo run -- help
cargo run -- profile help
cargo run -- profile create --help
```

**Debugging Guide**:

**Error**: "unexpected argument"
**Solution**: Check command structure matches enum hierarchy

**Error**: Clap derive not working
**Solution**: Ensure `clap = { version = "4", features = ["derive"] }` in Cargo.toml

**Error**: Commands not recognized
**Solution**: Make sure to use `#[command(subcommand)]` attribute

**When You're Stuck**:
1. Run with `--help` to see generated interface
2. Check examples: `examples/cli/command_structure.rs`
3. Use `dbg!(&args)` to inspect parsed arguments
4. Ask in Slack: #rust-cli channel

---

### 🛑 CHECKPOINT 3.1: CLI Foundation Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 32 hours + 8 hours review = 40 hours total

**Pre-Checkpoint Checklist**:
- [ ] All commands parse correctly
- [ ] Help text clear and complete
- [ ] Output formats working
- [ ] Error messages helpful
- [ ] Integration with ProfileManager works

**Review Focus**:
- Command structure intuitive
- Help text quality
- Error handling comprehensive

---

## Week 2: Basic TUI Implementation

### 3.2 Terminal User Interface Foundation (40 hours)

#### Task 3.2.1: TUI Architecture with Ratatui (10 hours)

💡 **Junior Dev Concept**: Terminal User Interfaces
**What it is**: Full-screen terminal applications with windows, menus, and forms
**Why TUIs**: More intuitive than CLI for complex interactions, works over SSH
**Real Example**: `htop`, `vim`, `tmux` are all TUIs

**Prerequisites**:
- [ ] Complete: `examples/tui/simple_menu.rs`
- [ ] Read: [Ratatui Concepts](https://ratatui.rs/concepts/)
- [ ] Understand: Event loops and state machines
- [ ] Study: [Advanced Ratatui Concepts](./ADVANCED_RATATUI_CONCEPTS.md) - Comprehensive guide

**Visual TUI Architecture**:
```
┌─────────────────────────────────────────┐
│            Terminal                     │
│  ┌─────────────────────────────────┐   │
│  │         Ratatui Frame           │   │
│  │  ┌──────────┐  ┌─────────────┐ │   │
│  │  │  Widget  │  │   Widget    │ │   │
│  │  │  (Menu)  │  │  (Details)  │ │   │
│  │  └──────────┘  └─────────────┘ │   │
│  └─────────────────────────────────┘   │
│         ↑              ↑                │
│     Events          State              │
└─────────────────────────────────────────┘
```

**Step-by-Step Implementation**:

1. **Create App State Structure** (3 hours)
   ```rust
   // src/tui/app.rs
   
   use crate::profile::{Profile, ProfileManager};
   
   /// Main application state
   pub struct App {
       /// Current screen/mode
       pub screen: Screen,
       
       /// Profile manager instance
       pub profile_manager: Box<dyn ProfileManager>,
       
       /// List of profiles (cached)
       pub profiles: Vec<String>,
       
       /// Currently selected profile index
       pub selected_profile: usize,
       
       /// Current profile being viewed/edited
       pub current_profile: Option<Profile>,
       
       /// Input mode for text entry
       pub input_mode: InputMode,
       
       /// Current input buffer
       pub input_buffer: String,
       
       /// Status message to display
       pub status_message: Option<(String, MessageType)>,
       
       /// Should quit?
       pub should_quit: bool,
   }
   
   #[derive(Debug, Clone, Copy, PartialEq)]
   pub enum Screen {
       /// Main menu with profile list
       Main,
       /// Viewing profile details
       ProfileView,
       /// Editing a profile
       ProfileEdit,
       /// Creating new profile wizard
       ProfileWizard(WizardStep),
       /// Help screen
       Help,
   }
   
   #[derive(Debug, Clone, Copy, PartialEq)]
   pub enum WizardStep {
       Name,
       GitUserName,
       GitUserEmail,
       Description,
       Signing,
       Confirm,
   }
   
   #[derive(Debug, Clone, Copy, PartialEq)]
   pub enum InputMode {
       Normal,
       Insert,
       Search,
   }
   
   pub enum MessageType {
       Info,
       Success,
       Warning,
       Error,
   }
   ```
   
   💡 **State Machine**: App flows between screens based on user actions

2. **Implement Event Handling** (4 hours)
   ```rust
   // src/tui/events.rs
   
   use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
   use std::time::Duration;
   
   /// Events that can occur in the TUI
   #[derive(Debug, Clone)]
   pub enum AppEvent {
       /// Keyboard input
       Key(KeyEvent),
       /// Terminal resize
       Resize(u16, u16),
       /// Timer tick for updates
       Tick,
       /// Error occurred
       Error(String),
   }
   
   /// Event handler that polls for events
   pub struct EventHandler {
       /// Event receiver
       rx: mpsc::UnboundedReceiver<AppEvent>,
       /// Event sender
       tx: mpsc::UnboundedSender<AppEvent>,
   }
   
   impl EventHandler {
       pub fn new() -> Self {
           let (tx, rx) = mpsc::unbounded_channel();
           
           // Spawn event polling thread
           let event_tx = tx.clone();
           thread::spawn(move || {
               loop {
                   // Poll for events with timeout
                   if event::poll(Duration::from_millis(100)).unwrap() {
                       if let Ok(event) = event::read() {
                           match event {
                               Event::Key(key) => {
                                   event_tx.send(AppEvent::Key(key)).unwrap();
                               }
                               Event::Resize(w, h) => {
                                   event_tx.send(AppEvent::Resize(w, h)).unwrap();
                               }
                               _ => {}
                           }
                       }
                   }
                   
                   // Send tick for animations/updates
                   event_tx.send(AppEvent::Tick).unwrap();
               }
           });
           
           Self { rx, tx }
       }
       
       /// Get next event (non-blocking)
       pub fn next(&mut self) -> Option<AppEvent> {
           self.rx.try_recv().ok()
       }
   }
   ```

3. **Create Main Event Loop** (3 hours)
   ```rust
   impl App {
       /// Handle a single event
       pub async fn handle_event(&mut self, event: AppEvent) -> Result<(), TuiError> {
           match event {
               AppEvent::Key(key) => self.handle_key(key).await?,
               AppEvent::Resize(_, _) => {} // Ratatui handles this
               AppEvent::Tick => self.handle_tick().await?,
               AppEvent::Error(msg) => {
                   self.show_error(&msg);
               }
           }
           Ok(())
       }
       
       /// Handle keyboard input based on current mode
       async fn handle_key(&mut self, key: KeyEvent) -> Result<(), TuiError> {
           // Global hotkeys (work in any mode)
           match (key.code, key.modifiers) {
               (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                   self.should_quit = true;
                   return Ok(());
               }
               (KeyCode::F(1), _) => {
                   self.screen = Screen::Help;
                   return Ok(());
               }
               _ => {}
           }
           
           // Mode-specific handling
           match self.input_mode {
               InputMode::Normal => self.handle_normal_mode(key).await?,
               InputMode::Insert => self.handle_insert_mode(key).await?,
               InputMode::Search => self.handle_search_mode(key).await?,
           }
           
           Ok(())
       }
       
       /// Handle keys in normal mode (navigation)
       async fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<(), TuiError> {
           match self.screen {
               Screen::Main => {
                   match key.code {
                       KeyCode::Up | KeyCode::Char('k') => {
                           self.previous_profile();
                       }
                       KeyCode::Down | KeyCode::Char('j') => {
                           self.next_profile();
                       }
                       KeyCode::Enter => {
                           self.view_selected_profile().await?;
                       }
                       KeyCode::Char('n') => {
                           self.start_new_profile_wizard();
                       }
                       KeyCode::Char('d') => {
                           self.delete_selected_profile().await?;
                       }
                       KeyCode::Char('/') => {
                           self.input_mode = InputMode::Search;
                           self.input_buffer.clear();
                       }
                       KeyCode::Char('q') => {
                           self.should_quit = true;
                       }
                       _ => {}
                   }
               }
               Screen::ProfileView => {
                   match key.code {
                       KeyCode::Char('e') => {
                           self.screen = Screen::ProfileEdit;
                           self.input_mode = InputMode::Insert;
                       }
                       KeyCode::Esc | KeyCode::Char('q') => {
                           self.screen = Screen::Main;
                       }
                       _ => {}
                   }
               }
               // ... handle other screens
           }
           Ok(())
       }
   }
   ```
   
   ⚠️ **Common Mistake**: Not handling all key combinations
   ✅ **Solution**: Add catch-all pattern and log unhandled keys

**Testing TUI Components**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_navigation() {
        let mut app = App::test_new();
        app.profiles = vec!["one".into(), "two".into(), "three".into()];
        
        // Test down navigation
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty())).await.unwrap();
        assert_eq!(app.selected_profile, 1);
        
        // Test wrap around
        app.selected_profile = 2;
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty())).await.unwrap();
        assert_eq!(app.selected_profile, 0);
    }
}
```

**Debugging Guide**:

**Issue**: TUI not responding to keys
**Debug**: Add logging to handle_key method, check event loop running

**Issue**: Screen flickers
**Solution**: Ensure you're not clearing screen unnecessarily

**Issue**: Panic on resize
**Solution**: Handle Resize event properly, use safe terminal size

---

### 🛑 CHECKPOINT 3.2: Basic TUI Working

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 40 hours total

**Pre-Checkpoint Checklist**:
- [ ] Navigation between screens works
- [ ] Profile list displays correctly
- [ ] Keyboard shortcuts functional
- [ ] No panics on any input
- [ ] Status messages display

---

## Week 3: Advanced TUI Features

### 3.3 Search and Polish (40 hours)

#### Task 3.3.1: Real-time Search Implementation (12 hours)

💡 **Junior Dev Concept**: Fuzzy Search in TUI
**What it is**: Search that finds matches even with typos
**Why useful**: Users can find profiles quickly
**Pattern**: Filter list as user types

**Prerequisites**:
- [ ] Review: [Advanced Search Patterns](./ADVANCED_RATATUI_CONCEPTS.md#complex-event-handling)
- [ ] Understand: Fuzzy matching algorithms
- [ ] Practice: String filtering in Rust

**Implementation**:

1. **Create Search State** (3 hours)
   ```rust
   // src/tui/search.rs
   
   use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
   
   pub struct SearchState {
       /// Current search query
       query: String,
       /// Fuzzy matcher instance
       matcher: SkimMatcherV2,
       /// Original items
       all_items: Vec<ProfileSummary>,
       /// Filtered results with scores
       results: Vec<(ProfileSummary, i64)>,
       /// Selected result index
       selected: usize,
   }
   
   impl SearchState {
       pub fn new(items: Vec<ProfileSummary>) -> Self {
           Self {
               query: String::new(),
               matcher: SkimMatcherV2::default(),
               all_items: items.clone(),
               results: items.into_iter().map(|i| (i, 0)).collect(),
               selected: 0,
           }
       }
       
       /// Update search query and filter results
       pub fn update_query(&mut self, query: String) {
           self.query = query;
           self.filter_results();
           self.selected = 0;
       }
       
       /// Perform fuzzy search
       fn filter_results(&mut self) {
           if self.query.is_empty() {
               // Show all items when no query
               self.results = self.all_items
                   .iter()
                   .cloned()
                   .map(|i| (i, 0))
                   .collect();
               return;
           }
           
           // Score each item
           let mut scored: Vec<_> = self.all_items
               .iter()
               .filter_map(|item| {
                   // Search in name, email, and description
                   let search_text = format!(
                       "{} {} {}",
                       item.name,
                       item.email,
                       item.description.as_deref().unwrap_or("")
                   );
                   
                   self.matcher
                       .fuzzy_match(&search_text, &self.query)
                       .map(|score| (item.clone(), score))
               })
               .collect();
           
           // Sort by score (highest first)
           scored.sort_by(|a, b| b.1.cmp(&a.1));
           
           self.results = scored;
       }
   }
   ```

2. **Create Search Widget** (4 hours)
   ```rust
   /// Search interface widget
   pub struct SearchWidget<'a> {
       search_state: &'a mut SearchState,
       style: SearchStyle,
   }
   
   impl<'a> SearchWidget<'a> {
       pub fn render(&mut self, f: &mut Frame, area: Rect) {
           let chunks = Layout::default()
               .direction(Direction::Vertical)
               .constraints([
                   Constraint::Length(3),   // Search box
                   Constraint::Length(1),   // Separator
                   Constraint::Min(5),      // Results
                   Constraint::Length(2),   // Help
               ])
               .split(area);
           
           // Render search input
           self.render_search_box(f, chunks[0]);
           
           // Render separator
           let separator = Block::default()
               .borders(Borders::BOTTOM)
               .border_style(Style::default().fg(Color::DarkGray));
           f.render_widget(separator, chunks[1]);
           
           // Render results
           self.render_results(f, chunks[2]);
           
           // Render help
           self.render_help(f, chunks[3]);
       }
       
       fn render_search_box(&self, f: &mut Frame, area: Rect) {
           let input = Paragraph::new(self.search_state.query.as_str())
               .style(Style::default().fg(Color::Yellow))
               .block(
                   Block::default()
                       .borders(Borders::ALL)
                       .title("Search (fuzzy)")
                       .border_style(Style::default().fg(Color::Cyan))
               );
           
           f.render_widget(input, area);
           
           // Show cursor
           f.set_cursor(
               area.x + self.search_state.query.len() as u16 + 1,
               area.y + 1,
           );
       }
       
       fn render_results(&self, f: &mut Frame, area: Rect) {
           let items: Vec<ListItem> = self.search_state.results
               .iter()
               .enumerate()
               .map(|(i, (item, score))| {
                   let content = if self.search_state.query.is_empty() {
                       vec![
                           Line::from(vec![
                               Span::styled(&item.name, Style::default().fg(Color::White)),
                           ]),
                           Line::from(vec![
                               Span::raw("  "),
                               Span::styled(&item.email, Style::default().fg(Color::Gray)),
                           ]),
                       ]
                   } else {
                       // Highlight matched portions
                       let name_highlighted = self.highlight_matches(&item.name);
                       let email_highlighted = self.highlight_matches(&item.email);
                       
                       vec![
                           Line::from(name_highlighted),
                           Line::from(vec![
                               Span::raw("  "),
                               Line::from(email_highlighted),
                               Span::styled(
                                   format!(" (score: {})", score),
                                   Style::default().fg(Color::DarkGray)
                               ),
                           ]),
                       ]
                   };
                   
                   ListItem::new(content)
               })
               .collect();
           
           let list = List::new(items)
               .block(Block::default().borders(Borders::NONE))
               .highlight_style(
                   Style::default()
                       .bg(Color::DarkGray)
                       .add_modifier(Modifier::BOLD)
               )
               .highlight_symbol("▸ ");
           
           let mut state = ListState::default();
           state.select(Some(self.search_state.selected));
           
           f.render_stateful_widget(list, area, &mut state);
       }
       
       /// Highlight matched portions of text
       fn highlight_matches(&self, text: &str) -> Vec<Span> {
           // Simple highlight - in production use fuzzy_matcher indices
           let query_lower = self.search_state.query.to_lowercase();
           let text_lower = text.to_lowercase();
           
           if let Some(start) = text_lower.find(&query_lower) {
               let end = start + query_lower.len();
               vec![
                   Span::raw(&text[..start]),
                   Span::styled(
                       &text[start..end],
                       Style::default()
                           .fg(Color::Yellow)
                           .add_modifier(Modifier::BOLD)
                   ),
                   Span::raw(&text[end..]),
               ]
           } else {
               vec![Span::raw(text)]
           }
       }
   }
   ```

3. **Integration with Main App** (5 hours)
   ```rust
   impl App {
       /// Handle search mode input
       async fn handle_search_mode(&mut self, key: KeyEvent) -> Result<(), TuiError> {
           match key.code {
               KeyCode::Char(c) => {
                   self.search_state.query.push(c);
                   self.search_state.update_query(self.search_state.query.clone());
               }
               KeyCode::Backspace => {
                   self.search_state.query.pop();
                   self.search_state.update_query(self.search_state.query.clone());
               }
               KeyCode::Up => {
                   self.search_state.previous();
               }
               KeyCode::Down => {
                   self.search_state.next();
               }
               KeyCode::Enter => {
                   if let Some(selected) = self.search_state.get_selected() {
                       self.load_profile(selected)?;
                       self.input_mode = InputMode::Normal;
                   }
               }
               KeyCode::Esc => {
                   self.input_mode = InputMode::Normal;
                   self.search_state.clear();
               }
               _ => {}
           }
           Ok(())
       }
   }
   ```

**Testing Search**:
```rust
#[test]
fn test_fuzzy_search() {
    let items = vec![
        ProfileSummary { name: "work".into(), email: "alice@work.com".into(), description: None },
        ProfileSummary { name: "personal".into(), email: "alice@personal.com".into(), description: None },
        ProfileSummary { name: "client-work".into(), email: "alice@client.com".into(), description: None },
    ];
    
    let mut search = SearchState::new(items);
    
    // Test fuzzy matching
    search.update_query("wrk".to_string());
    assert_eq!(search.results.len(), 2); // "work" and "client-work"
    
    // Test email search
    search.update_query("personal.com".to_string());
    assert_eq!(search.results.len(), 1);
}
```

#### Task 3.3.2: Context-Sensitive Help System (8 hours)

💡 **Junior Dev Concept**: Dynamic Help Content
**What it is**: Help that changes based on current screen/mode
**Why important**: Users get relevant help without leaving context
**Implementation**: Help overlay with context detection

See [Modal Dialogs](./ADVANCED_RATATUI_CONCEPTS.md#modal-dialogs) for implementation patterns.

#### Task 3.3.3: Polish and Animations (8 hours)

1. **Loading Indicators** (3 hours)
   ```rust
   pub struct Spinner {
       frames: Vec<&'static str>,
       current: usize,
   }
   
   impl Spinner {
       pub fn new() -> Self {
           Self {
               frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
               current: 0,
           }
       }
       
       pub fn tick(&mut self) {
           self.current = (self.current + 1) % self.frames.len();
       }
       
       pub fn frame(&self) -> &str {
           self.frames[self.current]
       }
   }
   ```

2. **Progress Indicators** (2 hours)
3. **Smooth Transitions** (3 hours)

#### Task 3.3.4: Cross-Platform Testing (12 hours)

**Testing Matrix**:

| Platform | Terminal | Tests | Notes |
|----------|----------|-------|-------|
| Windows | Terminal | ✓ | Handle path display |
| Windows | ConEmu | ✓ | Check Unicode |
| macOS | Terminal.app | ✓ | Test colors |
| macOS | iTerm2 | ✓ | Check true color |
| Linux | GNOME Terminal | ✓ | Standard testing |
| Linux | Alacritty | ✓ | GPU accelerated |
| SSH | Various | ✓ | Latency handling |

**Platform-Specific Fixes**:
```rust
#[cfg(windows)]
fn setup_terminal() -> Result<()> {
    // Enable ANSI support on Windows
    let _ = ansi_term::enable_ansi_support();
    Ok(())
}

#[cfg(not(windows))]
fn setup_terminal() -> Result<()> {
    Ok(())
}
```

---

### 🛑 FINAL CHECKPOINT 3: UI Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Final Deliverables**:
- Complete CLI with all commands
- Full-featured TUI with search
- Context-sensitive help system
- Cross-platform compatibility verified
- Performance optimized (<50ms response)
- Visual polish and animations
- Cross-platform compatibility
- Comprehensive help system
- All tests passing

---

## Summary

Phase 3 transforms git-setup-rs from a library into a usable application. The progressive approach from CLI to basic TUI to advanced features ensures steady progress while maintaining quality.