//! Form components for the TUI application.
//!
//! This module provides form field types, validation, and state management
//! for building interactive forms in the TUI.

use crate::{
    error::{Result, GitSetupError},
    tui::{Component, ComponentAction, Event, Theme, UIHelpers},
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, Paragraph, List, ListItem, Clear},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use regex::Regex;

/// Types of form fields
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Select(Vec<String>),
    Checkbox,
    Number,
    Path,
}

/// Validation result for form fields
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Warning(String),
}

/// Validation rules for form fields
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: ValidationRuleType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ValidationRuleType {
    Required,
    MinLength(usize),
    MaxLength(usize),
    EmailFormat,
    Regex(Regex),
    Custom(fn(&str) -> bool),
}

/// Form field configuration
#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub value: String,
    pub default_value: String,
    pub placeholder: String,
    pub help_text: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
    pub is_required: bool,
    pub is_readonly: bool,
    pub is_hidden: bool,
    pub tab_index: usize,
}

impl FormField {
    /// Create a new form field
    pub fn new(name: &str, label: &str, field_type: FieldType) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            field_type,
            value: String::new(),
            default_value: String::new(),
            placeholder: String::new(),
            help_text: None,
            validation_rules: Vec::new(),
            is_required: false,
            is_readonly: false,
            is_hidden: false,
            tab_index: 0,
        }
    }

    /// Set the field value
    pub fn set_value(&mut self, value: &str) -> &mut Self {
        self.value = value.to_string();
        self
    }

    /// Set the default value
    pub fn set_default(&mut self, default: &str) -> &mut Self {
        self.default_value = default.to_string();
        self
    }

    /// Set the placeholder text
    pub fn set_placeholder(&mut self, placeholder: &str) -> &mut Self {
        self.placeholder = placeholder.to_string();
        self
    }

    /// Set help text
    pub fn set_help(&mut self, help: &str) -> &mut Self {
        self.help_text = Some(help.to_string());
        self
    }

    /// Mark field as required
    pub fn set_required(&mut self, required: bool) -> &mut Self {
        self.is_required = required;
        self
    }

    /// Mark field as readonly
    pub fn set_readonly(&mut self, readonly: bool) -> &mut Self {
        self.is_readonly = readonly;
        self
    }

    /// Set tab index
    pub fn set_tab_index(&mut self, index: usize) -> &mut Self {
        self.tab_index = index;
        self
    }

    /// Add validation rule
    pub fn add_validation_rule(&mut self, rule: ValidationRule) -> &mut Self {
        self.validation_rules.push(rule);
        self
    }

    /// Validate the current field value
    pub fn validate(&self) -> ValidationResult {
        // Check required field
        if self.is_required && self.value.trim().is_empty() {
            return ValidationResult::Invalid("This field is required".to_string());
        }

        // Run validation rules
        for rule in &self.validation_rules {
            match &rule.rule_type {
                ValidationRuleType::Required => {
                    if self.value.trim().is_empty() {
                        return ValidationResult::Invalid(rule.message.clone());
                    }
                }
                ValidationRuleType::MinLength(min) => {
                    if self.value.len() < *min {
                        return ValidationResult::Invalid(rule.message.clone());
                    }
                }
                ValidationRuleType::MaxLength(max) => {
                    if self.value.len() > *max {
                        return ValidationResult::Invalid(rule.message.clone());
                    }
                }
                ValidationRuleType::EmailFormat => {
                    if !self.value.contains('@') || !self.value.contains('.') {
                        return ValidationResult::Invalid(rule.message.clone());
                    }
                }
                ValidationRuleType::Regex(regex) => {
                    if !regex.is_match(&self.value) {
                        return ValidationResult::Invalid(rule.message.clone());
                    }
                }
                ValidationRuleType::Custom(validate_fn) => {
                    if !validate_fn(&self.value) {
                        return ValidationResult::Invalid(rule.message.clone());
                    }
                }
            }
        }

        ValidationResult::Valid
    }

    /// Get display value (for password fields, return asterisks)
    pub fn display_value(&self) -> String {
        match self.field_type {
            FieldType::Password => "*".repeat(self.value.len()),
            _ => self.value.clone(),
        }
    }
}

/// Form state management
#[derive(Debug)]
pub struct FormState {
    pub fields: HashMap<String, FormField>,
    pub field_order: Vec<String>,
    pub current_field: usize,
    pub show_validation: bool,
    pub validation_results: HashMap<String, ValidationResult>,
    pub is_dirty: bool,
}

impl FormState {
    /// Create a new form state
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            field_order: Vec::new(),
            current_field: 0,
            show_validation: false,
            validation_results: HashMap::new(),
            is_dirty: false,
        }
    }

    /// Add a field to the form
    pub fn add_field(&mut self, field: FormField) -> &mut Self {
        let name = field.name.clone();
        let tab_index = field.tab_index;
        
        self.fields.insert(name.clone(), field);
        
        // Insert in order based on tab index
        let insert_pos = self.field_order
            .iter()
            .position(|fname| {
                self.fields.get(fname).map_or(false, |f| f.tab_index > tab_index)
            })
            .unwrap_or(self.field_order.len());
        
        self.field_order.insert(insert_pos, name);
        self
    }

    /// Get current field
    pub fn current_field(&self) -> Option<&FormField> {
        self.field_order
            .get(self.current_field)
            .and_then(|name| self.fields.get(name))
    }

    /// Get current field name
    pub fn current_field_name(&self) -> Option<&str> {
        self.field_order.get(self.current_field).map(|s| s.as_str())
    }

    /// Get mutable current field
    pub fn current_field_mut(&mut self) -> Option<&mut FormField> {
        if let Some(name) = self.field_order.get(self.current_field).cloned() {
            self.fields.get_mut(&name)
        } else {
            None
        }
    }

    /// Navigate to next field
    pub fn next_field(&mut self) -> bool {
        if self.current_field < self.field_order.len().saturating_sub(1) {
            self.current_field += 1;
            true
        } else {
            false
        }
    }

    /// Navigate to previous field
    pub fn prev_field(&mut self) -> bool {
        if self.current_field > 0 {
            self.current_field -= 1;
            true
        } else {
            false
        }
    }

    /// Set field value
    pub fn set_field_value(&mut self, name: &str, value: &str) -> Result<()> {
        if let Some(field) = self.fields.get_mut(name) {
            field.set_value(value);
            self.is_dirty = true;
            
            // Validate field if validation is enabled
            if self.show_validation {
                let result = field.validate();
                self.validation_results.insert(name.to_string(), result);
            }
            
            Ok(())
        } else {
            Err(GitSetupError::InvalidProfile { reason: format!("Field '{}' not found", name) })
        }
    }

    /// Validate all fields
    pub fn validate_all(&mut self) -> bool {
        self.show_validation = true;
        let mut all_valid = true;
        
        for (name, field) in &self.fields {
            let result = field.validate();
            if matches!(result, ValidationResult::Invalid(_)) {
                all_valid = false;
            }
            self.validation_results.insert(name.clone(), result);
        }
        
        all_valid
    }

    /// Get validation result for a field
    pub fn get_validation_result(&self, name: &str) -> Option<&ValidationResult> {
        self.validation_results.get(name)
    }

    /// Check if form has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Mark form as clean
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    /// Get form data as key-value pairs
    pub fn get_data(&self) -> HashMap<String, String> {
        self.fields
            .iter()
            .map(|(name, field)| (name.clone(), field.value.clone()))
            .collect()
    }

    /// Load form data from key-value pairs
    pub fn load_data(&mut self, data: HashMap<String, String>) {
        for (name, value) in data {
            if let Some(field) = self.fields.get_mut(&name) {
                field.set_value(&value);
            }
        }
        self.is_dirty = false;
    }
}

/// Form component for rendering and handling forms
pub struct FormComponent {
    state: FormState,
    title: String,
    show_help: bool,
    cursor_position: usize,
}

impl FormComponent {
    /// Create a new form component
    pub fn new(title: &str) -> Self {
        Self {
            state: FormState::new(),
            title: title.to_string(),
            show_help: false,
            cursor_position: 0,
        }
    }

    /// Add a field to the form
    pub fn add_field(&mut self, field: FormField) -> &mut Self {
        self.state.add_field(field);
        self
    }

    /// Get form state
    pub fn state(&self) -> &FormState {
        &self.state
    }

    /// Get mutable form state
    pub fn state_mut(&mut self) -> &mut FormState {
        &mut self.state
    }

    /// Handle character input
    fn handle_char_input(&mut self, ch: char) -> Result<ComponentAction> {
        if let Some(field) = self.state.current_field_mut() {
            if field.is_readonly {
                return Ok(ComponentAction::None);
            }

            match field.field_type {
                FieldType::Text | FieldType::Email | FieldType::Password | FieldType::Path => {
                    if self.cursor_position <= field.value.len() {
                        field.value.insert(self.cursor_position, ch);
                        self.cursor_position += 1;
                        self.state.is_dirty = true;
                    }
                }
                FieldType::Number => {
                    if ch.is_ascii_digit() || ch == '.' {
                        if self.cursor_position <= field.value.len() {
                            field.value.insert(self.cursor_position, ch);
                            self.cursor_position += 1;
                            self.state.is_dirty = true;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(ComponentAction::None)
    }

    /// Handle backspace
    fn handle_backspace(&mut self) -> Result<ComponentAction> {
        if let Some(field) = self.state.current_field_mut() {
            if field.is_readonly {
                return Ok(ComponentAction::None);
            }

            if self.cursor_position > 0 {
                field.value.remove(self.cursor_position - 1);
                self.cursor_position -= 1;
                self.state.is_dirty = true;
            }
        }
        Ok(ComponentAction::None)
    }

    /// Handle delete
    fn handle_delete(&mut self) -> Result<ComponentAction> {
        if let Some(field) = self.state.current_field_mut() {
            if field.is_readonly {
                return Ok(ComponentAction::None);
            }

            if self.cursor_position < field.value.len() {
                field.value.remove(self.cursor_position);
                self.state.is_dirty = true;
            }
        }
        Ok(ComponentAction::None)
    }

    /// Handle cursor movement
    fn handle_cursor_movement(&mut self, direction: CursorDirection) -> Result<ComponentAction> {
        if let Some(field) = self.state.current_field() {
            match direction {
                CursorDirection::Left => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                    }
                }
                CursorDirection::Right => {
                    if self.cursor_position < field.value.len() {
                        self.cursor_position += 1;
                    }
                }
                CursorDirection::Home => {
                    self.cursor_position = 0;
                }
                CursorDirection::End => {
                    self.cursor_position = field.value.len();
                }
            }
        }
        Ok(ComponentAction::None)
    }

    /// Render a form field
    fn render_field(&self, frame: &mut Frame, area: Rect, field: &FormField, is_focused: bool, theme: &Theme) -> Result<()> {
        let mut style = theme.styles.base;
        if is_focused {
            style = theme.styles.selected;
        }

        let block = Block::default()
            .title(format!("{}{}", field.label, if field.is_required { " *" } else { "" }))
            .borders(Borders::ALL)
            .border_style(if is_focused { theme.styles.selected } else { theme.styles.base });

        match &field.field_type {
            FieldType::Text | FieldType::Email | FieldType::Password | FieldType::Path | FieldType::Number => {
                let display_value = if field.value.is_empty() && !field.placeholder.is_empty() {
                    field.placeholder.clone()
                } else {
                    field.display_value()
                };

                let text = Paragraph::new(display_value)
                    .block(block)
                    .style(style);

                frame.render_widget(text, area);

                // Show cursor if focused
                if is_focused && self.cursor_position <= field.value.len() {
                    let cursor_x = area.x + 1 + self.cursor_position as u16;
                    let cursor_y = area.y + 1;
                    if cursor_x < area.x + area.width - 1 {
                        frame.set_cursor(cursor_x, cursor_y);
                    }
                }
            }
            FieldType::Select(options) => {
                let items: Vec<ListItem> = options
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let selected = field.value == *option;
                        let marker = if selected { "●" } else { "○" };
                        ListItem::new(format!("{} {}", marker, option))
                    })
                    .collect();

                let list = List::new(items)
                    .block(block)
                    .style(style);

                frame.render_widget(list, area);
            }
            FieldType::Checkbox => {
                let checked = field.value == "true";
                let marker = if checked { "☑" } else { "☐" };
                let text = Paragraph::new(format!("{} {}", marker, field.label))
                    .block(block)
                    .style(style);

                frame.render_widget(text, area);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum CursorDirection {
    Left,
    Right,
    Home,
    End,
}

impl Component for FormComponent {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        // Split area for form fields and help text
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                if self.show_help { Constraint::Length(3) } else { Constraint::Length(0) },
            ])
            .split(area);

        let form_area = chunks[0];
        let help_area = chunks[1];

        // Create layout for form fields
        let field_count = self.state.field_order.len();
        if field_count == 0 {
            return Ok(());
        }

        let constraints: Vec<Constraint> = (0..field_count)
            .map(|_| Constraint::Length(3))
            .collect();

        let field_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(form_area);

        // Render each field
        for (i, field_name) in self.state.field_order.iter().enumerate() {
            if let Some(field) = self.state.fields.get(field_name) {
                if !field.is_hidden && i < field_chunks.len() {
                    let is_focused = i == self.state.current_field;
                    self.render_field(frame, field_chunks[i], field, is_focused, theme)?;

                    // Show validation error if any
                    if self.state.show_validation {
                        if let Some(ValidationResult::Invalid(msg)) = self.state.get_validation_result(field_name) {
                            let error_area = Rect::new(
                                field_chunks[i].x + 1,
                                field_chunks[i].y + field_chunks[i].height - 1,
                                field_chunks[i].width - 2,
                                1,
                            );
                            let error_text = Paragraph::new(msg.clone())
                                .style(theme.styles.error);
                            frame.render_widget(error_text, error_area);
                        }
                    }
                }
            }
        }

        // Render help text if enabled
        if self.show_help && help_area.height > 0 {
            let help_text = "Tab/Shift+Tab: Navigate • Enter: Next field • Ctrl+S: Save • Esc: Cancel • F1: Toggle help";
            let help = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .style(theme.styles.help);
            frame.render_widget(help, help_area);
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Tab => {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            self.state.prev_field();
                        } else {
                            self.state.next_field();
                        }
                        // Reset cursor position for new field
                        self.cursor_position = if let Some(field) = self.state.current_field() {
                            field.value.len()
                        } else {
                            0
                        };
                        Ok(ComponentAction::None)
                    }
                    KeyCode::Enter => {
                        if self.state.next_field() {
                            self.cursor_position = if let Some(field) = self.state.current_field() {
                                field.value.len()
                            } else {
                                0
                            };
                        } else {
                            // Last field, validate and submit
                            if self.state.validate_all() {
                                return Ok(ComponentAction::Return("form_submitted".to_string()));
                            }
                        }
                        Ok(ComponentAction::None)
                    }
                    KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.state.validate_all() {
                            Ok(ComponentAction::Return("form_saved".to_string()))
                        } else {
                            Ok(ComponentAction::None)
                        }
                    }
                    KeyCode::Char(ch) => {
                        self.handle_char_input(ch)
                    }
                    KeyCode::Backspace => {
                        self.handle_backspace()
                    }
                    KeyCode::Delete => {
                        self.handle_delete()
                    }
                    KeyCode::Left => {
                        self.handle_cursor_movement(CursorDirection::Left)
                    }
                    KeyCode::Right => {
                        self.handle_cursor_movement(CursorDirection::Right)
                    }
                    KeyCode::Home => {
                        self.handle_cursor_movement(CursorDirection::Home)
                    }
                    KeyCode::End => {
                        self.handle_cursor_movement(CursorDirection::End)
                    }
                    KeyCode::F(1) => {
                        self.show_help = !self.show_help;
                        Ok(ComponentAction::None)
                    }
                    KeyCode::Esc => {
                        if self.state.is_dirty() {
                            Ok(ComponentAction::ShowPopup("Unsaved changes will be lost. Are you sure?".to_string()))
                        } else {
                            Ok(ComponentAction::NavigateBack)
                        }
                    }
                    _ => Ok(ComponentAction::None),
                }
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_creation() {
        let mut field = FormField::new("email", "Email Address", FieldType::Email);
        field.set_value("test@example.com")
            .set_required(true)
            .set_placeholder("Enter your email");

        assert_eq!(field.name, "email");
        assert_eq!(field.label, "Email Address");
        assert_eq!(field.value, "test@example.com");
        assert_eq!(field.placeholder, "Enter your email");
        assert!(field.is_required);
        assert!(!field.is_readonly);
    }

    #[test]
    fn test_form_field_validation() {
        let mut field = FormField::new("name", "Name", FieldType::Text);
        field.set_required(true);
        
        // Empty required field should be invalid
        assert!(matches!(field.validate(), ValidationResult::Invalid(_)));
        
        // Non-empty required field should be valid
        field.set_value("John Doe");
        assert!(matches!(field.validate(), ValidationResult::Valid));
    }

    #[test]
    fn test_form_field_validation_rules() {
        let mut field = FormField::new("password", "Password", FieldType::Password);
        field.add_validation_rule(ValidationRule {
            name: "min_length".to_string(),
            rule_type: ValidationRuleType::MinLength(8),
            message: "Password must be at least 8 characters".to_string(),
        });

        // Short password should be invalid
        field.set_value("123");
        assert!(matches!(field.validate(), ValidationResult::Invalid(_)));
        
        // Long enough password should be valid
        field.set_value("12345678");
        assert!(matches!(field.validate(), ValidationResult::Valid));
    }

    #[test]
    fn test_form_field_display_value() {
        let mut field = FormField::new("password", "Password", FieldType::Password);
        field.set_value("secret123");
        
        assert_eq!(field.display_value(), "*********");
        assert_eq!(field.value, "secret123");
    }

    #[test]
    fn test_form_state_field_management() {
        let mut state = FormState::new();
        
        let field1 = FormField::new("name", "Name", FieldType::Text);
        let field2 = FormField::new("email", "Email", FieldType::Email);
        
        state.add_field(field1);
        state.add_field(field2);
        
        assert_eq!(state.field_order.len(), 2);
        assert_eq!(state.current_field_name(), Some("name"));
        
        assert!(state.next_field());
        assert_eq!(state.current_field_name(), Some("email"));
        
        assert!(!state.next_field()); // Should be at last field
        
        assert!(state.prev_field());
        assert_eq!(state.current_field_name(), Some("name"));
    }

    #[test]
    fn test_form_state_validation() {
        let mut state = FormState::new();
        
        let mut field = FormField::new("email", "Email", FieldType::Email);
        field.set_required(true);
        state.add_field(field);
        
        // Empty required field should fail validation
        assert!(!state.validate_all());
        
        // Valid email should pass validation
        state.set_field_value("email", "test@example.com").unwrap();
        assert!(state.validate_all());
    }

    #[test]
    fn test_form_state_dirty_tracking() {
        let mut state = FormState::new();
        
        let field = FormField::new("name", "Name", FieldType::Text);
        state.add_field(field);
        
        assert!(!state.is_dirty());
        
        state.set_field_value("name", "John").unwrap();
        assert!(state.is_dirty());
        
        state.mark_clean();
        assert!(!state.is_dirty());
    }

    #[test]
    fn test_form_state_data_operations() {
        let mut state = FormState::new();
        
        let field1 = FormField::new("name", "Name", FieldType::Text);
        let field2 = FormField::new("email", "Email", FieldType::Email);
        
        state.add_field(field1);
        state.add_field(field2);
        
        // Load data
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John Doe".to_string());
        data.insert("email".to_string(), "john@example.com".to_string());
        
        state.load_data(data);
        
        assert_eq!(state.fields.get("name").unwrap().value, "John Doe");
        assert_eq!(state.fields.get("email").unwrap().value, "john@example.com");
        assert!(!state.is_dirty());
        
        // Get data
        let form_data = state.get_data();
        assert_eq!(form_data.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(form_data.get("email"), Some(&"john@example.com".to_string()));
    }

    #[test]
    fn test_validation_rule_types() {
        let regex_rule = ValidationRule {
            name: "phone".to_string(),
            rule_type: ValidationRuleType::Regex(Regex::new(r"^\d{10}$").unwrap()),
            message: "Invalid phone number".to_string(),
        };
        
        let mut field = FormField::new("phone", "Phone", FieldType::Text);
        field.add_validation_rule(regex_rule);
        
        field.set_value("123456789");
        assert!(matches!(field.validate(), ValidationResult::Invalid(_)));
        
        field.set_value("1234567890");
        assert!(matches!(field.validate(), ValidationResult::Valid));
    }

    #[test]
    fn test_form_component_creation() {
        let mut form = FormComponent::new("Test Form");
        
        let field = FormField::new("test", "Test Field", FieldType::Text);
        form.add_field(field);
        
        assert_eq!(form.title, "Test Form");
        assert_eq!(form.state.field_order.len(), 1);
        assert_eq!(form.state.current_field_name(), Some("test"));
    }
}