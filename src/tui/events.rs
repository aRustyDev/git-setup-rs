use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use crate::error::{Result, GitSetupError};

/// Application events
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Periodic tick for updates
    Tick,
    /// Custom application event
    Custom(String),
}

/// Event handler that manages input events
pub struct EventHandler {
    receiver: Receiver<Event>,
    _handler_thread: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();

        let handler_thread = thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if sender.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if sender.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(width, height)) => {
                            if sender.send(Event::Resize(width, height)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Ok(Self {
            receiver,
            _handler_thread: handler_thread,
        })
    }

    /// Get next event (non-blocking)
    pub fn next(&self) -> Result<Option<Event>> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(GitSetupError::ExternalCommand {
                    command: "EventHandler".to_string(),
                    error: "Event handler disconnected".to_string(),
                })
            }
        }
    }
}

/// Key binding definition
#[derive(Debug, Clone, PartialEq)]
pub struct KeyBinding {
    pub key: KeyEvent,
    pub description: String,
    pub action: KeyAction,
}

/// Actions that can be triggered by key bindings
#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    Quit,
    Help,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Select,
    Back,
    Delete,
    Edit,
    Create,
    Search,
    Filter,
    Custom(String),
}

/// Key binding manager
#[derive(Clone)]
pub struct KeyBindings {
    bindings: Vec<KeyBinding>,
}

impl KeyBindings {
    pub fn default() -> Self {
        use crossterm::event::{KeyCode, KeyModifiers};

        let bindings = vec![
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
                description: "Quit".to_string(),
                action: KeyAction::Quit,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()),
                description: "Help".to_string(),
                action: KeyAction::Help,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Up, KeyModifiers::empty()),
                description: "Navigate up".to_string(),
                action: KeyAction::NavigateUp,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
                description: "Navigate down".to_string(),
                action: KeyAction::NavigateDown,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
                description: "Select".to_string(),
                action: KeyAction::Select,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
                description: "Back".to_string(),
                action: KeyAction::Back,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()),
                description: "Delete".to_string(),
                action: KeyAction::Delete,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()),
                description: "Edit".to_string(),
                action: KeyAction::Edit,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty()),
                description: "New/Create".to_string(),
                action: KeyAction::Create,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()),
                description: "Search".to_string(),
                action: KeyAction::Search,
            },
            // Vim-style navigation
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
                description: "Down (vim)".to_string(),
                action: KeyAction::NavigateDown,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty()),
                description: "Up (vim)".to_string(),
                action: KeyAction::NavigateUp,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()),
                description: "Left (vim)".to_string(),
                action: KeyAction::NavigateLeft,
            },
            KeyBinding {
                key: KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()),
                description: "Right (vim)".to_string(),
                action: KeyAction::NavigateRight,
            },
        ];

        Self { bindings }
    }

    pub fn get_action(&self, key: &KeyEvent) -> Option<KeyAction> {
        self.bindings
            .iter()
            .find(|binding| binding.key == *key)
            .map(|binding| binding.action.clone())
    }

    pub fn get_help_text(&self) -> Vec<(String, String)> {
        self.bindings
            .iter()
            .map(|binding| {
                let key_str = format_key_event(&binding.key);
                (key_str, binding.description.clone())
            })
            .collect()
    }
}

/// Format a key event for display
fn format_key_event(key: &KeyEvent) -> String {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mut parts = Vec::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    let key_str = match key.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PgUp".to_string(),
        KeyCode::PageDown => "PgDn".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Delete => "Del".to_string(),
        KeyCode::Insert => "Ins".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => "?".to_string(),
    };

    parts.push(&key_str);
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_key_bindings_default() {
        let bindings = KeyBindings::default();

        // Test quit binding
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        assert_eq!(bindings.get_action(&quit_key), Some(KeyAction::Quit));

        // Test navigation
        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert_eq!(bindings.get_action(&up_key), Some(KeyAction::NavigateUp));

        // Test vim navigation
        let vim_down = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        assert_eq!(bindings.get_action(&vim_down), Some(KeyAction::NavigateDown));

        // Test unknown key
        let unknown_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        assert_eq!(bindings.get_action(&unknown_key), None);
    }

    #[test]
    fn test_format_key_event() {
        // Simple character
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        assert_eq!(format_key_event(&char_key), "a");

        // With modifiers
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(format_key_event(&ctrl_c), "Ctrl+c");

        // Special keys
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(format_key_event(&enter), "Enter");

        let arrow = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert_eq!(format_key_event(&arrow), "↑");

        // Multiple modifiers
        let ctrl_shift_a = KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        );
        assert_eq!(format_key_event(&ctrl_shift_a), "Ctrl+Shift+a");
    }

    #[test]
    fn test_key_action_equality() {
        assert_eq!(KeyAction::Quit, KeyAction::Quit);
        assert_ne!(KeyAction::Quit, KeyAction::Help);
        assert_eq!(
            KeyAction::Custom("test".to_string()),
            KeyAction::Custom("test".to_string())
        );
    }

    #[test]
    fn test_key_bindings_help_text() {
        let bindings = KeyBindings::default();
        let help_text = bindings.get_help_text();

        // Should have some help text
        assert!(!help_text.is_empty());

        // Check that quit is included
        assert!(help_text.iter().any(|(key, desc)| desc == "Quit"));
    }
}
