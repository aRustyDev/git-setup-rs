//! Main menu screen for the TUI application.
//!
//! This module provides the main menu interface where users can navigate
//! to different sections of the application.

use crate::{
    error::Result,
    tui::{
        Component, ComponentAction, Event, Theme, UIHelpers,
        events::{KeyAction, KeyBindings},
        screens::{Screen, ScreenType},
    },
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, Paragraph, List, ListItem, Clear},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::any::Any;

/// Menu item configuration
#[derive(Debug, Clone)]
struct MenuItem {
    pub label: String,
    pub description: String,
    pub action: MenuAction,
    pub key_hint: Option<char>,
    pub enabled: bool,
}

/// Actions that can be triggered from the main menu
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    ViewProfiles,
    CreateProfile,
    ImportProfiles,
    Settings,
    Help,
    Exit,
}

/// Main menu screen component
pub struct MainMenuScreen {
    title: String,
    menu_items: Vec<MenuItem>,
    selected_index: usize,
    theme: Theme,
    key_bindings: KeyBindings,
}

impl MainMenuScreen {
    /// Create a new main menu screen
    pub fn new(theme: Theme) -> Self {
        let menu_items = vec![
            MenuItem {
                label: "Profile Management".to_string(),
                description: "View, edit, and manage Git profiles".to_string(),
                action: MenuAction::ViewProfiles,
                key_hint: Some('1'),
                enabled: true,
            },
            MenuItem {
                label: "Create Profile".to_string(),
                description: "Create a new Git profile from scratch".to_string(),
                action: MenuAction::CreateProfile,
                key_hint: Some('2'),
                enabled: true,
            },
            MenuItem {
                label: "Import Profiles".to_string(),
                description: "Import profiles from existing Git configurations".to_string(),
                action: MenuAction::ImportProfiles,
                key_hint: Some('3'),
                enabled: true,
            },
            MenuItem {
                label: "Settings".to_string(),
                description: "Configure application settings and preferences".to_string(),
                action: MenuAction::Settings,
                key_hint: Some('s'),
                enabled: true,
            },
            MenuItem {
                label: "Help".to_string(),
                description: "View help documentation and keyboard shortcuts".to_string(),
                action: MenuAction::Help,
                key_hint: Some('?'),
                enabled: true,
            },
            MenuItem {
                label: "Exit".to_string(),
                description: "Exit the application".to_string(),
                action: MenuAction::Exit,
                key_hint: Some('q'),
                enabled: true,
            },
        ];

        Self {
            title: "Git Profile Manager".to_string(),
            menu_items,
            selected_index: 0,
            theme,
            key_bindings: KeyBindings::default(),
        }
    }

    /// Get the currently selected menu item
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.menu_items.get(self.selected_index)
    }

    /// Get the selected menu action
    pub fn selected_action(&self) -> Option<MenuAction> {
        self.selected_item().map(|item| item.action.clone())
    }

    /// Get the selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Navigate to the previous menu item
    pub fn previous_item(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.menu_items.len() - 1;
        }
    }

    /// Navigate to the next menu item
    pub fn next_item(&mut self) {
        if self.selected_index < self.menu_items.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    /// Select menu item by character key
    pub fn select_by_key(&mut self, key: char) -> bool {
        for (index, item) in self.menu_items.iter().enumerate() {
            if item.key_hint == Some(key) && item.enabled {
                self.selected_index = index;
                return true;
            }
        }
        false
    }

    /// Handle menu selection
    pub fn handle_selection(&self) -> Result<ComponentAction> {
        if let Some(item) = self.selected_item() {
            if !item.enabled {
                return Ok(ComponentAction::None);
            }

            match item.action {
                MenuAction::ViewProfiles => Ok(ComponentAction::NavigateTo(ScreenType::ProfileList)),
                MenuAction::CreateProfile => Ok(ComponentAction::NavigateTo(ScreenType::ProfileCreate)),
                MenuAction::ImportProfiles => Ok(ComponentAction::ShowPopup(
                    "Import functionality will be implemented in a future version".to_string()
                )),
                MenuAction::Settings => Ok(ComponentAction::NavigateTo(ScreenType::Settings)),
                MenuAction::Help => Ok(ComponentAction::NavigateTo(ScreenType::Help)),
                MenuAction::Exit => Ok(ComponentAction::Exit),
            }
        } else {
            Ok(ComponentAction::None)
        }
    }

    /// Render the main menu content
    fn render_menu(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(0),     // Menu items
                Constraint::Length(4),  // Description
                Constraint::Length(3),  // Status/Help
            ])
            .split(area);

        // Render title
        self.render_title(frame, chunks[0])?;

        // Render menu items
        self.render_menu_items(frame, chunks[1])?;

        // Render description
        self.render_description(frame, chunks[2])?;

        // Render status/help
        self.render_status(frame, chunks[3])?;

        Ok(())
    }

    /// Render the title section
    fn render_title(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let title = Paragraph::new(self.title.as_str())
            .style(self.theme.styles.title)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
            );

        frame.render_widget(title, area);
        Ok(())
    }

    /// Render the menu items list
    fn render_menu_items(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let items: Vec<ListItem> = self.menu_items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = index == self.selected_index;
                let marker = if is_selected { "▶" } else { " " };
                
                let key_hint = if let Some(key) = item.key_hint {
                    format!(" [{}]", key)
                } else {
                    String::new()
                };

                let content = format!("{} {}{}", marker, item.label, key_hint);
                
                let style = if is_selected {
                    if item.enabled {
                        self.theme.styles.selected
                    } else {
                        self.theme.styles.selected.fg(self.theme.colors.muted)
                    }
                } else if item.enabled {
                    self.theme.styles.base
                } else {
                    self.theme.styles.base.fg(self.theme.colors.muted)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let menu_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Menu")
            )
            .style(self.theme.styles.base);

        frame.render_widget(menu_list, area);
        Ok(())
    }

    /// Render the description section
    fn render_description(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let description = if let Some(item) = self.selected_item() {
            item.description.clone()
        } else {
            "No item selected".to_string()
        };

        let desc_widget = Paragraph::new(description)
            .style(self.theme.styles.info)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Description")
            );

        frame.render_widget(desc_widget, area);
        Ok(())
    }

    /// Render the status/help section
    fn render_status(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let help_text = "↑/↓ Navigate • Enter Select • q Exit • ? Help";
        
        let status_widget = Paragraph::new(help_text)
            .style(self.theme.styles.help)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.styles.border)
                    .title("Controls")
            );

        frame.render_widget(status_widget, area);
        Ok(())
    }
}

impl Component for MainMenuScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        // Use the screen's theme if different from provided theme
        self.render_menu(frame, area)
    }

    fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        match event {
            Event::Key(key_event) => {
                // First try to handle with key bindings
                if let Some(action) = self.key_bindings.get_action(&key_event) {
                    match action {
                        KeyAction::NavigateUp => {
                            self.previous_item();
                            Ok(ComponentAction::None)
                        }
                        KeyAction::NavigateDown => {
                            self.next_item();
                            Ok(ComponentAction::None)
                        }
                        KeyAction::Select => {
                            self.handle_selection()
                        }
                        KeyAction::Quit => {
                            Ok(ComponentAction::Exit)
                        }
                        KeyAction::Help => {
                            Ok(ComponentAction::NavigateTo(ScreenType::Help))
                        }
                        _ => Ok(ComponentAction::None),
                    }
                } else {
                    // Handle custom key mappings
                    match key_event.code {
                        KeyCode::Char(c) => {
                            if self.select_by_key(c) {
                                self.handle_selection()
                            } else {
                                Ok(ComponentAction::None)
                            }
                        }
                        KeyCode::Enter => {
                            self.handle_selection()
                        }
                        _ => Ok(ComponentAction::None),
                    }
                }
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

impl Screen for MainMenuScreen {
    fn title(&self) -> &str {
        &self.title
    }

    fn screen_type(&self) -> ScreenType {
        ScreenType::Main
    }

    fn screen_help(&self) -> Vec<(&str, &str)> {
        vec![
            ("↑/↓ or j/k", "Navigate menu items"),
            ("Enter", "Select menu item"),
            ("1-3", "Quick selection"),
            ("s", "Settings"),
            ("?", "Help"),
            ("q", "Exit"),
        ]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::Theme;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_menu() -> MainMenuScreen {
        MainMenuScreen::new(Theme::default())
    }

    #[test]
    fn test_main_menu_creation() {
        let menu = create_test_menu();
        
        assert_eq!(menu.title, "Git Profile Manager");
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.menu_items.len(), 6);
        assert_eq!(menu.selected_action(), Some(MenuAction::ViewProfiles));
    }

    #[test]
    fn test_navigation() {
        let mut menu = create_test_menu();
        
        // Test next item
        menu.next_item();
        assert_eq!(menu.selected_index, 1);
        assert_eq!(menu.selected_action(), Some(MenuAction::CreateProfile));
        
        // Test previous item
        menu.previous_item();
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.selected_action(), Some(MenuAction::ViewProfiles));
        
        // Test wrap around - previous from first item
        menu.previous_item();
        assert_eq!(menu.selected_index, 5);
        assert_eq!(menu.selected_action(), Some(MenuAction::Exit));
        
        // Test wrap around - next from last item
        menu.next_item();
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.selected_action(), Some(MenuAction::ViewProfiles));
    }

    #[test]
    fn test_key_selection() {
        let mut menu = create_test_menu();
        
        // Test valid key selection
        assert!(menu.select_by_key('2'));
        assert_eq!(menu.selected_index, 1);
        assert_eq!(menu.selected_action(), Some(MenuAction::CreateProfile));
        
        // Test help key
        assert!(menu.select_by_key('?'));
        assert_eq!(menu.selected_action(), Some(MenuAction::Help));
        
        // Test exit key
        assert!(menu.select_by_key('q'));
        assert_eq!(menu.selected_action(), Some(MenuAction::Exit));
        
        // Test invalid key
        assert!(!menu.select_by_key('x'));
    }

    #[test]
    fn test_handle_selection() {
        let menu = create_test_menu();
        
        // Test each menu action
        let test_cases = vec![
            (0, MenuAction::ViewProfiles, ComponentAction::NavigateTo(ScreenType::ProfileList)),
            (1, MenuAction::CreateProfile, ComponentAction::NavigateTo(ScreenType::ProfileCreate)),
            (3, MenuAction::Settings, ComponentAction::NavigateTo(ScreenType::Settings)),
            (4, MenuAction::Help, ComponentAction::NavigateTo(ScreenType::Help)),
            (5, MenuAction::Exit, ComponentAction::Exit),
        ];
        
        for (index, expected_action, expected_component_action) in test_cases {
            let mut test_menu = create_test_menu();
            test_menu.selected_index = index;
            
            assert_eq!(test_menu.selected_action(), Some(expected_action));
            
            let result = test_menu.handle_selection().unwrap();
            match (result, expected_component_action) {
                (ComponentAction::NavigateTo(screen1), ComponentAction::NavigateTo(screen2)) => {
                    assert_eq!(screen1, screen2);
                }
                (ComponentAction::Exit, ComponentAction::Exit) => {}
                _ => panic!("Unexpected component action"),
            }
        }
    }

    #[test]
    fn test_import_profiles_shows_popup() {
        let mut menu = create_test_menu();
        menu.selected_index = 2; // Import profiles
        
        let result = menu.handle_selection().unwrap();
        match result {
            ComponentAction::ShowPopup(msg) => {
                assert!(msg.contains("Import functionality"));
            }
            _ => panic!("Expected ShowPopup action"),
        }
    }

    #[test]
    fn test_key_event_handling() {
        let mut menu = create_test_menu();
        
        // Test navigation keys
        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        let result = menu.handle_event(Event::Key(up_key)).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(menu.selected_index, 5); // Wrapped to last item
        
        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        let result = menu.handle_event(Event::Key(down_key)).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(menu.selected_index, 0); // Wrapped to first item
        
        // Test vim navigation
        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        let result = menu.handle_event(Event::Key(j_key)).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(menu.selected_index, 1);
        
        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        let result = menu.handle_event(Event::Key(k_key)).unwrap();
        assert_eq!(result, ComponentAction::None);
        assert_eq!(menu.selected_index, 0);
        
        // Test selection
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let result = menu.handle_event(Event::Key(enter_key)).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::ProfileList));
        
        // Test quit key
        let q_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        let result = menu.handle_event(Event::Key(q_key)).unwrap();
        assert_eq!(result, ComponentAction::Exit);
        
        // Test help key
        let help_key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
        let result = menu.handle_event(Event::Key(help_key)).unwrap();
        assert_eq!(result, ComponentAction::NavigateTo(ScreenType::Help));
    }

    #[test]
    fn test_screen_trait_implementation() {
        let menu = create_test_menu();
        
        assert_eq!(menu.title(), "Git Profile Manager");
        assert_eq!(menu.screen_type(), ScreenType::Main);
        
        let help_text = menu.screen_help();
        assert!(!help_text.is_empty());
        assert!(help_text.iter().any(|(_, desc)| desc.contains("Navigate")));
    }

    #[test]
    fn test_menu_item_properties() {
        let menu = create_test_menu();
        
        // Test all menu items are enabled by default
        for item in &menu.menu_items {
            assert!(item.enabled);
        }
        
        // Test menu items have labels and descriptions
        for item in &menu.menu_items {
            assert!(!item.label.is_empty());
            assert!(!item.description.is_empty());
        }
        
        // Test specific key hints
        assert_eq!(menu.menu_items[0].key_hint, Some('1'));
        assert_eq!(menu.menu_items[1].key_hint, Some('2'));
        assert_eq!(menu.menu_items[2].key_hint, Some('3'));
        assert_eq!(menu.menu_items[3].key_hint, Some('s'));
        assert_eq!(menu.menu_items[4].key_hint, Some('?'));
        assert_eq!(menu.menu_items[5].key_hint, Some('q'));
    }

    #[test]
    fn test_selected_item_bounds() {
        let mut menu = create_test_menu();
        
        // Test valid selection
        assert!(menu.selected_item().is_some());
        
        // Test selection beyond bounds (should still work due to wrapping)
        menu.selected_index = 100;
        assert!(menu.selected_item().is_some()); // Will select last item due to bounds
        
        // Test with empty menu (this is more of a hypothetical test)
        // In practice, the menu should always have items
    }
}