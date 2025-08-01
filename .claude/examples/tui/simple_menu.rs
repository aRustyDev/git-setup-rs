//! Simple TUI Menu Example
//! 
//! This example demonstrates the basics of creating a Terminal UI with Ratatui.
//! Perfect starting point for junior developers new to TUI programming.
//! 
//! Key concepts:
//! - Terminal setup and cleanup
//! - Event handling (keyboard input)
//! - Basic rendering
//! - State management

use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

/// Application state - what menu item is selected
struct MenuState {
    /// List of menu items
    items: Vec<String>,
    /// Currently selected index
    selected: usize,
}

impl MenuState {
    fn new() -> Self {
        Self {
            items: vec![
                "View Profiles".to_string(),
                "Create Profile".to_string(),
                "Edit Profile".to_string(),
                "Delete Profile".to_string(),
                "Settings".to_string(),
                "Exit".to_string(),
            ],
            selected: 0,
        }
    }
    
    /// Move selection up
    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            // Wrap to bottom
            self.selected = self.items.len() - 1;
        }
    }
    
    /// Move selection down
    fn next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
        } else {
            // Wrap to top
            self.selected = 0;
        }
    }
    
    /// Get the currently selected item
    fn current_item(&self) -> &str {
        &self.items[self.selected]
    }
}

/// Main TUI application
fn main() -> Result<(), io::Error> {
    // Step 1: Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    // Create backend and terminal
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Step 2: Create app state
    let mut state = MenuState::new();
    let mut should_quit = false;
    let mut message = String::new();
    
    // Step 3: Main loop
    while !should_quit {
        // Draw UI
        terminal.draw(|frame| draw_ui(frame, &state, &message))?;
        
        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        should_quit = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.previous();
                        message.clear();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.next();
                        message.clear();
                    }
                    KeyCode::Enter => {
                        message = format!("Selected: {}", state.current_item());
                        if state.current_item() == "Exit" {
                            should_quit = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Step 4: Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    Ok(())
}

/// Draw the user interface
fn draw_ui(frame: &mut Frame, state: &MenuState, message: &str) {
    // Create layout with 3 sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),      // Title
            Constraint::Min(10),        // Menu
            Constraint::Length(3),      // Status
        ])
        .split(frame.size());
    
    // Title
    let title = Paragraph::new("Git Setup - Main Menu")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);
    
    // Menu items
    let items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(item.as_str()).style(style)
        })
        .collect();
    
    let menu = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Use ↑/↓ or j/k to navigate, Enter to select"),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        );
    frame.render_widget(menu, chunks[1]);
    
    // Status bar
    let status_text = if message.is_empty() {
        "Press 'q' or Esc to quit"
    } else {
        message
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, chunks[2]);
}

/// Junior Developer Notes:
/// 
/// 1. **Terminal Raw Mode**:
///    - Raw mode = keyboard input goes directly to your app
///    - Normal mode = terminal handles special keys (Ctrl+C, etc)
///    - ALWAYS restore normal mode on exit!
/// 
/// 2. **Event Loop Pattern**:
///    ```
///    loop {
///        draw();     // Render current state
///        input();    // Handle user input
///        update();   // Update state based on input
///    }
///    ```
/// 
/// 3. **State Management**:
///    - Keep UI state separate from rendering
///    - State changes trigger redraws
///    - Immutable state makes debugging easier
/// 
/// 4. **Error Handling**:
///    - Use ? operator for cleaner error propagation
///    - Always cleanup terminal on error
///    - Consider using panic handler for cleanup
/// 
/// Exercise: Add these features:
/// 1. Add a "Help" menu item
/// 2. Show item number next to each menu item
/// 3. Add vi-style navigation (h/l for back/forward)
/// 
/// Hint: Modify the MenuState and draw_ui function