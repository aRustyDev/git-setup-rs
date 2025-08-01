# Advanced Ratatui Concepts for Phase 3

## Overview

This document provides in-depth explanations of advanced Ratatui concepts needed for Phase 3's TUI implementation. Each concept includes visual diagrams, practical examples, and references to official documentation.

## Table of Contents

1. [Layout Management and Constraints](#layout-management)
2. [Custom Widget Creation](#custom-widgets)
3. [Stateful Widgets](#stateful-widgets)  
4. [Rendering Optimization](#rendering-optimization)
5. [Complex Event Handling](#event-handling)
6. [Modal Dialogs](#modal-dialogs)
7. [Form Validation in TUI](#form-validation)
8. [Visual Mockups](#visual-mockups)

## Layout Management and Constraints {#layout-management}

### 💡 Junior Dev Concept: Ratatui Layout System

**What it is**: A constraint-based system for dividing terminal space into regions
**Why complex**: Terminal size changes dynamically, layouts must be responsive
**Key insight**: Think of it like CSS Flexbox for terminals

### Visual Layout Concepts

```
┌─────────────────── Terminal (80x24) ──────────────────┐
│                                                        │
│  ┌─── Layout::horizontal([30%, 70%]) ───┐            │
│  │                                       │            │
│  │  ┌─ Left: 30% ─┐  ┌─ Right: 70% ───┐│            │
│  │  │              │  │                 ││            │
│  │  │  Profile     │  │  Details       ││            │
│  │  │  List        │  │  View          ││            │
│  │  │              │  │                 ││            │
│  │  └──────────────┘  └─────────────────┘│            │
│  └────────────────────────────────────────┘            │
│                                                        │
└────────────────────────────────────────────────────────┘
```

### Implementation Example: Responsive Layout

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Advanced layout that adapts to terminal size
pub fn create_adaptive_layout(area: Rect) -> (Rect, Rect, Rect) {
    // Responsive breakpoints
    let use_three_column = area.width >= 120;
    let use_two_column = area.width >= 80;
    
    if use_three_column {
        // Wide screen: 3-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25),      // Fixed sidebar
                Constraint::Min(40),         // Main content
                Constraint::Length(30),      // Right panel
            ])
            .split(area);
        (chunks[0], chunks[1], chunks[2])
    } else if use_two_column {
        // Medium screen: 2-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),  // Sidebar
                Constraint::Percentage(70),  // Main
            ])
            .split(area);
        (chunks[0], chunks[1], Rect::default())
    } else {
        // Narrow screen: Stacked layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),      // Top
                Constraint::Min(5),          // Main
            ])
            .split(area);
        (chunks[0], chunks[1], Rect::default())
    }
}
```

### Complex Layout Pattern: Nested Splits

```rust
/// Create a complex dashboard layout
pub fn create_dashboard_layout(area: Rect) -> DashboardLayout {
    // Main vertical split
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Body
            Constraint::Length(3),   // Status bar
        ])
        .split(area);
    
    // Split body horizontally
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // Sidebar
            Constraint::Percentage(75),  // Content
        ])
        .split(main_chunks[1]);
    
    // Split content area for details
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Main view
            Constraint::Percentage(40),  // Details
        ])
        .split(body_chunks[1]);
    
    DashboardLayout {
        header: main_chunks[0],
        sidebar: body_chunks[0],
        main_content: content_chunks[0],
        details: content_chunks[1],
        status_bar: main_chunks[2],
    }
}
```

**Common Pitfalls & Solutions**:
- **Pitfall**: Fixed pixel layouts break on resize
- **Solution**: Use Percentage and Min constraints
- **Pitfall**: Nested layouts causing overflow
- **Solution**: Always leave Min constraint as last

## Custom Widget Creation {#custom-widgets}

### 💡 Junior Dev Concept: Building Custom Widgets

**What it is**: Creating reusable UI components beyond built-in widgets
**Why needed**: Complex UIs need specialized components
**Key pattern**: Implement the `Widget` trait

### Example: Profile Card Widget

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

/// Custom widget for displaying a profile card
pub struct ProfileCard<'a> {
    profile: &'a Profile,
    selected: bool,
    block: Option<Block<'a>>,
}

impl<'a> ProfileCard<'a> {
    pub fn new(profile: &'a Profile) -> Self {
        Self {
            profile,
            selected: false,
            block: None,
        }
    }
    
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for ProfileCard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render block if provided
        let inner = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };
        
        // Calculate layout
        if inner.height < 4 {
            return; // Not enough space
        }
        
        // Style based on selection
        let style = if self.selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        // Render profile name
        let name_line = Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.profile.name, style),
        ]);
        buf.set_line(inner.x, inner.y, &name_line, inner.width);
        
        // Render email
        let email_line = Line::from(vec![
            Span::styled("Email: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.profile.git.user_email, style),
        ]);
        buf.set_line(inner.x, inner.y + 1, &email_line, inner.width);
        
        // Render signing status
        let signing_status = if self.profile.signing.is_some() {
            "✓ Signing configured"
        } else {
            "✗ No signing"
        };
        let signing_color = if self.profile.signing.is_some() {
            Color::Green
        } else {
            Color::Red
        };
        
        let signing_line = Line::from(vec![
            Span::styled(signing_status, Style::default().fg(signing_color)),
        ]);
        buf.set_line(inner.x, inner.y + 3, &signing_line, inner.width);
    }
}
```

### Visual Custom Widget Example

```
┌─ Profile Card Widget ──────────────┐
│                                    │
│ Name: work-profile                 │
│ Email: alice@company.com           │
│                                    │
│ ✓ Signing configured               │
│ SSH + GPG enabled                  │
│                                    │
│ Last used: 2 hours ago             │
└────────────────────────────────────┘
```

## Stateful Widgets {#stateful-widgets}

### 💡 Junior Dev Concept: Widgets with State

**What it is**: Widgets that maintain internal state between renders
**Why complex**: State must be stored separately from widget
**Pattern**: Widget + WidgetState separation

### Example: Scrollable Profile List

```rust
use ratatui::widgets::{List, ListState};

/// Stateful profile list manager
pub struct ProfileListWidget {
    items: Vec<String>,
    state: ListState,
}

impl ProfileListWidget {
    pub fn new(items: Vec<String>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        
        Self { items, state }
    }
    
    /// Navigate to next item with wraparound
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    /// Navigate to previous item with wraparound
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    /// Jump to item by first letter
    pub fn jump_to_letter(&mut self, letter: char) {
        let letter = letter.to_lowercase().to_string();
        
        // Find first item starting with letter
        let position = self.items.iter().position(|item| {
            item.to_lowercase().starts_with(&letter)
        });
        
        if let Some(pos) = position {
            self.state.select(Some(pos));
        }
    }
    
    /// Get the currently selected item
    pub fn selected(&self) -> Option<&String> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
    
    /// Render the widget
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let list = List::new(self.items.clone())
            .block(Block::default().borders(Borders::ALL).title("Profiles"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(list, area, &mut self.state);
    }
}
```

### State Management Pattern

```rust
/// Application state manager
pub struct AppState {
    profile_list: ProfileListWidget,
    detail_view: ProfileDetailState,
    form_state: FormState,
    help_visible: bool,
}

impl AppState {
    /// Handle keyboard input based on current screen
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.help_visible {
            self.handle_help_key(key)
        } else {
            match self.current_screen {
                Screen::Main => self.handle_main_key(key),
                Screen::ProfileEdit => self.handle_edit_key(key),
                _ => Ok(()),
            }
        }
    }
    
    fn handle_main_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.profile_list.previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.profile_list.next();
            }
            KeyCode::Char(c) if c.is_alphabetic() => {
                self.profile_list.jump_to_letter(c);
            }
            KeyCode::Enter => {
                if let Some(profile) = self.profile_list.selected() {
                    self.load_profile_details(profile)?;
                    self.current_screen = Screen::ProfileView;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Rendering Optimization {#rendering-optimization}

### 💡 Junior Dev Concept: Efficient TUI Rendering

**What it is**: Techniques to minimize terminal updates and improve performance
**Why important**: Slow rendering = bad user experience
**Key insight**: Only redraw what changed

### Optimization Techniques

```rust
/// Efficient rendering with dirty tracking
pub struct OptimizedApp {
    /// Track what needs redrawing
    dirty_regions: DirtyRegions,
    /// Cache rendered content
    render_cache: RenderCache,
    /// Previous terminal size
    last_size: (u16, u16),
}

#[derive(Default)]
struct DirtyRegions {
    header: bool,
    sidebar: bool,
    content: bool,
    status: bool,
}

impl OptimizedApp {
    /// Mark region as needing redraw
    pub fn invalidate_region(&mut self, region: Region) {
        match region {
            Region::Header => self.dirty_regions.header = true,
            Region::Sidebar => self.dirty_regions.sidebar = true,
            Region::Content => self.dirty_regions.content = true,
            Region::Status => self.dirty_regions.status = true,
            Region::All => {
                self.dirty_regions = DirtyRegions {
                    header: true,
                    sidebar: true,
                    content: true,
                    status: true,
                };
            }
        }
    }
    
    /// Smart render - only update dirty regions
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<()> {
        let size = f.size();
        
        // Check if terminal was resized
        if (size.width, size.height) != self.last_size {
            self.invalidate_region(Region::All);
            self.last_size = (size.width, size.height);
        }
        
        // Create layout
        let layout = create_dashboard_layout(size);
        
        // Only render dirty regions
        if self.dirty_regions.header {
            self.render_header(f, layout.header);
            self.dirty_regions.header = false;
        }
        
        if self.dirty_regions.sidebar {
            self.render_sidebar(f, layout.sidebar);
            self.dirty_regions.sidebar = false;
        }
        
        if self.dirty_regions.content {
            self.render_content(f, layout.main_content);
            self.dirty_regions.content = false;
        }
        
        if self.dirty_regions.status {
            self.render_status(f, layout.status_bar);
            self.dirty_regions.status = false;
        }
        
        Ok(())
    }
}
```

### Performance Tips Visual Guide

```
┌─ Rendering Performance Tips ──────────────────────┐
│                                                   │
│ ❌ BAD: Redraw everything on every frame         │
│ ┌─────────────────────────────────────────┐      │
│ │ Header (redrawn)                        │      │
│ │ List (redrawn)    Content (redrawn)     │      │
│ │ Status (redrawn)                        │      │
│ └─────────────────────────────────────────┘      │
│                                                   │
│ ✅ GOOD: Only redraw what changed                │
│ ┌─────────────────────────────────────────┐      │
│ │ Header (cached)                         │      │
│ │ List (redrawn) ← Content (cached)       │      │
│ │ Status (cached)                         │      │
│ └─────────────────────────────────────────┘      │
└───────────────────────────────────────────────────┘
```

## Complex Event Handling {#event-handling}

### 💡 Junior Dev Concept: Advanced Event Patterns

**What it is**: Handling complex user interactions beyond simple key presses
**Examples**: Chord keys, mouse support, paste handling
**Pattern**: Event routing and bubbling

### Advanced Event Handler

```rust
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

/// Advanced event handling with context
pub struct EventRouter {
    /// Global hotkeys that work everywhere
    global_hotkeys: HashMap<KeyCombo, Action>,
    /// Context-specific handlers
    context_handlers: HashMap<Screen, Box<dyn EventHandler>>,
    /// Event history for combos
    event_history: VecDeque<TimedEvent>,
}

#[derive(Hash, Eq, PartialEq)]
struct KeyCombo {
    modifiers: KeyModifiers,
    code: KeyCode,
}

struct TimedEvent {
    event: Event,
    timestamp: Instant,
}

impl EventRouter {
    pub fn new() -> Self {
        let mut global_hotkeys = HashMap::new();
        
        // Register global hotkeys
        global_hotkeys.insert(
            KeyCombo {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('q'),
            },
            Action::Quit,
        );
        
        global_hotkeys.insert(
            KeyCombo {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('s'),
            },
            Action::Save,
        );
        
        Self {
            global_hotkeys,
            context_handlers: HashMap::new(),
            event_history: VecDeque::with_capacity(10),
        }
    }
    
    /// Route event to appropriate handler
    pub async fn handle_event(
        &mut self,
        event: Event,
        current_screen: Screen,
        app_state: &mut AppState,
    ) -> Result<Option<Action>> {
        // Record event for combo detection
        self.event_history.push_back(TimedEvent {
            event: event.clone(),
            timestamp: Instant::now(),
        });
        
        // Clean old events (>1 second)
        let cutoff = Instant::now() - Duration::from_secs(1);
        self.event_history.retain(|e| e.timestamp > cutoff);
        
        match event {
            Event::Key(key) => {
                // Check global hotkeys first
                let combo = KeyCombo {
                    modifiers: key.modifiers,
                    code: key.code,
                };
                
                if let Some(action) = self.global_hotkeys.get(&combo) {
                    return Ok(Some(action.clone()));
                }
                
                // Check for key sequences (like vim's gg)
                if let Some(action) = self.check_key_sequence() {
                    return Ok(Some(action));
                }
                
                // Route to context handler
                if let Some(handler) = self.context_handlers.get_mut(&current_screen) {
                    handler.handle_key(key, app_state).await
                } else {
                    Ok(None)
                }
            }
            
            Event::Mouse(mouse) => {
                self.handle_mouse(mouse, current_screen, app_state).await
            }
            
            Event::Paste(text) => {
                self.handle_paste(text, current_screen, app_state).await
            }
            
            _ => Ok(None),
        }
    }
    
    /// Detect key sequences like 'gg' for top, 'G' for bottom
    fn check_key_sequence(&self) -> Option<Action> {
        let recent_keys: Vec<_> = self.event_history
            .iter()
            .rev()
            .take(2)
            .filter_map(|e| {
                if let Event::Key(key) = &e.event {
                    Some(key.code)
                } else {
                    None
                }
            })
            .collect();
        
        match recent_keys.as_slice() {
            [KeyCode::Char('g'), KeyCode::Char('g')] => Some(Action::GoToTop),
            [KeyCode::Char('G')] => Some(Action::GoToBottom),
            [KeyCode::Char('d'), KeyCode::Char('d')] => Some(Action::DeleteLine),
            _ => None,
        }
    }
}
```

### Visual Event Flow

```
┌─ Event Flow Diagram ─────────────────────────────┐
│                                                  │
│  User Input                                      │
│      ↓                                          │
│  ┌─────────────┐                               │
│  │Event Capture│                               │
│  └──────┬──────┘                               │
│         ↓                                       │
│  ┌──────────────┐     ┌─────────────┐         │
│  │Global Hotkeys│ ──> │  Execute    │         │
│  └──────┬───────┘     │  Action     │         │
│         ↓ (no match)  └─────────────┘         │
│  ┌──────────────┐                             │
│  │Key Sequences │                             │
│  └──────┬───────┘                             │
│         ↓ (no match)                          │
│  ┌──────────────┐                             │
│  │Context Handler│                            │
│  └──────┬───────┘                             │
│         ↓                                      │
│  ┌──────────────┐                             │
│  │Update State  │                             │
│  └──────────────┘                             │
└──────────────────────────────────────────────────┘
```

## Modal Dialogs {#modal-dialogs}

### 💡 Junior Dev Concept: Modal Dialog Implementation

**What it is**: Popup dialogs that capture all input until dismissed
**Why tricky**: Must handle rendering order and input capture
**Pattern**: Overlay rendering with input interception

### Modal Dialog Implementation

```rust
/// Modal dialog system
pub struct ModalManager {
    active_modal: Option<Box<dyn Modal>>,
    backdrop_style: Style,
}

pub trait Modal {
    /// Render the modal
    fn render(&self, f: &mut Frame, area: Rect);
    
    /// Handle input (returns true if handled)
    fn handle_input(&mut self, key: KeyEvent) -> ModalResult;
    
    /// Get ideal size for modal
    fn ideal_size(&self) -> (u16, u16);
}

pub enum ModalResult {
    /// Modal handled the input
    Handled,
    /// Modal completed with result
    Completed(ModalAction),
    /// Modal cancelled
    Cancelled,
}

/// Confirmation dialog modal
pub struct ConfirmModal {
    title: String,
    message: String,
    selected_button: usize,
    buttons: Vec<String>,
}

impl Modal for ConfirmModal {
    fn render(&self, f: &mut Frame, area: Rect) {
        // Calculate centered position
        let (width, height) = self.ideal_size();
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        
        let modal_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: width.min(area.width),
            height: height.min(area.height),
        };
        
        // Draw shadow
        let shadow_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width,
            height: modal_area.height,
        };
        
        f.render_widget(
            Block::default()
                .style(Style::default().bg(Color::Black)),
            shadow_area,
        );
        
        // Draw modal
        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(Color::DarkGray));
        
        let inner = block.inner(modal_area);
        f.render_widget(block, modal_area);
        
        // Layout for content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),      // Message
                Constraint::Length(3),   // Buttons
            ])
            .split(inner);
        
        // Render message
        let message = Paragraph::new(self.message.clone())
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        f.render_widget(message, chunks[0]);
        
        // Render buttons
        let button_width = 10;
        let total_width = self.buttons.len() as u16 * (button_width + 2);
        let start_x = (chunks[1].width.saturating_sub(total_width)) / 2;
        
        for (i, button) in self.buttons.iter().enumerate() {
            let button_area = Rect {
                x: chunks[1].x + start_x + (i as u16 * (button_width + 2)),
                y: chunks[1].y,
                width: button_width,
                height: 3,
            };
            
            let style = if i == self.selected_button {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
            };
            
            let button_widget = Paragraph::new(button.clone())
                .style(style)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            
            f.render_widget(button_widget, button_area);
        }
    }
    
    fn handle_input(&mut self, key: KeyEvent) -> ModalResult {
        match key.code {
            KeyCode::Left => {
                if self.selected_button > 0 {
                    self.selected_button -= 1;
                }
                ModalResult::Handled
            }
            KeyCode::Right => {
                if self.selected_button < self.buttons.len() - 1 {
                    self.selected_button += 1;
                }
                ModalResult::Handled
            }
            KeyCode::Enter => {
                if self.selected_button == 0 {
                    ModalResult::Completed(ModalAction::Confirm)
                } else {
                    ModalResult::Cancelled
                }
            }
            KeyCode::Esc => ModalResult::Cancelled,
            _ => ModalResult::Handled,
        }
    }
    
    fn ideal_size(&self) -> (u16, u16) {
        let width = 50.max(self.message.len() as u16 + 4);
        let height = 10;
        (width, height)
    }
}

/// Render modals on top of regular content
impl ModalManager {
    pub fn render_modal<B: Backend>(&self, f: &mut Frame<B>) {
        if let Some(modal) = &self.active_modal {
            // Render backdrop
            let backdrop = Block::default()
                .style(self.backdrop_style);
            f.render_widget(backdrop, f.size());
            
            // Render modal
            modal.render(f, f.size());
        }
    }
}
```

### Visual Modal Examples

```
┌────────────────────────────────────────────────┐
│                                                │
│    ┌─── Confirm Delete ──────────────┐        │
│    │                                 │▒       │
│    │  Are you sure you want to      │▒       │
│    │  delete the profile "work"?    │▒       │
│    │                                 │▒       │
│    │  ┌─────────┐   ┌─────────┐    │▒       │
│    │  │   Yes   │   │   No    │    │▒       │
│    │  └─────────┘   └─────────┘    │▒       │
│    └─────────────────────────────────┘▒       │
│     ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒        │
│                                                │
└────────────────────────────────────────────────┘

┌─ Form Modal Example ───────────────────────────┐
│                                                │
│    ┌─── Create New Profile ─────────┐         │
│    │                                │▒        │
│    │  Name:     [________________] │▒        │
│    │                                │▒        │
│    │  Email:    [________________] │▒        │
│    │                                │▒        │
│    │  Git User: [________________] │▒        │
│    │                                │▒        │
│    │  ┌────────┐   ┌──────────┐   │▒        │
│    │  │ Create │   │  Cancel  │   │▒        │
│    │  └────────┘   └──────────┘   │▒        │
│    └────────────────────────────────┘▒        │
│     ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒         │
└────────────────────────────────────────────────┘
```

## Form Validation in TUI {#form-validation}

### 💡 Junior Dev Concept: Real-time Form Validation

**What it is**: Validating user input as they type in TUI forms
**Why complex**: Must provide immediate feedback without blocking
**Pattern**: Incremental validation with visual feedback

### Form Validation Implementation

```rust
/// Form field with validation
pub struct ValidatedField {
    label: String,
    value: String,
    validator: Box<dyn Validator>,
    error: Option<String>,
    touched: bool,
}

pub trait Validator: Send + Sync {
    fn validate(&self, value: &str) -> Result<(), String>;
    fn validate_partial(&self, value: &str) -> ValidationHint;
}

pub enum ValidationHint {
    Valid,
    Invalid(String),
    Incomplete,
}

/// Email validator with real-time hints
pub struct EmailValidator;

impl Validator for EmailValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err("Email is required".to_string());
        }
        
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();
        
        if email_regex.is_match(value) {
            Ok(())
        } else {
            Err("Invalid email format".to_string())
        }
    }
    
    fn validate_partial(&self, value: &str) -> ValidationHint {
        if value.is_empty() {
            return ValidationHint::Incomplete;
        }
        
        // Progressive validation
        if !value.contains('@') {
            if value.len() > 20 {
                ValidationHint::Invalid("Missing @ symbol".to_string())
            } else {
                ValidationHint::Incomplete
            }
        } else if value.ends_with('@') {
            ValidationHint::Incomplete
        } else if !value.contains('.') {
            ValidationHint::Incomplete
        } else {
            match self.validate(value) {
                Ok(_) => ValidationHint::Valid,
                Err(e) => ValidationHint::Invalid(e),
            }
        }
    }
}

/// Form widget with multiple fields
pub struct FormWidget {
    fields: Vec<ValidatedField>,
    focused_field: usize,
    submit_enabled: bool,
}

impl FormWidget {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let field_height = 4; // Label + input + error + spacing
        let fields_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                self.fields
                    .iter()
                    .map(|_| Constraint::Length(field_height))
                    .collect::<Vec<_>>(),
            )
            .split(area);
        
        for (i, (field, field_area)) in self.fields.iter().zip(fields_area.iter()).enumerate() {
            self.render_field(f, field, *field_area, i == self.focused_field);
        }
    }
    
    fn render_field(&self, f: &mut Frame, field: &ValidatedField, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Label
                Constraint::Length(3), // Input
            ])
            .split(area);
        
        // Render label
        let label_style = if focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let label = Paragraph::new(format!("{}:", field.label))
            .style(label_style);
        f.render_widget(label, chunks[0]);
        
        // Determine field style based on validation
        let (border_color, helper_text) = if !field.touched {
            (Color::Gray, None)
        } else if let Some(error) = &field.error {
            (Color::Red, Some(error.clone()))
        } else {
            (Color::Green, Some("✓".to_string()))
        };
        
        // Render input field
        let input = Paragraph::new(field.value.clone())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(if let Some(helper) = helper_text {
                        helper
                    } else {
                        String::new()
                    }),
            );
        
        f.render_widget(input, chunks[1]);
        
        // Show cursor if focused
        if focused {
            f.set_cursor(
                chunks[1].x + field.value.len() as u16 + 1,
                chunks[1].y + 1,
            );
        }
    }
}
```

### Visual Form Validation States

```
┌─ Form Validation States ───────────────────────┐
│                                                │
│ Empty (untouched):                             │
│ Email:                                         │
│ ┌────────────────────────────────┐            │
│ │                                │            │
│ └────────────────────────────────┘            │
│                                                │
│ Typing (incomplete):                           │
│ Email:                                         │
│ ┌────────────────────────────────┐            │
│ │alice@                          │            │
│ └────────────────────────────────┘            │
│                                                │
│ Invalid (touched):                             │
│ Email:                                         │
│ ┌─ Invalid email format ─────────┐            │
│ │alice@invalid                   │            │
│ └────────────────────────────────┘            │
│                                                │
│ Valid:                                         │
│ Email:                                         │
│ ┌─ ✓ ────────────────────────────┐            │
│ │alice@example.com               │            │
│ └────────────────────────────────┘            │
└────────────────────────────────────────────────┘
```

## Visual Mockups {#visual-mockups}

### Main Application Screen

```
┌─ git-setup-rs ─────────────────────────────────────────────────┐
│ File  Edit  Profile  Help                            v1.0.0    │
├─────────────┬───────────────────────────────────────┬──────────┤
│ Profiles    │ Profile Details: work                 │ Actions  │
│             │                                       │          │
│ > work      │ Name: work                           │ [Apply]  │
│   personal  │ Email: alice@company.com            │ [Edit]   │
│   client-a  │ User: Alice Smith                   │ [Delete] │
│   client-b  │                                     │          │
│             │ Signing:                            │ Quick    │
│ [+] New     │   Method: SSH                       │ Actions: │
│ [/] Search  │   Key: ~/.ssh/id_ed25519.pub       │          │
│ [?] Help    │   ✓ Sign commits                    │ a: Apply │
│             │   ✓ Sign tags                       │ e: Edit  │
│             │                                     │ d: Delete│
│             │ Remote Patterns:                    │ n: New   │
│             │   github.com/company/*             │ q: Quit  │
│             │   gitlab.company.com/*             │          │
├─────────────┴───────────────────────────────────────┴──────────┤
│ Ready | Profile 1/4 | SSH signing configured                   │
└─────────────────────────────────────────────────────────────────┘
```

### Profile Creation Wizard

```
┌─ Create New Profile (Step 3/6) ────────────────────────────────┐
│                                                                │
│                    Git Configuration                           │
│                                                                │
│  ┌─ Progress ─────────────────────────────────────────┐       │
│  │ ✓ Profile Name  ✓ Description  ● Git Config       │       │
│  │ ○ Signing       ○ Remotes      ○ Review           │       │
│  └───────────────────────────────────────────────────┘       │
│                                                                │
│  Git User Name:                                               │
│  ┌────────────────────────────────────────────────┐          │
│  │Alice Smith                                      │          │
│  └────────────────────────────────────────────────┘          │
│                                                                │
│  Git User Email:                                              │
│  ┌─ ✓ Valid ──────────────────────────────────────┐          │
│  │alice@company.com                               │          │
│  └────────────────────────────────────────────────┘          │
│                                                                │
│  Additional Git Config (optional):                            │
│  ┌────────────────────────────────────────────────┐          │
│  │core.editor = "vim"                             │          │
│  │pull.rebase = true                              │          │
│  │                                                 │          │
│  └────────────────────────────────────────────────┘          │
│                                                                │
│  Tip: These settings will be applied when you use            │
│       this profile in a Git repository.                       │
│                                                                │
│  [← Previous]  [Next →]  [Cancel]                            │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Search Interface

```
┌─ Search Profiles ───────────────────────────────────────────────┐
│                                                                │
│  Search: [work_________________]                              │
│                                                                │
│  Results (2 matches):                                         │
│  ┌──────────────────────────────────────────────────┐        │
│  │ ► work                                           │        │
│  │   Email: alice@company.com                       │        │
│  │   Last used: 2 hours ago                         │        │
│  │                                                   │        │
│  │ ► work-legacy                                    │        │
│  │   Email: alice@oldcompany.com                    │        │
│  │   Last used: 3 months ago                        │        │
│  └──────────────────────────────────────────────────┘        │
│                                                                │
│  Navigation: ↑↓ Select  Enter: Open  Esc: Cancel             │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Help Screen

```
┌─ Help ──────────────────────────────────────────────────────────┐
│                                                                │
│  Navigation                    Profile Management              │
│  ─────────────────────────    ──────────────────────────     │
│  ↑/k     Move up              n       Create new profile      │
│  ↓/j     Move down            e       Edit selected           │
│  Enter   Select/Open          d       Delete selected         │
│  Esc     Go back              a       Apply to repository     │
│  Tab     Next section         Space   Toggle selection        │
│                                                                │
│  Search & Filter              Global Commands                  │
│  ─────────────────────────    ──────────────────────────     │
│  /       Start search         Ctrl+s  Save changes            │
│  n       Next match           Ctrl+q  Quit application        │
│  N       Previous match       Ctrl+z  Undo last action        │
│  Esc     Clear search         F1      Show this help          │
│                                                                │
│  Quick Jump                   View Options                     │
│  ─────────────────────────    ──────────────────────────     │
│  gg      Go to top            v       Toggle details view     │
│  G       Go to bottom         l       Show logs               │
│  [a-z]   Jump to profile      r       Refresh list           │
│                                                                │
│  Press any key to return...                                   │
└────────────────────────────────────────────────────────────────┘
```

## References and Resources

### Official Documentation
- [Ratatui Book](https://ratatui.rs/) - Comprehensive guide
- [Ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples) - Working examples
- [Crossterm Events](https://docs.rs/crossterm/latest/crossterm/event/) - Event handling

### Recommended Learning Path
1. Start with basic widgets (List, Paragraph)
2. Learn layout system thoroughly
3. Implement stateful widgets
4. Add keyboard navigation
5. Implement forms with validation
6. Add modal dialogs
7. Optimize rendering

### Common Pitfalls to Avoid
1. **Not handling terminal resize** - Always test with different terminal sizes
2. **Blocking in render** - Never do I/O in render functions
3. **Not clearing old content** - Use Clear widget when needed
4. **Ignoring accessibility** - Provide keyboard navigation for everything
5. **Complex layouts in small terminals** - Always have a minimum size fallback

### Performance Checklist
- [ ] Only redraw changed regions
- [ ] Cache computed layouts
- [ ] Avoid allocations in hot paths
- [ ] Profile with `cargo flamegraph`
- [ ] Test on slow terminals (SSH)
- [ ] Minimize string allocations

This completes the advanced Ratatui concepts documentation for Phase 3. Each concept includes practical examples, visual diagrams, and references to help junior developers successfully implement the TUI features.