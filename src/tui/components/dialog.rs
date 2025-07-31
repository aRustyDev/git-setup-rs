//! Dialog components for the TUI application.
//!
//! This module provides various dialog types for user interaction including
//! confirmation dialogs, message dialogs, and progress dialogs.

use crate::{
    error::Result,
    tui::{Component, ComponentAction, Event, Theme, UIHelpers},
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, Paragraph, Gauge, Clear, List, ListItem},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyEvent};

/// Types of dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    Confirm {
        title: String,
        message: String,
        default_yes: bool,
    },
    Message {
        title: String,
        message: String,
        dialog_type: MessageType,
    },
    Progress {
        title: String,
        message: String,
        progress: f64,
        can_cancel: bool,
    },
    Input {
        title: String,
        message: String,
        default_value: String,
        placeholder: String,
    },
    Select {
        title: String,
        message: String,
        options: Vec<String>,
        selected: usize,
    },
}

/// Message dialog types
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Info,
    Warning,
    Error,
    Success,
}

/// Dialog result
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    Yes,
    No,
    Ok,
    Cancel,
    Input(String),
    Selected(usize),
}

/// Dialog component for various user interactions
pub struct DialogComponent {
    dialog_type: DialogType,
    visible: bool,
    input_value: String,
    cursor_position: usize,
}

impl DialogComponent {
    /// Create a new dialog component
    pub fn new(dialog_type: DialogType) -> Self {
        let input_value = match &dialog_type {
            DialogType::Input { default_value, .. } => default_value.clone(),
            _ => String::new(),
        };

        Self {
            dialog_type,
            visible: true,
            input_value,
            cursor_position: 0,
        }
    }

    /// Create a confirmation dialog
    pub fn confirm(title: &str, message: &str, default_yes: bool) -> Self {
        Self::new(DialogType::Confirm {
            title: title.to_string(),
            message: message.to_string(),
            default_yes,
        })
    }

    /// Create an info message dialog
    pub fn info(title: &str, message: &str) -> Self {
        Self::new(DialogType::Message {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: MessageType::Info,
        })
    }

    /// Create a warning message dialog
    pub fn warning(title: &str, message: &str) -> Self {
        Self::new(DialogType::Message {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: MessageType::Warning,
        })
    }

    /// Create an error message dialog
    pub fn error(title: &str, message: &str) -> Self {
        Self::new(DialogType::Message {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: MessageType::Error,
        })
    }

    /// Create a success message dialog
    pub fn success(title: &str, message: &str) -> Self {
        Self::new(DialogType::Message {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: MessageType::Success,
        })
    }

    /// Create a progress dialog
    pub fn progress(title: &str, message: &str, progress: f64, can_cancel: bool) -> Self {
        Self::new(DialogType::Progress {
            title: title.to_string(),
            message: message.to_string(),
            progress,
            can_cancel,
        })
    }

    /// Create an input dialog
    pub fn input(title: &str, message: &str, default_value: &str, placeholder: &str) -> Self {
        Self::new(DialogType::Input {
            title: title.to_string(),
            message: message.to_string(),
            default_value: default_value.to_string(),
            placeholder: placeholder.to_string(),
        })
    }

    /// Create a selection dialog
    pub fn select(title: &str, message: &str, options: Vec<String>, selected: usize) -> Self {
        Self::new(DialogType::Select {
            title: title.to_string(),
            message: message.to_string(),
            options,
            selected,
        })
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if dialog is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Update progress (for progress dialogs)
    pub fn update_progress(&mut self, progress: f64, message: Option<&str>) {
        if let DialogType::Progress { progress: ref mut p, message: ref mut m, .. } = self.dialog_type {
            *p = progress.clamp(0.0, 1.0);
            if let Some(new_message) = message {
                *m = new_message.to_string();
            }
        }
    }

    /// Handle character input for input dialogs
    fn handle_char_input(&mut self, ch: char) -> Result<ComponentAction> {
        if let DialogType::Input { .. } = self.dialog_type {
            if self.cursor_position <= self.input_value.len() {
                self.input_value.insert(self.cursor_position, ch);
                self.cursor_position += 1;
            }
        }
        Ok(ComponentAction::None)
    }

    /// Handle backspace for input dialogs
    fn handle_backspace(&mut self) -> Result<ComponentAction> {
        if let DialogType::Input { .. } = self.dialog_type {
            if self.cursor_position > 0 {
                self.input_value.remove(self.cursor_position - 1);
                self.cursor_position -= 1;
            }
        }
        Ok(ComponentAction::None)
    }

    /// Handle selection navigation
    fn handle_selection_nav(&mut self, direction: i32) -> Result<ComponentAction> {
        if let DialogType::Select { options, selected, .. } = &mut self.dialog_type {
            let new_selected = (*selected as i32 + direction).max(0) as usize;
            *selected = new_selected.min(options.len().saturating_sub(1));
        }
        Ok(ComponentAction::None)
    }

    /// Get the message type style
    fn get_message_style(&self, message_type: &MessageType, theme: &Theme) -> Style {
        match message_type {
            MessageType::Info => theme.styles.info,
            MessageType::Warning => theme.styles.warning,
            MessageType::Error => theme.styles.error,
            MessageType::Success => theme.styles.success,
        }
    }

    /// Render confirmation dialog
    fn render_confirm(&self, frame: &mut Frame, area: Rect, title: &str, message: &str, default_yes: bool, theme: &Theme) -> Result<()> {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme.styles.dialog_border);

        frame.render_widget(Clear, area);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(inner);

        // Message
        let message_text = Paragraph::new(message)
            .style(theme.styles.base)
            .alignment(Alignment::Center);
        frame.render_widget(message_text, chunks[0]);

        // Buttons
        let button_area = chunks[1];
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(button_area);

        let yes_style = if default_yes { theme.styles.selected } else { theme.styles.base };
        let no_style = if !default_yes { theme.styles.selected } else { theme.styles.base };

        let yes_button = Paragraph::new("[ Yes ]")
            .style(yes_style)
            .alignment(Alignment::Center);
        let no_button = Paragraph::new("[ No ]")
            .style(no_style)
            .alignment(Alignment::Center);

        frame.render_widget(yes_button, button_chunks[0]);
        frame.render_widget(no_button, button_chunks[1]);

        Ok(())
    }

    /// Render message dialog
    fn render_message(&self, frame: &mut Frame, area: Rect, title: &str, message: &str, msg_type: &MessageType, theme: &Theme) -> Result<()> {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(self.get_message_style(msg_type, theme));

        frame.render_widget(Clear, area);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(inner);

        // Message
        let message_text = Paragraph::new(message)
            .style(self.get_message_style(msg_type, theme))
            .alignment(Alignment::Center);
        frame.render_widget(message_text, chunks[0]);

        // OK button
        let ok_button = Paragraph::new("[ OK ]")
            .style(theme.styles.selected)
            .alignment(Alignment::Center);
        frame.render_widget(ok_button, chunks[1]);

        Ok(())
    }

    /// Render progress dialog
    fn render_progress(&self, frame: &mut Frame, area: Rect, title: &str, message: &str, progress: f64, can_cancel: bool, theme: &Theme) -> Result<()> {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme.styles.dialog_border);

        frame.render_widget(Clear, area);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                if can_cancel { Constraint::Length(3) } else { Constraint::Length(0) },
            ])
            .split(inner);

        // Message
        let message_text = Paragraph::new(message)
            .style(theme.styles.base)
            .alignment(Alignment::Center);
        frame.render_widget(message_text, chunks[0]);

        // Progress bar
        let progress_bar = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(theme.styles.progress)
            .ratio(progress)
            .label(format!("{:.1}%", progress * 100.0));
        frame.render_widget(progress_bar, chunks[1]);

        // Cancel button
        if can_cancel {
            let cancel_button = Paragraph::new("[ Cancel ]")
                .style(theme.styles.base)
                .alignment(Alignment::Center);
            frame.render_widget(cancel_button, chunks[2]);
        }

        Ok(())
    }

    /// Render input dialog
    fn render_input(&self, frame: &mut Frame, area: Rect, title: &str, message: &str, placeholder: &str, theme: &Theme) -> Result<()> {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme.styles.dialog_border);

        frame.render_widget(Clear, area);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(inner);

        // Message
        let message_text = Paragraph::new(message)
            .style(theme.styles.base)
            .alignment(Alignment::Center);
        frame.render_widget(message_text, chunks[0]);

        // Input field
        let display_value = if self.input_value.is_empty() {
            placeholder
        } else {
            &self.input_value
        };

        let input_field = Paragraph::new(display_value)
            .block(Block::default().borders(Borders::ALL).title("Input"))
            .style(theme.styles.selected);
        frame.render_widget(input_field, chunks[1]);

        // Show cursor
        let cursor_x = chunks[1].x + 1 + self.cursor_position as u16;
        let cursor_y = chunks[1].y + 1;
        if cursor_x < chunks[1].x + chunks[1].width - 1 {
            frame.set_cursor(cursor_x, cursor_y);
        }

        // Buttons
        let button_area = chunks[2];
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(button_area);

        let ok_button = Paragraph::new("[ OK ]")
            .style(theme.styles.base)
            .alignment(Alignment::Center);
        let cancel_button = Paragraph::new("[ Cancel ]")
            .style(theme.styles.base)
            .alignment(Alignment::Center);

        frame.render_widget(ok_button, button_chunks[0]);
        frame.render_widget(cancel_button, button_chunks[1]);

        Ok(())
    }

    /// Render selection dialog
    fn render_select(&self, frame: &mut Frame, area: Rect, title: &str, message: &str, options: &[String], selected: usize, theme: &Theme) -> Result<()> {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme.styles.dialog_border);

        frame.render_widget(Clear, area);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(inner);

        // Message
        let message_text = Paragraph::new(message)
            .style(theme.styles.base)
            .alignment(Alignment::Center);
        frame.render_widget(message_text, chunks[0]);

        // Options list
        let items: Vec<ListItem> = options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let marker = if i == selected { ">" } else { " " };
                ListItem::new(format!("{} {}", marker, option))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Options"))
            .style(theme.styles.base)
            .highlight_style(theme.styles.selected);
        frame.render_widget(list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("↑/↓: Navigate • Enter: Select • Esc: Cancel")
            .style(theme.styles.help)
            .alignment(Alignment::Center);
        frame.render_widget(instructions, chunks[2]);

        Ok(())
    }
}

impl Component for DialogComponent {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Calculate dialog size based on type
        let (width, height) = match &self.dialog_type {
            DialogType::Confirm { .. } => (50, 8),
            DialogType::Message { .. } => (50, 8),
            DialogType::Progress { .. } => (60, 10),
            DialogType::Input { .. } => (50, 10),
            DialogType::Select { options, .. } => (60, 8 + options.len().min(10) as u16),
        };

        let dialog_area = UIHelpers::centered_rect(width, height, area);

        match &self.dialog_type {
            DialogType::Confirm { title, message, default_yes } => {
                self.render_confirm(frame, dialog_area, title, message, *default_yes, theme)?;
            }
            DialogType::Message { title, message, dialog_type } => {
                self.render_message(frame, dialog_area, title, message, dialog_type, theme)?;
            }
            DialogType::Progress { title, message, progress, can_cancel } => {
                self.render_progress(frame, dialog_area, title, message, *progress, *can_cancel, theme)?;
            }
            DialogType::Input { title, message, placeholder, .. } => {
                self.render_input(frame, dialog_area, title, message, placeholder, theme)?;
            }
            DialogType::Select { title, message, options, selected } => {
                self.render_select(frame, dialog_area, title, message, options, *selected, theme)?;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        if !self.visible {
            return Ok(ComponentAction::None);
        }

        match event {
            Event::Key(key_event) => {
                match &self.dialog_type {
                    DialogType::Confirm { default_yes, .. } => {
                        match key_event.code {
                            KeyCode::Enter => {
                                let result = if *default_yes { "yes" } else { "no" }.to_string();
                                self.hide();
                                Ok(ComponentAction::Return(result))
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                self.hide();
                                Ok(ComponentAction::Return("yes".to_string()))
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                self.hide();
                                Ok(ComponentAction::Return("no".to_string()))
                            }
                            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                                // Toggle default selection
                                if let DialogType::Confirm { default_yes, .. } = &mut self.dialog_type {
                                    *default_yes = !*default_yes;
                                }
                                Ok(ComponentAction::None)
                            }
                            KeyCode::Esc => {
                                self.hide();
                                Ok(ComponentAction::Return("no".to_string()))
                            }
                            _ => Ok(ComponentAction::None),
                        }
                    }
                    DialogType::Message { .. } => {
                        match key_event.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                self.hide();
                                Ok(ComponentAction::Return("ok".to_string()))
                            }
                            _ => Ok(ComponentAction::None),
                        }
                    }
                    DialogType::Progress { can_cancel, .. } => {
                        if *can_cancel {
                            match key_event.code {
                                KeyCode::Esc => {
                                    self.hide();
                                    Ok(ComponentAction::Return("cancel".to_string()))
                                }
                                _ => Ok(ComponentAction::None),
                            }
                        } else {
                            Ok(ComponentAction::None)
                        }
                    }
                    DialogType::Input { .. } => {
                        match key_event.code {
                            KeyCode::Enter => {
                                self.hide();
                                Ok(ComponentAction::Return(format!("input:{}", self.input_value)))
                            }
                            KeyCode::Esc => {
                                self.hide();
                                Ok(ComponentAction::Return("cancel".to_string()))
                            }
                            KeyCode::Char(ch) => {
                                self.handle_char_input(ch)
                            }
                            KeyCode::Backspace => {
                                self.handle_backspace()
                            }
                            KeyCode::Left => {
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                }
                                Ok(ComponentAction::None)
                            }
                            KeyCode::Right => {
                                if self.cursor_position < self.input_value.len() {
                                    self.cursor_position += 1;
                                }
                                Ok(ComponentAction::None)
                            }
                            KeyCode::Home => {
                                self.cursor_position = 0;
                                Ok(ComponentAction::None)
                            }
                            KeyCode::End => {
                                self.cursor_position = self.input_value.len();
                                Ok(ComponentAction::None)
                            }
                            _ => Ok(ComponentAction::None),
                        }
                    }
                    DialogType::Select { .. } => {
                        match key_event.code {
                            KeyCode::Enter => {
                                if let DialogType::Select { selected, .. } = &self.dialog_type {
                                    let result = format!("selected:{}", selected);
                                    self.hide();
                                    Ok(ComponentAction::Return(result))
                                } else {
                                    Ok(ComponentAction::None)
                                }
                            }
                            KeyCode::Esc => {
                                self.hide();
                                Ok(ComponentAction::Return("cancel".to_string()))
                            }
                            KeyCode::Up => {
                                self.handle_selection_nav(-1)
                            }
                            KeyCode::Down => {
                                self.handle_selection_nav(1)
                            }
                            _ => Ok(ComponentAction::None),
                        }
                    }
                }
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::Theme;

    #[test]
    fn test_dialog_creation() {
        let confirm = DialogComponent::confirm("Test", "Are you sure?", true);
        assert!(confirm.is_visible());
        assert!(matches!(confirm.dialog_type, DialogType::Confirm { .. }));

        let info = DialogComponent::info("Info", "This is information");
        assert!(matches!(info.dialog_type, DialogType::Message { dialog_type: MessageType::Info, .. }));

        let error = DialogComponent::error("Error", "Something went wrong");
        assert!(matches!(error.dialog_type, DialogType::Message { dialog_type: MessageType::Error, .. }));
    }

    #[test]
    fn test_dialog_visibility() {
        let mut dialog = DialogComponent::confirm("Test", "Are you sure?", true);
        assert!(dialog.is_visible());

        dialog.hide();
        assert!(!dialog.is_visible());

        dialog.show();
        assert!(dialog.is_visible());
    }

    #[test]
    fn test_progress_dialog() {
        let mut dialog = DialogComponent::progress("Processing", "Please wait...", 0.5, true);
        
        if let DialogType::Progress { progress, .. } = &dialog.dialog_type {
            assert!((progress - 0.5).abs() < f64::EPSILON);
        }

        dialog.update_progress(0.8, Some("Almost done..."));
        
        if let DialogType::Progress { progress, message, .. } = &dialog.dialog_type {
            assert!((progress - 0.8).abs() < f64::EPSILON);
            assert_eq!(message, "Almost done...");
        }
    }

    #[test]
    fn test_input_dialog() {
        let mut dialog = DialogComponent::input("Input", "Enter name:", "default", "placeholder");
        
        assert_eq!(dialog.input_value, "default");
        assert_eq!(dialog.cursor_position, 0);

        // Test character input
        dialog.handle_char_input('a').unwrap();
        assert_eq!(dialog.input_value, "adefault");
        assert_eq!(dialog.cursor_position, 1);

        // Test backspace
        dialog.handle_backspace().unwrap();
        assert_eq!(dialog.input_value, "default");
        assert_eq!(dialog.cursor_position, 0);
    }

    #[test]
    fn test_select_dialog() {
        let mut dialog = DialogComponent::select(
            "Select",
            "Choose option:",
            vec!["Option 1".to_string(), "Option 2".to_string(), "Option 3".to_string()],
            0,
        );

        if let DialogType::Select { selected, .. } = &dialog.dialog_type {
            assert_eq!(*selected, 0);
        }

        // Test navigation
        dialog.handle_selection_nav(1).unwrap();
        if let DialogType::Select { selected, .. } = &dialog.dialog_type {
            assert_eq!(*selected, 1);
        }

        dialog.handle_selection_nav(-1).unwrap();
        if let DialogType::Select { selected, .. } = &dialog.dialog_type {
            assert_eq!(*selected, 0);
        }
    }

    #[test]
    fn test_dialog_component_trait() {
        let dialog = DialogComponent::confirm("Test", "Are you sure?", true);
        let theme = Theme::default();
        
        // Test that we can use the Component trait
        // Note: We can't easily test rendering without a terminal, but we can test the trait exists
        assert!(dialog.is_visible());
    }
}