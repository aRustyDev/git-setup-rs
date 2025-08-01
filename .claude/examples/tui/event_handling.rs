//! Advanced Event Handling in TUI
//! 
//! This example shows how to handle complex keyboard input, including:
//! - Key combinations (Ctrl+, Alt+)
//! - Input modes (normal vs insert)
//! - Event timeouts and non-blocking input
//! - Custom keybindings

use std::io;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

/// Different input modes (like Vim)
#[derive(Debug, Clone, Copy, PartialEq)]
enum InputMode {
    Normal,
    Insert,
    Command,
}

/// Application state with input handling
struct App {
    /// Current input mode
    mode: InputMode,
    /// Text being edited
    input: String,
    /// Command buffer (for : commands)
    command: String,
    /// Status messages
    messages: Vec<String>,
    /// Should quit?
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            mode: InputMode::Normal,
            input: String::new(),
            command: String::new(),
            messages: vec!["Welcome! Press 'i' to enter insert mode.".to_string()],
            should_quit: false,
        }
    }
    
    /// Add a status message
    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        // Keep only last 5 messages
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }
    
    /// Handle key events based on current mode
    fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Insert => self.handle_insert_mode(key),
            InputMode::Command => self.handle_command_mode(key),
        }
    }
    
    /// Handle keys in normal mode (like Vim)
    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            // Basic navigation
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.mode = InputMode::Insert;
                self.add_message("-- INSERT --".to_string());
            }
            (KeyCode::Char(':'), KeyModifiers::NONE) => {
                self.mode = InputMode::Command;
                self.command.clear();
                self.add_message("Command mode".to_string());
            }
            
            // Quit commands
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.should_quit = true;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.add_message("Use :q to quit".to_string());
            }
            
            // Clear with Ctrl+L
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                self.messages.clear();
                self.add_message("Screen cleared".to_string());
            }
            
            // Delete commands
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if !self.input.is_empty() {
                    self.input.clear();
                    self.add_message("Text cleared".to_string());
                }
            }
            
            _ => {
                self.add_message(format!("Unknown command: {:?}", key));
            }
        }
    }
    
    /// Handle keys in insert mode
    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = InputMode::Normal;
                self.add_message("-- NORMAL --".to_string());
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                self.input.push('\n');
            }
            _ => {}
        }
    }
    
    /// Handle command mode (like Vim's : commands)
    fn handle_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = InputMode::Normal;
                self.command.clear();
                self.add_message("Command cancelled".to_string());
            }
            KeyCode::Enter => {
                self.execute_command();
                self.mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.command.push(c);
            }
            KeyCode::Backspace => {
                self.command.pop();
            }
            _ => {}
        }
    }
    
    /// Execute a : command
    fn execute_command(&mut self) {
        let cmd = self.command.trim();
        
        match cmd {
            "q" | "quit" => self.should_quit = true,
            "w" | "write" => self.add_message("Would save file here".to_string()),
            "wq" => {
                self.add_message("Would save and quit".to_string());
                self.should_quit = true;
            }
            "clear" => {
                self.input.clear();
                self.add_message("Buffer cleared".to_string());
            }
            _ => {
                self.add_message(format!("Unknown command: {}", cmd));
            }
        }
        
        self.command.clear();
    }
}

fn main() -> Result<(), io::Error> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app
    let mut app = App::new();
    
    // Main event loop
    loop {
        // Draw UI
        terminal.draw(|f| draw_ui(f, &app))?;
        
        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key(key);
                }
                Event::Mouse(_) => {
                    app.add_message("Mouse events not supported".to_string());
                }
                Event::Resize(width, height) => {
                    app.add_message(format!("Resized to {}x{}", width, height));
                }
                _ => {}
            }
        }
        
        if app.should_quit {
            break;
        }
    }
    
    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    Ok(())
}

fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),      // Mode indicator
            Constraint::Min(10),        // Main content
            Constraint::Length(7),      // Status messages
            Constraint::Length(3),      // Command line
        ])
        .split(f.size());
    
    // Mode indicator
    let mode_color = match app.mode {
        InputMode::Normal => Color::Blue,
        InputMode::Insert => Color::Green,
        InputMode::Command => Color::Yellow,
    };
    
    let mode_text = format!(" {} MODE ", format!("{:?}", app.mode).to_uppercase());
    let mode = Paragraph::new(mode_text)
        .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(mode, chunks[0]);
    
    // Main content area
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Content"))
        .wrap(Wrap { trim: false });
    f.render_widget(input, chunks[1]);
    
    // Status messages
    let messages: Vec<Line> = app.messages
        .iter()
        .map(|m| Line::from(m.as_str()))
        .collect();
    let msg_widget = Paragraph::new(messages)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(msg_widget, chunks[2]);
    
    // Command line
    let command_text = match app.mode {
        InputMode::Command => format!(":{}", app.command),
        InputMode::Normal => "Press ':' for command, 'i' for insert".to_string(),
        InputMode::Insert => "Press 'Esc' to return to normal mode".to_string(),
    };
    let command = Paragraph::new(command_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(command, chunks[3]);
}

/// Junior Developer Learning Points:
/// 
/// 1. **Modal Interfaces** (like Vim):
///    - Different modes handle keys differently
///    - Mode indicator helps users understand state
///    - Always provide escape route to normal mode
/// 
/// 2. **Key Modifiers**:
///    - Ctrl+key: System commands
///    - Alt+key: Alternative actions  
///    - Shift+key: Usually uppercase (handled automatically)
/// 
/// 3. **Event Types**:
///    - Key events: Most common
///    - Mouse events: Optional support
///    - Resize events: Handle gracefully
/// 
/// 4. **Non-blocking Input**:
///    - poll() with timeout prevents blocking
///    - Allows periodic updates even without input
///    - Good for animations or status updates
/// 
/// Exercise: Add these keybindings:
/// 1. Ctrl+S to "save" in any mode
/// 2. Tab to insert 4 spaces in insert mode
/// 3. Arrow keys for cursor movement (track position)
/// 
/// Advanced: Add undo/redo with Ctrl+Z/Ctrl+Y