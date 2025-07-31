//! Profile creation screen for the TUI application.
//!
//! This module provides a wizard-style interface for creating new profiles.

use crate::{
    error::Result,
    tui::{
        Component, ComponentAction, Event, Theme,
        screens::{Screen, ScreenType},
        components::{FormComponent, FormField, FieldType, ValidationRule, ValidationRuleType},
    },
    config::types::{Profile, KeyType, Scope},
    profile::ProfileManager,
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    widgets::{Block, Borders, Paragraph},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::any::Any;
use std::sync::Arc;
use regex::Regex;

/// Profile creation screen
pub struct ProfileCreateScreen {
    form: FormComponent,
    theme: Theme,
    profile_manager: Arc<dyn ProfileManager>,
    current_step: usize,
    total_steps: usize,
}

impl ProfileCreateScreen {
    /// Create a new profile creation screen
    pub fn new(theme: Theme, profile_manager: Arc<dyn ProfileManager>) -> Self {
        let mut form = FormComponent::new("Create New Profile");
        
        // Add form fields
        let mut name_field = FormField::new("name", "Profile Name", FieldType::Text);
        name_field.set_required(true)
            .set_placeholder("Enter a unique name for this profile")
            .set_help("A descriptive name for this profile (e.g., 'work', 'personal')")
            .set_tab_index(0)
            .add_validation_rule(ValidationRule {
                name: "min_length".to_string(),
                rule_type: ValidationRuleType::MinLength(1),
                message: "Name must not be empty".to_string(),
            })
            .add_validation_rule(ValidationRule {
                name: "max_length".to_string(),
                rule_type: ValidationRuleType::MaxLength(50),
                message: "Name must be 50 characters or less".to_string(),
            })
            .add_validation_rule(ValidationRule {
                name: "no_special_chars".to_string(),
                rule_type: ValidationRuleType::Regex(Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()),
                message: "Name can only contain letters, numbers, hyphens, and underscores".to_string(),
            });

        let mut email_field = FormField::new("email", "Git User Email", FieldType::Email);
        email_field.set_required(true)
            .set_placeholder("your.email@example.com")
            .set_help("The email address to use for Git commits")
            .set_tab_index(1)
            .add_validation_rule(ValidationRule {
                name: "email_format".to_string(),
                rule_type: ValidationRuleType::EmailFormat,
                message: "Please enter a valid email address".to_string(),
            });

        let mut user_name_field = FormField::new("user_name", "Git User Name", FieldType::Text);
        user_name_field.set_placeholder("Your Full Name")
            .set_help("The name to use for Git commits (optional)")
            .set_tab_index(2);

        let mut key_type_field = FormField::new("key_type", "Key Type", FieldType::Select(vec![
            "Ssh".to_string(),
            "Gpg".to_string(),
            "X509".to_string(),
            "Gitsign".to_string(),
        ]));
        key_type_field.set_required(true)
            .set_value("Ssh")
            .set_help("The type of key to use for signing")
            .set_tab_index(3);

        let mut signing_key_field = FormField::new("signing_key", "Signing Key", FieldType::Text);
        signing_key_field.set_placeholder("ssh-ed25519 AAAAC3... or GPG key ID")
            .set_help("The key to use for signing commits (optional)")
            .set_tab_index(4);

        let mut vault_name_field = FormField::new("vault_name", "1Password Vault", FieldType::Text);
        vault_name_field.set_placeholder("Vault Name")
            .set_help("1Password vault containing SSH keys (optional)")
            .set_tab_index(5);

        let mut ssh_key_title_field = FormField::new("ssh_key_title", "SSH Key Title", FieldType::Text);
        ssh_key_title_field.set_placeholder("SSH Key Title")
            .set_help("Title of SSH key in 1Password (optional)")
            .set_tab_index(6);

        let mut scope_field = FormField::new("scope", "Scope", FieldType::Select(vec![
            "Local".to_string(),
            "Global".to_string(),
            "System".to_string(),
        ]));
        scope_field.set_value("Local")
            .set_help("The scope for Git configuration")
            .set_tab_index(7);

        let mut one_password_field = FormField::new("one_password", "Use 1Password", FieldType::Checkbox);
        one_password_field.set_value("false")
            .set_help("Whether to use 1Password for SSH key management")
            .set_tab_index(8);

        // Add fields to form
        form.add_field(name_field)
            .add_field(email_field)
            .add_field(user_name_field)
            .add_field(key_type_field)
            .add_field(signing_key_field)
            .add_field(vault_name_field)
            .add_field(ssh_key_title_field)
            .add_field(scope_field)
            .add_field(one_password_field);

        Self {
            form,
            theme,
            profile_manager,
            current_step: 1,
            total_steps: 3,
        }
    }

    /// Get the current step
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Get the total steps
    pub fn total_steps(&self) -> usize {
        self.total_steps
    }

    /// Create profile from form data
    pub fn create_profile(&mut self) -> Result<Profile> {
        let form_data = self.form.state().get_data();
        
        let name = form_data.get("name").unwrap_or(&String::new()).clone();
        let email = form_data.get("email").unwrap_or(&String::new()).clone();
        let user_name = form_data.get("user_name").unwrap_or(&String::new()).clone();
        let key_type_str = form_data.get("key_type").unwrap_or(&"Ssh".to_string()).clone();
        let signing_key = form_data.get("signing_key").unwrap_or(&String::new()).clone();
        let vault_name = form_data.get("vault_name").unwrap_or(&String::new()).clone();
        let ssh_key_title = form_data.get("ssh_key_title").unwrap_or(&String::new()).clone();
        let scope_str = form_data.get("scope").unwrap_or(&"Local".to_string()).clone();
        let one_password_str = form_data.get("one_password").unwrap_or(&"false".to_string()).clone();

        // Parse key type
        let key_type = match key_type_str.as_str() {
            "Ssh" => KeyType::Ssh,
            "Gpg" => KeyType::Gpg,
            "X509" => KeyType::X509,
            "Gitsign" => KeyType::Gitsign,
            _ => KeyType::Ssh,
        };

        // Parse scope
        let scope = match scope_str.as_str() {
            "Local" => Some(Scope::Local),
            "Global" => Some(Scope::Global),
            "System" => Some(Scope::System),
            _ => Some(Scope::Local),
        };

        // Parse one password
        let one_password = one_password_str == "true";

        let profile = Profile {
            name,
            git_user_name: if user_name.is_empty() { None } else { Some(user_name) },
            git_user_email: email,
            key_type,
            signing_key: if signing_key.is_empty() { None } else { Some(signing_key) },
            vault_name: if vault_name.is_empty() { None } else { Some(vault_name) },
            ssh_key_title: if ssh_key_title.is_empty() { None } else { Some(ssh_key_title) },
            scope,
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password,
        };

        // Validate that profile name doesn't already exist
        if self.profile_manager.exists(&profile.name)? {
            return Err(crate::error::GitSetupError::DuplicateProfile { 
                name: profile.name.clone() 
            });
        }

        // Create the profile
        self.profile_manager.create(profile.clone())?;

        Ok(profile)
    }

    /// Render the profile creation screen
    fn render_profile_create(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(0),     // Form
                Constraint::Length(3),  // Progress/Help
            ])
            .split(area);

        // Render title
        let title_text = format!("Create New Profile - Step {} of {}", self.current_step, self.total_steps);
        let title = Paragraph::new(title_text)
            .style(self.theme.styles.title)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
            );
        frame.render_widget(title, chunks[0]);

        // Render form
        self.form.render(frame, chunks[1], &self.theme)?;

        // Render progress/help
        let help_text = "Tab/Shift+Tab: Navigate • Ctrl+S: Save • Esc: Cancel • ? for help";
        let help = Paragraph::new(help_text)
            .style(self.theme.styles.help)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Controls")
            );
        frame.render_widget(help, chunks[2]);

        Ok(())
    }

    /// Handle form submission
    fn handle_form_submit(&mut self) -> Result<ComponentAction> {
        if self.form.state().validate_all() {
            match self.create_profile() {
                Ok(profile) => {
                    Ok(ComponentAction::ShowPopup(format!("Profile '{}' created successfully!", profile.name)))
                }
                Err(e) => {
                    Ok(ComponentAction::ShowPopup(format!("Error creating profile: {}", e)))
                }
            }
        } else {
            Ok(ComponentAction::ShowPopup("Please fix validation errors before saving".to_string()))
        }
    }
}

impl Component for ProfileCreateScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        self.render_profile_create(frame, area)
    }

    fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Char('s') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        self.handle_form_submit()
                    }
                    KeyCode::Esc => {
                        if self.form.state().is_dirty() {
                            Ok(ComponentAction::ShowPopup("Unsaved changes will be lost. Are you sure?".to_string()))
                        } else {
                            Ok(ComponentAction::NavigateBack)
                        }
                    }
                    _ => {
                        // Forward event to form
                        self.form.handle_event(event)
                    }
                }
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

impl Screen for ProfileCreateScreen {
    fn title(&self) -> &str {
        "Create Profile"
    }

    fn screen_type(&self) -> ScreenType {
        ScreenType::ProfileCreate
    }

    fn screen_help(&self) -> Vec<(&str, &str)> {
        vec![
            ("Tab/Shift+Tab", "Navigate between fields"),
            ("Ctrl+S", "Save profile"),
            ("Esc", "Cancel creation"),
            ("?", "Toggle help"),
        ]
    }

    fn can_exit(&self) -> bool {
        !self.form.state().is_dirty()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::ProfileManager;
    use std::sync::Arc;

    struct MockProfileManager {
        profiles: Vec<Profile>,
    }

    impl MockProfileManager {
        fn new() -> Self {
            Self { profiles: vec![] }
        }

        fn with_profile(profile: Profile) -> Self {
            Self { profiles: vec![profile] }
        }
    }

    impl ProfileManager for MockProfileManager {
        fn create(&self, _profile: Profile) -> Result<()> {
            Ok(())
        }

        fn read(&self, name: &str) -> Result<Option<Profile>> {
            Ok(self.profiles.iter().find(|p| p.name == name).cloned())
        }

        fn update(&self, _name: &str, _profile: Profile) -> Result<()> {
            Ok(())
        }

        fn delete(&self, _name: &str) -> Result<()> {
            Ok(())
        }

        fn list(&self) -> Result<Vec<Profile>> {
            Ok(self.profiles.clone())
        }

        fn exists(&self, name: &str) -> Result<bool> {
            Ok(self.profiles.iter().any(|p| p.name == name))
        }
    }

    fn create_test_screen() -> ProfileCreateScreen {
        let manager = Arc::new(MockProfileManager::new());
        ProfileCreateScreen::new(Theme::default(), manager)
    }

    #[test]
    fn test_profile_create_screen_creation() {
        let screen = create_test_screen();
        
        assert_eq!(screen.title(), "Create Profile");
        assert_eq!(screen.screen_type(), ScreenType::ProfileCreate);
        assert_eq!(screen.current_step(), 1);
        assert_eq!(screen.total_steps(), 3);
        assert!(screen.can_exit());
    }

    #[test]
    fn test_profile_create_screen_help() {
        let screen = create_test_screen();
        let help = screen.screen_help();
        
        assert!(!help.is_empty());
        assert!(help.iter().any(|(_, desc)| desc.contains("Navigate")));
        assert!(help.iter().any(|(_, desc)| desc.contains("Save")));
        assert!(help.iter().any(|(_, desc)| desc.contains("Cancel")));
    }

    #[test]
    fn test_profile_create_form_validation() {
        let mut screen = create_test_screen();
        
        // Test empty form validation
        assert!(!screen.form.state().validate_all());
        
        // Set required fields
        screen.form.state_mut().set_field_value("name", "test").unwrap();
        screen.form.state_mut().set_field_value("email", "test@example.com").unwrap();
        
        // Should now be valid
        assert!(screen.form.state().validate_all());
    }

    #[test]
    fn test_profile_create_duplicate_name() {
        let existing_profile = Profile {
            name: "existing".to_string(),
            git_user_name: None,
            git_user_email: "existing@example.com".to_string(),
            key_type: KeyType::Ssh,
            signing_key: None,
            vault_name: None,
            ssh_key_title: None,
            scope: Some(Scope::Local),
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password: false,
        };

        let manager = Arc::new(MockProfileManager::with_profile(existing_profile));
        let mut screen = ProfileCreateScreen::new(Theme::default(), manager);
        
        // Set form data to duplicate name
        screen.form.state_mut().set_field_value("name", "existing").unwrap();
        screen.form.state_mut().set_field_value("email", "test@example.com").unwrap();
        
        // Should fail due to duplicate name
        let result = screen.create_profile();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::error::GitSetupError::DuplicateProfile { .. }));
    }

    #[test]
    fn test_profile_create_success() {
        let mut screen = create_test_screen();
        
        // Set valid form data
        screen.form.state_mut().set_field_value("name", "test").unwrap();
        screen.form.state_mut().set_field_value("email", "test@example.com").unwrap();
        screen.form.state_mut().set_field_value("user_name", "Test User").unwrap();
        screen.form.state_mut().set_field_value("key_type", "Ssh").unwrap();
        screen.form.state_mut().set_field_value("scope", "Local").unwrap();
        screen.form.state_mut().set_field_value("one_password", "true").unwrap();
        
        // Should succeed
        let result = screen.create_profile();
        assert!(result.is_ok());
        
        let profile = result.unwrap();
        assert_eq!(profile.name, "test");
        assert_eq!(profile.git_user_email, "test@example.com");
        assert_eq!(profile.git_user_name, Some("Test User".to_string()));
        assert_eq!(profile.key_type, KeyType::Ssh);
        assert_eq!(profile.scope, Some(Scope::Local));
        assert!(profile.one_password);
    }

    #[test]
    fn test_profile_create_event_handling() {
        let mut screen = create_test_screen();
        
        // Test escape without changes
        let escape_event = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(escape_event)).unwrap();
        assert_eq!(result, ComponentAction::NavigateBack);
        
        // Make form dirty
        screen.form.state_mut().set_field_value("name", "test").unwrap();
        
        // Test escape with changes
        let escape_event = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(escape_event)).unwrap();
        match result {
            ComponentAction::ShowPopup(msg) => assert!(msg.contains("Unsaved changes")),
            _ => panic!("Expected ShowPopup action"),
        }
    }

    #[test]
    fn test_profile_create_save_event() {
        let mut screen = create_test_screen();
        
        // Test save without valid data
        let save_event = KeyEvent::new(KeyCode::Char('s'), crossterm::event::KeyModifiers::CONTROL);
        let result = screen.handle_event(Event::Key(save_event)).unwrap();
        match result {
            ComponentAction::ShowPopup(msg) => assert!(msg.contains("validation errors")),
            _ => panic!("Expected ShowPopup action"),
        }
        
        // Set valid form data
        screen.form.state_mut().set_field_value("name", "test").unwrap();
        screen.form.state_mut().set_field_value("email", "test@example.com").unwrap();
        
        // Test save with valid data
        let save_event = KeyEvent::new(KeyCode::Char('s'), crossterm::event::KeyModifiers::CONTROL);
        let result = screen.handle_event(Event::Key(save_event)).unwrap();
        match result {
            ComponentAction::ShowPopup(msg) => assert!(msg.contains("created successfully")),
            _ => panic!("Expected ShowPopup action"),
        }
    }

    #[test]
    fn test_profile_create_can_exit() {
        let mut screen = create_test_screen();
        
        // Should be able to exit initially
        assert!(screen.can_exit());
        
        // Make form dirty
        screen.form.state_mut().set_field_value("name", "test").unwrap();
        
        // Should not be able to exit with unsaved changes
        assert!(!screen.can_exit());
    }

    #[test]
    fn test_profile_create_key_type_parsing() {
        let mut screen = create_test_screen();
        
        // Test different key types
        let test_cases = vec![
            ("Ssh", KeyType::Ssh),
            ("Gpg", KeyType::Gpg),
            ("X509", KeyType::X509),
            ("Gitsign", KeyType::Gitsign),
            ("Unknown", KeyType::Ssh), // Should default to Ssh
        ];
        
        for (key_type_str, expected_key_type) in test_cases {
            screen.form.state_mut().set_field_value("name", "test").unwrap();
            screen.form.state_mut().set_field_value("email", "test@example.com").unwrap();
            screen.form.state_mut().set_field_value("key_type", key_type_str).unwrap();
            
            let result = screen.create_profile();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().key_type, expected_key_type);
        }
    }

    #[test]
    fn test_profile_create_scope_parsing() {
        let mut screen = create_test_screen();
        
        // Test different scopes
        let test_cases = vec![
            ("Local", Some(Scope::Local)),
            ("Global", Some(Scope::Global)),
            ("System", Some(Scope::System)),
            ("Unknown", Some(Scope::Local)), // Should default to Local
        ];
        
        for (scope_str, expected_scope) in test_cases {
            screen.form.state_mut().set_field_value("name", "test").unwrap();
            screen.form.state_mut().set_field_value("email", "test@example.com").unwrap();
            screen.form.state_mut().set_field_value("scope", scope_str).unwrap();
            
            let result = screen.create_profile();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().scope, expected_scope);
        }
    }

    #[test]
    fn test_profile_create_optional_fields() {
        let mut screen = create_test_screen();
        
        // Set only required fields
        screen.form.state_mut().set_field_value("name", "test").unwrap();
        screen.form.state_mut().set_field_value("email", "test@example.com").unwrap();
        
        let result = screen.create_profile();
        assert!(result.is_ok());
        
        let profile = result.unwrap();
        assert_eq!(profile.name, "test");
        assert_eq!(profile.git_user_email, "test@example.com");
        assert_eq!(profile.git_user_name, None);
        assert_eq!(profile.signing_key, None);
        assert_eq!(profile.vault_name, None);
        assert_eq!(profile.ssh_key_title, None);
        assert!(!profile.one_password);
    }
}