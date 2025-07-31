//! Profile list screen for the TUI application.
//!
//! This module provides a comprehensive profile management interface
//! with search, filtering, and profile operations.

use crate::{
    error::Result,
    tui::{
        Component, ComponentAction, Event, Theme,
        screens::{Screen, ScreenType},
        widgets::input::{InputWidget, InputState},
    },
    config::types::Profile,
    profile::ProfileManager,
    matching::{MatchResult, ProfileFuzzyMatcher, FuzzyMatcher},
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, List, ListItem, Clear},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::any::Any;
use std::sync::Arc;

/// Sorting fields for profile list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    Email,
    KeyType,
    Scope,
    LastUsed,
}

/// Sorting direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Profile list screen state
pub struct ProfileListScreen {
    /// All profiles
    profiles: Vec<Profile>,
    /// Filtered profiles based on search
    filtered_profiles: Vec<Profile>,
    /// Currently selected profile index
    selected_index: usize,
    /// Search query
    search_query: String,
    /// Whether search mode is active
    search_active: bool,
    /// Input state for search
    search_input: InputState,
    /// Current sort field
    sort_field: SortField,
    /// Sort direction
    sort_direction: SortDirection,
    /// Theme
    theme: Theme,
    /// Profile manager
    profile_manager: Arc<dyn ProfileManager>,
    /// Fuzzy matcher for search
    fuzzy_matcher: ProfileFuzzyMatcher,
    /// Whether to show help
    show_help: bool,
}

impl ProfileListScreen {
    /// Create a new profile list screen
    pub fn new(theme: Theme, profile_manager: Arc<dyn ProfileManager>) -> Result<Self> {
        let profiles = profile_manager.list()?;
        let filtered_profiles = profiles.clone();
        
        Ok(Self {
            profiles,
            filtered_profiles,
            selected_index: 0,
            search_query: String::new(),
            search_active: false,
            search_input: InputState::new(),
            sort_field: SortField::Name,
            sort_direction: SortDirection::Ascending,
            theme,
            profile_manager,
            fuzzy_matcher: ProfileFuzzyMatcher::new(),
            show_help: false,
        })
    }

    /// Get the currently selected profile
    pub fn selected_profile(&self) -> Option<&Profile> {
        self.filtered_profiles.get(self.selected_index)
    }

    /// Get the selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get the number of profiles
    pub fn profile_count(&self) -> usize {
        self.filtered_profiles.len()
    }

    /// Get the search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Check if search is active
    pub fn is_search_active(&self) -> bool {
        self.search_active
    }

    /// Refresh the profile list
    pub fn refresh(&mut self) -> Result<()> {
        self.profiles = self.profile_manager.list()?;
        self.apply_filter_and_sort();
        Ok(())
    }

    /// Navigate to the previous profile
    pub fn previous_profile(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.filtered_profiles.is_empty() {
            self.selected_index = self.filtered_profiles.len() - 1;
        }
    }

    /// Navigate to the next profile
    pub fn next_profile(&mut self) {
        if self.selected_index < self.filtered_profiles.len().saturating_sub(1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.search_active = true;
        self.search_input = InputState::new();
        self.search_input.content = self.search_query.clone();
        self.search_input.cursor_position = self.search_query.len();
    }

    /// Stop search mode
    pub fn stop_search(&mut self) {
        self.search_active = false;
    }

    /// Handle search input
    pub fn handle_search_input(&mut self, ch: char) {
        if self.search_active {
            self.search_input.insert_char(ch);
            self.search_query = self.search_input.content.clone();
            self.apply_filter_and_sort();
        }
    }

    /// Handle search backspace
    pub fn handle_search_backspace(&mut self) {
        if self.search_active {
            self.search_input.delete_char();
            self.search_query = self.search_input.content.clone();
            self.apply_filter_and_sort();
        }
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_input.clear();
        self.apply_filter_and_sort();
    }

    /// Set sort field
    pub fn set_sort_field(&mut self, field: SortField) {
        if self.sort_field == field {
            // Toggle direction if same field
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_field = field;
            self.sort_direction = SortDirection::Ascending;
        }
        self.apply_filter_and_sort();
    }

    /// Apply filter and sort to profiles
    fn apply_filter_and_sort(&mut self) {
        // Apply search filter
        if self.search_query.is_empty() {
            self.filtered_profiles = self.profiles.clone();
        } else {
            let matches = self.fuzzy_matcher.find_matches(&self.search_query, &self.profiles);
            self.filtered_profiles = matches.into_iter().map(|m| m.profile).collect();
        }

        // Apply sort
        self.filtered_profiles.sort_by(|a, b| {
            let cmp = match self.sort_field {
                SortField::Name => a.name.cmp(&b.name),
                SortField::Email => a.git_user_email.cmp(&b.git_user_email),
                SortField::KeyType => format!("{:?}", a.key_type).cmp(&format!("{:?}", b.key_type)),
                SortField::Scope => {
                    let a_scope = a.scope.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
                    let b_scope = b.scope.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
                    a_scope.cmp(&b_scope)
                }
                SortField::LastUsed => {
                    // For now, sort by name as we don't have last used timestamp
                    a.name.cmp(&b.name)
                }
            };

            match self.sort_direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });

        // Adjust selected index if needed
        if self.selected_index >= self.filtered_profiles.len() && !self.filtered_profiles.is_empty() {
            self.selected_index = self.filtered_profiles.len() - 1;
        } else if self.filtered_profiles.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Handle profile action
    pub fn handle_profile_action(&self, action: ProfileAction) -> Result<ComponentAction> {
        match action {
            ProfileAction::Edit => {
                if let Some(profile) = self.selected_profile() {
                    Ok(ComponentAction::NavigateTo(ScreenType::ProfileEdit(profile.name.clone())))
                } else {
                    Ok(ComponentAction::None)
                }
            }
            ProfileAction::View => {
                if let Some(profile) = self.selected_profile() {
                    Ok(ComponentAction::NavigateTo(ScreenType::ProfileView(profile.name.clone())))
                } else {
                    Ok(ComponentAction::None)
                }
            }
            ProfileAction::Delete => {
                if let Some(profile) = self.selected_profile() {
                    Ok(ComponentAction::ShowPopup(format!("Delete profile '{}'?", profile.name)))
                } else {
                    Ok(ComponentAction::None)
                }
            }
            ProfileAction::Apply => {
                if let Some(profile) = self.selected_profile() {
                    Ok(ComponentAction::Return(profile.name.clone()))
                } else {
                    Ok(ComponentAction::None)
                }
            }
            ProfileAction::Create => {
                Ok(ComponentAction::NavigateTo(ScreenType::ProfileCreate))
            }
        }
    }

    /// Render the profile list
    fn render_profile_list(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Search bar
                Constraint::Length(3),  // Sort info
                Constraint::Min(0),     // Profile table
                Constraint::Length(3),  // Status bar
                if self.show_help { Constraint::Length(8) } else { Constraint::Length(0) },  // Help
            ])
            .split(area);

        // Render search bar
        self.render_search_bar(frame, chunks[0])?;

        // Render sort info
        self.render_sort_info(frame, chunks[1])?;

        // Render profile table
        self.render_profile_table(frame, chunks[2])?;

        // Render status bar
        self.render_status_bar(frame, chunks[3])?;

        // Render help if shown
        if self.show_help {
            self.render_help(frame, chunks[4])?;
        }

        Ok(())
    }

    /// Render search bar
    fn render_search_bar(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let search_text = if self.search_active {
            format!("Search: {}", self.search_query)
        } else if self.search_query.is_empty() {
            "Press '/' to search profiles".to_string()
        } else {
            format!("Search: {} (press '/' to edit)", self.search_query)
        };

        let search_widget = Paragraph::new(search_text)
            .style(if self.search_active {
                self.theme.styles.selected
            } else {
                self.theme.styles.base
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if self.search_active {
                        self.theme.styles.selected
                    } else {
                        self.theme.styles.border
                    })
                    .title("Search")
            );

        frame.render_widget(search_widget, area);

        // Show cursor if search is active
        if self.search_active {
            let cursor_x = area.x + 1 + "Search: ".len() as u16 + self.search_input.cursor_position as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width - 1 {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }

        Ok(())
    }

    /// Render sort info
    fn render_sort_info(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let sort_text = format!(
            "Sort by: {:?} {} | {} profile(s)",
            self.sort_field,
            match self.sort_direction {
                SortDirection::Ascending => "↑",
                SortDirection::Descending => "↓",
            },
            self.filtered_profiles.len()
        );

        let sort_widget = Paragraph::new(sort_text)
            .style(self.theme.styles.info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Sort")
            );

        frame.render_widget(sort_widget, area);
        Ok(())
    }

    /// Render profile table
    fn render_profile_table(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.filtered_profiles.is_empty() {
            let empty_message = if self.search_query.is_empty() {
                "No profiles found. Press 'n' to create a new profile."
            } else {
                "No profiles match your search. Press 'Esc' to clear search."
            };

            let empty_widget = Paragraph::new(empty_message)
                .style(self.theme.styles.info)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.theme.styles.border)
                        .title("Profiles")
                );

            frame.render_widget(empty_widget, area);
            return Ok(());
        }

        // Create header
        let header_cells = ["Name", "Email", "Key Type", "Scope", "1Password"]
            .iter()
            .map(|h| Cell::from(*h).style(self.theme.styles.title));
        let header = Row::new(header_cells)
            .style(self.theme.styles.base)
            .height(1)
            .bottom_margin(1);

        // Create rows
        let rows: Vec<Row> = self.filtered_profiles
            .iter()
            .enumerate()
            .map(|(index, profile)| {
                let style = if index == self.selected_index {
                    self.theme.styles.selected
                } else {
                    self.theme.styles.base
                };

                let cells = vec![
                    Cell::from(profile.name.as_str()),
                    Cell::from(profile.git_user_email.as_str()),
                    Cell::from(format!("{:?}", profile.key_type)),
                    Cell::from(profile.scope.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default()),
                    Cell::from(if profile.one_password { "Yes" } else { "No" }),
                ];

                Row::new(cells).style(style).height(1)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(30),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.styles.border)
                .title("Profiles")
        );

        frame.render_widget(table, area);
        Ok(())
    }

    /// Render status bar
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let status_text = if self.filtered_profiles.is_empty() {
            "No profiles".to_string()
        } else {
            format!(
                "Profile {} of {} | {} total",
                self.selected_index + 1,
                self.filtered_profiles.len(),
                self.profiles.len()
            )
        };

        let status_widget = Paragraph::new(status_text)
            .style(self.theme.styles.info)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Status")
            );

        frame.render_widget(status_widget, area);
        Ok(())
    }

    /// Render help
    fn render_help(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let help_text = vec![
            "Navigation: ↑/↓ or j/k to move, Enter/v to view, e to edit",
            "Actions: n to create, d to delete, a to apply, r to refresh",
            "Search: / to search, Esc to clear/exit search",
            "Sort: s to cycle sort field, S to reverse direction",
            "Other: ? to toggle help, q to quit",
        ].join("\n");

        let help_widget = Paragraph::new(help_text)
            .style(self.theme.styles.help)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Help")
            );

        frame.render_widget(help_widget, area);
        Ok(())
    }
}

/// Profile actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProfileAction {
    Edit,
    View,
    Delete,
    Apply,
    Create,
}

impl Component for ProfileListScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        self.render_profile_list(frame, area)
    }

    fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        match event {
            Event::Key(key_event) => {
                if self.search_active {
                    self.handle_search_event(key_event)
                } else {
                    self.handle_normal_event(key_event)
                }
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

impl ProfileListScreen {
    /// Handle events in search mode
    fn handle_search_event(&mut self, key_event: KeyEvent) -> Result<ComponentAction> {
        match key_event.code {
            KeyCode::Esc => {
                self.stop_search();
                Ok(ComponentAction::None)
            }
            KeyCode::Enter => {
                self.stop_search();
                Ok(ComponentAction::None)
            }
            KeyCode::Char(ch) => {
                self.handle_search_input(ch);
                Ok(ComponentAction::None)
            }
            KeyCode::Backspace => {
                self.handle_search_backspace();
                Ok(ComponentAction::None)
            }
            KeyCode::Left => {
                self.search_input.move_cursor_left();
                Ok(ComponentAction::None)
            }
            KeyCode::Right => {
                self.search_input.move_cursor_right();
                Ok(ComponentAction::None)
            }
            KeyCode::Home => {
                self.search_input.move_cursor_to_start();
                Ok(ComponentAction::None)
            }
            KeyCode::End => {
                self.search_input.move_cursor_to_end();
                Ok(ComponentAction::None)
            }
            _ => Ok(ComponentAction::None),
        }
    }

    /// Handle events in normal mode
    fn handle_normal_event(&mut self, key_event: KeyEvent) -> Result<ComponentAction> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous_profile();
                Ok(ComponentAction::None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next_profile();
                Ok(ComponentAction::None)
            }
            KeyCode::Enter | KeyCode::Char('v') => {
                self.handle_profile_action(ProfileAction::View)
            }
            KeyCode::Char('e') => {
                self.handle_profile_action(ProfileAction::Edit)
            }
            KeyCode::Char('d') => {
                self.handle_profile_action(ProfileAction::Delete)
            }
            KeyCode::Char('a') => {
                self.handle_profile_action(ProfileAction::Apply)
            }
            KeyCode::Char('n') => {
                self.handle_profile_action(ProfileAction::Create)
            }
            KeyCode::Char('/') => {
                self.start_search();
                Ok(ComponentAction::None)
            }
            KeyCode::Char('s') => {
                // Cycle through sort fields
                let next_field = match self.sort_field {
                    SortField::Name => SortField::Email,
                    SortField::Email => SortField::KeyType,
                    SortField::KeyType => SortField::Scope,
                    SortField::Scope => SortField::LastUsed,
                    SortField::LastUsed => SortField::Name,
                };
                self.set_sort_field(next_field);
                Ok(ComponentAction::None)
            }
            KeyCode::Char('S') => {
                // Reverse sort direction
                self.sort_direction = match self.sort_direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
                self.apply_filter_and_sort();
                Ok(ComponentAction::None)
            }
            KeyCode::Char('r') => {
                self.refresh()?;
                Ok(ComponentAction::Refresh)
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                Ok(ComponentAction::None)
            }
            KeyCode::Char('q') => {
                Ok(ComponentAction::Exit)
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.clear_search();
                    Ok(ComponentAction::None)
                } else {
                    Ok(ComponentAction::NavigateBack)
                }
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

impl Screen for ProfileListScreen {
    fn title(&self) -> &str {
        "Profile List"
    }

    fn screen_type(&self) -> ScreenType {
        ScreenType::ProfileList
    }

    fn on_screen_enter(&mut self) -> Result<()> {
        self.refresh()
    }

    fn screen_help(&self) -> Vec<(&str, &str)> {
        vec![
            ("↑/↓ or j/k", "Navigate profiles"),
            ("Enter/v", "View profile"),
            ("e", "Edit profile"),
            ("d", "Delete profile"),
            ("a", "Apply profile"),
            ("n", "Create new profile"),
            ("/", "Search profiles"),
            ("s", "Change sort field"),
            ("S", "Reverse sort direction"),
            ("r", "Refresh"),
            ("?", "Toggle help"),
            ("Esc", "Clear search or go back"),
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

    /// Mock profile manager for testing
    struct MockProfileManager {
        profiles: Vec<Profile>,
    }

    impl MockProfileManager {
        fn new(profiles: Vec<Profile>) -> Self {
            Self { profiles }
        }

        fn empty() -> Self {
            Self { profiles: vec![] }
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

    fn create_test_profile(name: &str, email: &str) -> Profile {
        Profile {
            name: name.to_string(),
            git_user_name: Some(name.to_string()),
            git_user_email: email.to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("ssh-ed25519 AAAAC3...".to_string()),
            vault_name: Some("Vault".to_string()),
            ssh_key_title: Some("SSH Key".to_string()),
            scope: Some(Scope::Local),
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password: false,
        }
    }

    fn create_test_screen() -> ProfileListScreen {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
            create_test_profile("personal", "personal@example.com"),
            create_test_profile("opensource", "opensource@example.com"),
        ];
        let manager = Arc::new(MockProfileManager::new(profiles));
        ProfileListScreen::new(Theme::default(), manager).unwrap()
    }

    #[test]
    fn test_profile_list_screen_creation() {
        let screen = create_test_screen();
        
        assert_eq!(screen.title(), "Profile List");
        assert_eq!(screen.screen_type(), ScreenType::ProfileList);
        assert_eq!(screen.profile_count(), 3);
        assert_eq!(screen.selected_index(), 0);
        assert_eq!(screen.search_query(), "");
        assert!(!screen.is_search_active());
    }

    #[test]
    fn test_profile_list_screen_empty() {
        let manager = Arc::new(MockProfileManager::empty());
        let screen = ProfileListScreen::new(Theme::default(), manager).unwrap();
        
        assert_eq!(screen.profile_count(), 0);
        assert_eq!(screen.selected_index(), 0);
        assert!(screen.selected_profile().is_none());
    }

    #[test]
    fn test_navigation() {
        let mut screen = create_test_screen();
        
        // Test next
        screen.next_profile();
        assert_eq!(screen.selected_index(), 1);
        
        screen.next_profile();
        assert_eq!(screen.selected_index(), 2);
        
        // Test wrap around
        screen.next_profile();
        assert_eq!(screen.selected_index(), 0);
        
        // Test previous
        screen.previous_profile();
        assert_eq!(screen.selected_index(), 2);
        
        screen.previous_profile();
        assert_eq!(screen.selected_index(), 1);
        
        screen.previous_profile();
        assert_eq!(screen.selected_index(), 0);
    }

    #[test]
    fn test_search_functionality() {
        let mut screen = create_test_screen();
        
        // Start search
        screen.start_search();
        assert!(screen.is_search_active());
        
        // Test search input
        screen.handle_search_input('w');
        screen.handle_search_input('o');
        screen.handle_search_input('r');
        screen.handle_search_input('k');
        
        assert_eq!(screen.search_query(), "work");
        assert_eq!(screen.profile_count(), 1);
        assert_eq!(screen.selected_profile().unwrap().name, "work");
        
        // Test backspace
        screen.handle_search_backspace();
        assert_eq!(screen.search_query(), "wor");
        
        // Test clear search
        screen.clear_search();
        assert_eq!(screen.search_query(), "");
        assert_eq!(screen.profile_count(), 3);
        
        // Stop search
        screen.stop_search();
        assert!(!screen.is_search_active());
    }

    #[test]
    fn test_sorting() {
        let mut screen = create_test_screen();
        
        // Test initial sort (Name, Ascending)
        assert_eq!(screen.sort_field, SortField::Name);
        assert_eq!(screen.sort_direction, SortDirection::Ascending);
        
        // Test sort by email
        screen.set_sort_field(SortField::Email);
        assert_eq!(screen.sort_field, SortField::Email);
        assert_eq!(screen.sort_direction, SortDirection::Ascending);
        
        // Test toggle direction
        screen.set_sort_field(SortField::Email);
        assert_eq!(screen.sort_field, SortField::Email);
        assert_eq!(screen.sort_direction, SortDirection::Descending);
        
        // Test different field
        screen.set_sort_field(SortField::KeyType);
        assert_eq!(screen.sort_field, SortField::KeyType);
        assert_eq!(screen.sort_direction, SortDirection::Ascending);
    }

    #[test]
    fn test_profile_actions() {
        let screen = create_test_screen();
        
        // Test view action
        let result = screen.handle_profile_action(ProfileAction::View).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileView("work".to_string())));
        
        // Test edit action
        let result = screen.handle_profile_action(ProfileAction::Edit).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileEdit("work".to_string())));
        
        // Test apply action
        let result = screen.handle_profile_action(ProfileAction::Apply).unwrap();
        assert_eq!(result, ComponentAction::Return("work".to_string()));
        
        // Test create action
        let result = screen.handle_profile_action(ProfileAction::Create).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileCreate));
        
        // Test delete action
        let result = screen.handle_profile_action(ProfileAction::Delete).unwrap();
        match result {
            ComponentAction::ShowPopup(msg) => assert!(msg.contains("Delete profile 'work'?")),
            _ => panic!("Expected ShowPopup action"),
        }
    }

    #[test]
    fn test_search_event_handling() {
        let mut screen = create_test_screen();
        screen.start_search();
        
        // Test character input
        let char_event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty());
        let result = screen.handle_search_event(char_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(screen.search_query(), "w");
        
        // Test backspace
        let backspace_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
        let result = screen.handle_search_event(backspace_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(screen.search_query(), "");
        
        // Test escape
        let escape_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        let result = screen.handle_search_event(escape_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert!(!screen.is_search_active());
        
        // Test enter
        screen.start_search();
        let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let result = screen.handle_search_event(enter_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert!(!screen.is_search_active());
    }

    #[test]
    fn test_normal_event_handling() {
        let mut screen = create_test_screen();
        
        // Test navigation
        let down_event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        let result = screen.handle_normal_event(down_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(screen.selected_index(), 1);
        
        let up_event = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        let result = screen.handle_normal_event(up_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(screen.selected_index(), 0);
        
        // Test vim navigation
        let j_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        let result = screen.handle_normal_event(j_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(screen.selected_index(), 1);
        
        let k_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        let result = screen.handle_normal_event(k_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(screen.selected_index(), 0);
        
        // Test search activation
        let search_event = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty());
        let result = screen.handle_normal_event(search_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert!(screen.is_search_active());
        
        // Test profile actions
        screen.stop_search();
        let edit_event = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty());
        let result = screen.handle_normal_event(edit_event).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileEdit("work".to_string())));
        
        let view_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty());
        let result = screen.handle_normal_event(view_event).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileView("work".to_string())));
        
        let apply_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        let result = screen.handle_normal_event(apply_event).unwrap();
        assert_eq!(result, ComponentAction::Return("work".to_string()));
        
        let create_event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        let result = screen.handle_normal_event(create_event).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileCreate));
        
        // Test sort
        let sort_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty());
        let result = screen.handle_normal_event(sort_event).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(screen.sort_field, SortField::Email);
        
        // Test quit
        let quit_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        let result = screen.handle_normal_event(quit_event).unwrap();
        assert_eq!(result, ComponentAction::Exit);
    }

    #[test]
    fn test_screen_help() {
        let screen = create_test_screen();
        let help = screen.screen_help();
        
        assert!(!help.is_empty());
        assert!(help.iter().any(|(_, desc)| desc.contains("Navigate")));
        assert!(help.iter().any(|(_, desc)| desc.contains("Search")));
        assert!(help.iter().any(|(_, desc)| desc.contains("Sort")));
    }

    #[test]
    fn test_empty_list_actions() {
        let manager = Arc::new(MockProfileManager::empty());
        let screen = ProfileListScreen::new(Theme::default(), manager).unwrap();
        
        // Test actions on empty list
        let result = screen.handle_profile_action(ProfileAction::View).unwrap();
        assert_eq!(result, ComponentAction::None);
        
        let result = screen.handle_profile_action(ProfileAction::Edit).unwrap();
        assert_eq!(result, ComponentAction::None);
        
        let result = screen.handle_profile_action(ProfileAction::Delete).unwrap();
        assert_eq!(result, ComponentAction::None);
        
        let result = screen.handle_profile_action(ProfileAction::Apply).unwrap();
        assert_eq!(result, ComponentAction::None);
        
        // Create should still work
        let result = screen.handle_profile_action(ProfileAction::Create).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileCreate));
    }

    #[test]
    fn test_search_with_no_results() {
        let mut screen = create_test_screen();
        
        screen.start_search();
        screen.handle_search_input('x');
        screen.handle_search_input('y');
        screen.handle_search_input('z');
        
        assert_eq!(screen.search_query(), "xyz");
        assert_eq!(screen.profile_count(), 0);
        assert!(screen.selected_profile().is_none());
    }

    #[test]
    fn test_refresh_functionality() {
        let mut screen = create_test_screen();
        
        // Initial state
        assert_eq!(screen.profile_count(), 3);
        
        // Refresh should work
        let result = screen.refresh();
        assert!(result.is_ok());
        assert_eq!(screen.profile_count(), 3);
    }

    #[test]
    fn test_sort_field_enum() {
        use std::mem;
        
        // Test enum variants
        let fields = [
            SortField::Name,
            SortField::Email,
            SortField::KeyType,
            SortField::Scope,
            SortField::LastUsed,
        ];
        
        for field in fields {
            // Test debug formatting
            let debug_str = format!("{:?}", field);
            assert!(!debug_str.is_empty());
            
            // Test equality
            assert_eq!(field, field);
        }
        
        // Test size (should be small)
        assert!(mem::size_of::<SortField>() <= 8);
    }

    #[test]
    fn test_sort_direction_enum() {
        use std::mem;
        
        let directions = [SortDirection::Ascending, SortDirection::Descending];
        
        for direction in directions {
            let debug_str = format!("{:?}", direction);
            assert!(!debug_str.is_empty());
            assert_eq!(direction, direction);
        }
        
        assert!(mem::size_of::<SortDirection>() <= 4);
    }

    #[test]
    fn test_profile_action_enum() {
        use std::mem;
        
        let actions = [
            ProfileAction::Edit,
            ProfileAction::View,
            ProfileAction::Delete,
            ProfileAction::Apply,
            ProfileAction::Create,
        ];
        
        for action in actions {
            let debug_str = format!("{:?}", action);
            assert!(!debug_str.is_empty());
            assert_eq!(action, action);
        }
        
        assert!(mem::size_of::<ProfileAction>() <= 4);
    }
}