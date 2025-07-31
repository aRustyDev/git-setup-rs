use super::{
    ComponentAction, EventHandler, Event, TerminalManager, Theme, UIHelpers, UI,
};
use super::events::{KeyBindings, KeyAction};
use crate::{
    error::{Result, GitSetupError},
    cli::Args,
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint},
    widgets::{Block, Borders, Paragraph, List, ListItem},
};
use std::time::Duration;

/// Available screens in the application
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Main,
    ProfileList,
    ProfileEdit(String),
    ProfileCreate,
    Settings,
    Help,
}

/// Application state
pub struct AppState {
    pub current_screen: Screen,
    pub previous_screen: Option<Screen>,
    pub selected_profile: Option<String>,
    pub status_message: Option<String>,
    pub is_loading: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Main,
            previous_screen: None,
            selected_profile: None,
            status_message: None,
            is_loading: false,
        }
    }
}

/// Main TUI application
pub struct App {
    state: AppState,
    terminal_manager: TerminalManager,
    event_handler: EventHandler,
    key_bindings: KeyBindings,
    theme: Theme,
    args: Args,
    should_exit: bool,
    return_value: Option<String>,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let terminal_manager = TerminalManager::new()?;
        let event_handler = EventHandler::new(Duration::from_millis(250))?;
        let key_bindings = KeyBindings::default();
        let theme = Theme::default();

        Ok(Self {
            state: AppState::default(),
            terminal_manager,
            event_handler,
            key_bindings,
            theme,
            args,
            should_exit: false,
            return_value: None,
        })
    }

    pub fn run(&mut self) -> Result<Option<String>> {
        // Main event loop
        loop {
            // Draw UI - capture the state we need for rendering
            let current_screen = self.state.current_screen.clone();
            let status_message = self.state.status_message.clone();
            let theme = self.theme.clone();
            let key_bindings = self.key_bindings.clone();

            self.terminal_manager.terminal().draw(|f| {
                if let Err(e) = Self::render_frame_static(f, &current_screen, &status_message, &theme, &key_bindings) {
                    // Draw error screen
                    UI::draw_error(f, f.area(), &e.to_string(), &theme);
                }
            })?;

            // Handle events
            if let Some(event) = self.event_handler.next()? {
                if let Err(e) = self.handle_event(event) {
                    self.state.status_message = Some(format!("Error: {}", e));
                }
            }

            // Check exit condition
            if self.should_exit {
                break;
            }
        }

        // Restore terminal
        self.terminal_manager.restore()?;

        Ok(self.return_value.take())
    }

    fn render_frame_static(
        f: &mut Frame,
        current_screen: &Screen,
        status_message: &Option<String>,
        theme: &Theme,
        key_bindings: &KeyBindings
    ) -> Result<()> {
        let (header, content, footer) = UI::standard_layout(f.area());

        // Draw header
        Self::draw_header(f, header, current_screen, theme)?;

        // Draw current screen
        Self::draw_screen(f, content, current_screen, theme, key_bindings)?;

        // Draw status bar
        Self::draw_status_bar(f, footer, status_message, theme)?;

        Ok(())
    }

    fn draw_header(f: &mut Frame, area: Rect, current_screen: &Screen, theme: &Theme) -> Result<()> {
        UI::draw_title(f, area, &format!("Git Setup - {}", Self::get_screen_title(current_screen)), theme);
        Ok(())
    }

    fn draw_screen(f: &mut Frame, area: Rect, current_screen: &Screen, theme: &Theme, key_bindings: &KeyBindings) -> Result<()> {
        match current_screen {
            Screen::Main => Self::draw_main_menu(f, area, theme),
            Screen::ProfileList => Self::draw_profile_list(f, area, theme),
            Screen::ProfileEdit(name) => Self::draw_profile_edit(f, area, name, theme),
            Screen::ProfileCreate => Self::draw_profile_create(f, area, theme),
            Screen::Settings => Self::draw_settings(f, area, theme),
            Screen::Help => Self::draw_help(f, area, theme, key_bindings),
        }
    }

    fn draw_main_menu(f: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        let menu_items = vec![
            ListItem::new("1. List Profiles"),
            ListItem::new("2. Create Profile"),
            ListItem::new("3. Settings"),
            ListItem::new("4. Help"),
        ];

        let menu = List::new(menu_items)
            .block(UIHelpers::bordered_block("Main Menu"))
            .style(theme.styles.base)
            .highlight_style(theme.styles.selected)
            .highlight_symbol("> ");

        f.render_widget(menu, area);

        // Draw instructions at the bottom
        if area.height > 10 {
            let instructions = Paragraph::new("Use arrow keys or numbers to select, Enter to confirm, 'q' to quit")
                .style(theme.styles.help);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(2)])
                .split(area);

            f.render_widget(instructions, chunks[1]);
        }

        Ok(())
    }

    fn draw_status_bar(f: &mut Frame, area: Rect, status_message: &Option<String>, theme: &Theme) -> Result<()> {
        let status_text = if let Some(msg) = status_message {
            msg.clone()
        } else {
            "Press '?' for help | 'q' to quit".to_string()
        };

        UI::draw_status_bar(f, area, &status_text, theme);
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => {
                if let Some(action) = self.key_bindings.get_action(&key_event) {
                    self.handle_action(action)?;
                } else {
                    // Handle number keys for main menu
                    if self.state.current_screen == Screen::Main {
                        use crossterm::event::KeyCode;
                        match key_event.code {
                            KeyCode::Char('1') => self.navigate_to(Screen::ProfileList)?,
                            KeyCode::Char('2') => self.navigate_to(Screen::ProfileCreate)?,
                            KeyCode::Char('3') => self.navigate_to(Screen::Settings)?,
                            KeyCode::Char('4') => self.navigate_to(Screen::Help)?,
                            _ => {}
                        }
                    }
                }
            }
            Event::Resize(width, height) => {
                // Terminal automatically handles resize
                self.state.status_message = Some(format!("Resized to {}x{}", width, height));
            }
            Event::Tick => {
                // Clear status message after a few ticks
                if self.state.status_message.is_some() {
                    self.state.status_message = None;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_action(&mut self, action: KeyAction) -> Result<()> {
        match action {
            KeyAction::Quit => {
                self.should_exit = true;
            }
            KeyAction::Help => {
                self.navigate_to(Screen::Help)?;
            }
            KeyAction::Back => {
                self.navigate_back()?;
            }
            _ => {
                // Delegate to current screen handler
                // This would be implemented per-screen
            }
        }

        Ok(())
    }

    fn navigate_to(&mut self, screen: Screen) -> Result<()> {
        self.state.previous_screen = Some(self.state.current_screen.clone());
        self.state.current_screen = screen;
        Ok(())
    }

    fn navigate_back(&mut self) -> Result<()> {
        if let Some(prev) = self.state.previous_screen.take() {
            self.state.current_screen = prev;
        } else {
            // If no previous screen, go to main
            self.state.current_screen = Screen::Main;
        }
        Ok(())
    }

    fn get_screen_title(current_screen: &Screen) -> &str {
        match current_screen {
            Screen::Main => "Main Menu",
            Screen::ProfileList => "Profile List",
            Screen::ProfileEdit(_) => "Edit Profile",
            Screen::ProfileCreate => "Create Profile",
            Screen::Settings => "Settings",
            Screen::Help => "Help",
        }
    }


    // Placeholder methods for screens - will be implemented in T2
    fn draw_profile_list(f: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        let placeholder = Paragraph::new("Profile list will be implemented in T2")
            .style(theme.styles.base)
            .block(UIHelpers::bordered_block("Profiles"));
        f.render_widget(placeholder, area);
        Ok(())
    }

    fn draw_profile_edit(f: &mut Frame, area: Rect, name: &str, theme: &Theme) -> Result<()> {
        let placeholder = Paragraph::new(format!("Editing profile: {}", name))
            .style(theme.styles.base)
            .block(UIHelpers::bordered_block("Edit Profile"));
        f.render_widget(placeholder, area);
        Ok(())
    }

    fn draw_profile_create(f: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        let placeholder = Paragraph::new("Profile creation will be implemented in T2")
            .style(theme.styles.base)
            .block(UIHelpers::bordered_block("Create Profile"));
        f.render_widget(placeholder, area);
        Ok(())
    }

    fn draw_settings(f: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        let placeholder = Paragraph::new("Settings will be implemented in T2")
            .style(theme.styles.base)
            .block(UIHelpers::bordered_block("Settings"));
        f.render_widget(placeholder, area);
        Ok(())
    }

    fn draw_help(f: &mut Frame, area: Rect, theme: &Theme, key_bindings: &KeyBindings) -> Result<()> {
        let help_text = key_bindings.get_help_text()
            .into_iter()
            .map(|(key, desc)| format!("{:15} - {}", key, desc))
            .collect::<Vec<_>>()
            .join("\n");

        let help = Paragraph::new(help_text)
            .style(theme.styles.base)
            .block(UIHelpers::bordered_block("Help"));

        f.render_widget(help, area);
        Ok(())
    }
}

impl super::TuiApp for App {
    fn new() -> Result<Self> {
        // This would need the CommandContext from elsewhere
        Err(GitSetupError::ExternalCommand {
            command: "TuiApp::new".to_string(),
            error: "Use App::new(args) instead".to_string(),
        })
    }

    fn run(&mut self) -> Result<Option<String>> {
        self.run()
    }

    fn handle_resize(&mut self, _width: u16, _height: u16) -> Result<()> {
        // Handled automatically by ratatui
        Ok(())
    }

    fn theme(&self) -> &Theme {
        &self.theme
    }

    fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.current_screen, Screen::Main);
        assert!(state.previous_screen.is_none());
        assert!(state.selected_profile.is_none());
        assert!(state.status_message.is_none());
        assert!(!state.is_loading);
    }

    #[test]
    fn test_screen_navigation() {
        let mut state = AppState::default();
        assert_eq!(state.current_screen, Screen::Main);

        // Navigate to help
        state.previous_screen = Some(state.current_screen.clone());
        state.current_screen = Screen::Help;
        assert_eq!(state.current_screen, Screen::Help);
        assert_eq!(state.previous_screen, Some(Screen::Main));

        // Navigate to profile list
        state.previous_screen = Some(state.current_screen.clone());
        state.current_screen = Screen::ProfileList;
        assert_eq!(state.current_screen, Screen::ProfileList);
        assert_eq!(state.previous_screen, Some(Screen::Help));
    }

    #[test]
    fn test_screen_title() {
        assert_eq!(App::get_screen_title(&Screen::Main), "Main Menu");
        assert_eq!(App::get_screen_title(&Screen::Help), "Help");
        assert_eq!(App::get_screen_title(&Screen::ProfileList), "Profile List");
    }

    #[test]
    fn test_screen_equality() {
        assert_eq!(Screen::Main, Screen::Main);
        assert_ne!(Screen::Main, Screen::Help);
        assert_eq!(
            Screen::ProfileEdit("test".to_string()),
            Screen::ProfileEdit("test".to_string())
        );
        assert_ne!(
            Screen::ProfileEdit("test1".to_string()),
            Screen::ProfileEdit("test2".to_string())
        );
    }
}
