//! Profile view screen for the TUI application.
//!
//! This module provides a read-only view of a profile's details.

use crate::{
    error::Result,
    tui::{
        Component, ComponentAction, Event, Theme,
        screens::{Screen, ScreenType},
    },
    config::types::Profile,
    profile::ProfileManager,
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::any::Any;
use std::sync::Arc;

/// Profile view screen
pub struct ProfileViewScreen {
    profile_name: String,
    profile: Option<Profile>,
    theme: Theme,
    profile_manager: Arc<dyn ProfileManager>,
}

impl ProfileViewScreen {
    /// Create a new profile view screen
    pub fn new(profile_name: String, theme: Theme, profile_manager: Arc<dyn ProfileManager>) -> Result<Self> {
        let profile = profile_manager.read(&profile_name)?;
        
        Ok(Self {
            profile_name,
            profile,
            theme,
            profile_manager,
        })
    }

    /// Get the profile name
    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Get the profile
    pub fn profile(&self) -> Option<&Profile> {
        self.profile.as_ref()
    }

    /// Refresh the profile data
    pub fn refresh(&mut self) -> Result<()> {
        self.profile = self.profile_manager.read(&self.profile_name)?;
        Ok(())
    }

    /// Render the profile view
    fn render_profile_view(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Some(profile) = &self.profile {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Min(0),     // Profile details
                    Constraint::Length(3),  // Help
                ])
                .split(area);

            // Render title
            let title = Paragraph::new(format!("Profile: {}", profile.name))
                .style(self.theme.styles.title)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.theme.styles.border)
                );
            frame.render_widget(title, chunks[0]);

            // Render profile details
            self.render_profile_details(frame, chunks[1], profile)?;

            // Render help
            let help_text = "e: Edit • d: Delete • Esc: Back • q: Quit";
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
        } else {
            let error_msg = format!("Profile '{}' not found", self.profile_name);
            let error = Paragraph::new(error_msg)
                .style(self.theme.styles.error)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.theme.styles.border)
                        .title("Error")
                );
            frame.render_widget(error, area);
        }

        Ok(())
    }

    /// Render profile details
    fn render_profile_details(&self, frame: &mut Frame, area: Rect, profile: &Profile) -> Result<()> {
        let details = vec![
            format!("Name: {}", profile.name),
            format!("Git User Name: {}", profile.git_user_name.as_deref().unwrap_or("Not set")),
            format!("Git User Email: {}", profile.git_user_email),
            format!("Key Type: {:?}", profile.key_type),
            format!("Signing Key: {}", profile.signing_key.as_deref().unwrap_or("Not set")),
            format!("Vault Name: {}", profile.vault_name.as_deref().unwrap_or("Not set")),
            format!("SSH Key Title: {}", profile.ssh_key_title.as_deref().unwrap_or("Not set")),
            format!("Scope: {}", profile.scope.as_ref().map(|s| format!("{:?}", s)).unwrap_or("Not set".to_string())),
            format!("1Password: {}", if profile.one_password { "Yes" } else { "No" }),
            format!("Match Patterns: {}", if profile.match_patterns.is_empty() { "None" } else { &profile.match_patterns.join(", ") }),
            format!("Repositories: {}", if profile.repos.is_empty() { "None" } else { &profile.repos.join(", ") }),
            format!("Include If Dirs: {}", if profile.include_if_dirs.is_empty() { "None" } else { &profile.include_if_dirs.join(", ") }),
            format!("Host Patterns: {}", if profile.host_patterns.is_empty() { "None" } else { &profile.host_patterns.join(", ") }),
        ];

        let items: Vec<ListItem> = details
            .into_iter()
            .map(|detail| ListItem::new(detail))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Profile Details")
            )
            .style(self.theme.styles.base);

        frame.render_widget(list, area);
        Ok(())
    }
}

impl Component for ProfileViewScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        self.render_profile_view(frame, area)
    }

    fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Char('e') => {
                        if self.profile.is_some() {
                            Ok(ComponentAction::NavigateTo(ScreenType::ProfileEdit(self.profile_name.clone())))
                        } else {
                            Ok(ComponentAction::None)
                        }
                    }
                    KeyCode::Char('d') => {
                        if self.profile.is_some() {
                            Ok(ComponentAction::ShowPopup(format!("Delete profile '{}'?", self.profile_name)))
                        } else {
                            Ok(ComponentAction::None)
                        }
                    }
                    KeyCode::Char('q') => {
                        Ok(ComponentAction::Exit)
                    }
                    KeyCode::Esc => {
                        Ok(ComponentAction::NavigateBack)
                    }
                    _ => Ok(ComponentAction::None),
                }
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

impl Screen for ProfileViewScreen {
    fn title(&self) -> &str {
        "Profile View"
    }

    fn screen_type(&self) -> ScreenType {
        ScreenType::ProfileView(self.profile_name.clone())
    }

    fn on_screen_enter(&mut self) -> Result<()> {
        self.refresh()
    }

    fn screen_help(&self) -> Vec<(&str, &str)> {
        vec![
            ("e", "Edit profile"),
            ("d", "Delete profile"),
            ("Esc", "Go back"),
            ("q", "Quit"),
        ]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::types::{KeyType, Scope},
        profile::ProfileManager,
    };
    use std::sync::Arc;

    struct MockProfileManager {
        profiles: Vec<Profile>,
    }

    impl MockProfileManager {
        fn new(profiles: Vec<Profile>) -> Self {
            Self { profiles }
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

    fn create_test_profile(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            git_user_name: Some("Test User".to_string()),
            git_user_email: "test@example.com".to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("ssh-ed25519 AAAAC3...".to_string()),
            vault_name: Some("Test Vault".to_string()),
            ssh_key_title: Some("Test SSH Key".to_string()),
            scope: Some(Scope::Local),
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec!["*.example.com".to_string()],
            repos: vec!["git@github.com:user/repo.git".to_string()],
            include_if_dirs: vec!["/home/user/work".to_string()],
            host_patterns: vec!["*.work.com".to_string()],
            one_password: true,
        }
    }

    #[test]
    fn test_profile_view_screen_creation() {
        let profile = create_test_profile("test");
        let manager = Arc::new(MockProfileManager::new(vec![profile]));
        let screen = ProfileViewScreen::new("test".to_string(), Theme::default(), manager).unwrap();
        
        assert_eq!(screen.profile_name(), "test");
        assert!(screen.profile().is_some());
        assert_eq!(screen.title(), "Profile View");
        assert_eq!(screen.screen_type(), ScreenType::ProfileView("test".to_string()));
    }

    #[test]
    fn test_profile_view_screen_not_found() {
        let manager = Arc::new(MockProfileManager::new(vec![]));
        let screen = ProfileViewScreen::new("missing".to_string(), Theme::default(), manager).unwrap();
        
        assert_eq!(screen.profile_name(), "missing");
        assert!(screen.profile().is_none());
    }

    #[test]
    fn test_profile_view_event_handling() {
        let profile = create_test_profile("test");
        let manager = Arc::new(MockProfileManager::new(vec![profile]));
        let mut screen = ProfileViewScreen::new("test".to_string(), Theme::default(), manager).unwrap();
        
        // Test edit event
        let edit_event = KeyEvent::new(KeyCode::Char('e'), crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(edit_event)).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileEdit("test".to_string())));
        
        // Test delete event
        let delete_event = KeyEvent::new(KeyCode::Char('d'), crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(delete_event)).unwrap();
        match result {
            ComponentAction::ShowPopup(msg) => assert!(msg.contains("Delete profile 'test'?")),
            _ => panic!("Expected ShowPopup action"),
        }
        
        // Test quit event
        let quit_event = KeyEvent::new(KeyCode::Char('q'), crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(quit_event)).unwrap();
        assert_eq!(result, ComponentAction::Exit);
        
        // Test escape event
        let escape_event = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(escape_event)).unwrap();
        assert_eq!(result, ComponentAction::NavigateBack);
    }

    #[test]
    fn test_profile_view_screen_help() {
        let profile = create_test_profile("test");
        let manager = Arc::new(MockProfileManager::new(vec![profile]));
        let screen = ProfileViewScreen::new("test".to_string(), Theme::default(), manager).unwrap();
        
        let help = screen.screen_help();
        assert!(!help.is_empty());
        assert!(help.iter().any(|(_, desc)| desc.contains("Edit")));
        assert!(help.iter().any(|(_, desc)| desc.contains("Delete")));
    }

    #[test]
    fn test_profile_view_events_with_missing_profile() {
        let manager = Arc::new(MockProfileManager::new(vec![]));
        let mut screen = ProfileViewScreen::new("missing".to_string(), Theme::default(), manager).unwrap();
        
        // Test edit event with missing profile
        let edit_event = KeyEvent::new(KeyCode::Char('e'), crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(edit_event)).unwrap();
        assert_eq!(result, ComponentAction::None);
        
        // Test delete event with missing profile
        let delete_event = KeyEvent::new(KeyCode::Char('d'), crossterm::event::KeyModifiers::empty());
        let result = screen.handle_event(Event::Key(delete_event)).unwrap();
        assert_eq!(result, ComponentAction::None);
    }

    #[test]
    fn test_profile_view_refresh() {
        let profile = create_test_profile("test");
        let manager = Arc::new(MockProfileManager::new(vec![profile]));
        let mut screen = ProfileViewScreen::new("test".to_string(), Theme::default(), manager).unwrap();
        
        // Refresh should work
        let result = screen.refresh();
        assert!(result.is_ok());
        assert!(screen.profile().is_some());
    }
}