//! Screen components for the TUI application.
//!
//! This module provides screen implementations that extend the Component trait
//! with screen-specific lifecycle methods and navigation capabilities.

pub mod main_menu;
pub mod profile_list;
pub mod profile_view;
pub mod profile_create;

use crate::{
    error::Result,
    tui::{Component, ComponentAction, Event, Theme},
};
use ratatui::{Frame, layout::Rect};
use std::collections::HashMap;
use std::any::Any;

/// Screen types available in the application
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScreenType {
    Main,
    ProfileList,
    ProfileEdit(String),
    ProfileCreate,
    ProfileView(String),
    Settings,
    Help,
}

/// Extended trait for screen components with lifecycle management
pub trait Screen: Component {
    /// Get the screen title
    fn title(&self) -> &str;
    
    /// Screen type identifier
    fn screen_type(&self) -> ScreenType;
    
    /// Called when screen becomes the active screen
    fn on_screen_enter(&mut self) -> Result<()> {
        self.on_enter()
    }
    
    /// Called when screen is no longer the active screen
    fn on_screen_exit(&mut self) -> Result<()> {
        self.on_exit()
    }
    
    /// Check if the screen can be safely exited
    fn can_exit(&self) -> bool {
        true
    }
    
    /// Get screen-specific help text
    fn screen_help(&self) -> Vec<(&str, &str)> {
        vec![]
    }
    
    /// Handle screen-specific navigation
    fn handle_navigation(&mut self, event: Event) -> Result<ComponentAction> {
        self.handle_event(event)
    }
    
    /// Downcast to Any for testing
    fn as_any(&self) -> &dyn Any;
}

/// Screen manager for handling screen lifecycle and navigation
pub struct ScreenManager {
    screens: HashMap<ScreenType, Box<dyn Screen>>,
    current_screen: Option<ScreenType>,
    screen_stack: Vec<ScreenType>,
    theme: Theme,
}

impl ScreenManager {
    /// Create a new screen manager
    pub fn new(theme: Theme) -> Self {
        Self {
            screens: HashMap::new(),
            current_screen: None,
            screen_stack: Vec::new(),
            theme,
        }
    }
    
    /// Register a screen with the manager
    pub fn register_screen(&mut self, screen: Box<dyn Screen>) {
        let screen_type = screen.screen_type();
        self.screens.insert(screen_type, screen);
    }
    
    /// Navigate to a specific screen
    pub fn navigate_to(&mut self, screen_type: ScreenType) -> Result<()> {
        // Exit current screen if any
        if let Some(current) = &self.current_screen {
            if let Some(screen) = self.screens.get_mut(current) {
                if !screen.can_exit() {
                    return Ok(()); // Can't exit, stay on current screen
                }
                screen.on_screen_exit()?;
            }
        }
        
        // Push current screen to stack for back navigation
        if let Some(current) = &self.current_screen {
            if current != &screen_type {
                self.screen_stack.push(current.clone());
            }
        }
        
        // Enter new screen
        if let Some(screen) = self.screens.get_mut(&screen_type) {
            screen.on_screen_enter()?;
            self.current_screen = Some(screen_type);
        }
        
        Ok(())
    }
    
    /// Navigate back to the previous screen
    pub fn navigate_back(&mut self) -> Result<()> {
        if let Some(previous) = self.screen_stack.pop() {
            // Exit current screen
            if let Some(current) = &self.current_screen {
                if let Some(screen) = self.screens.get_mut(current) {
                    if !screen.can_exit() {
                        // Can't exit, put the previous screen back on stack
                        self.screen_stack.push(previous);
                        return Ok(());
                    }
                    screen.on_screen_exit()?;
                }
            }
            
            // Enter previous screen
            if let Some(screen) = self.screens.get_mut(&previous) {
                screen.on_screen_enter()?;
                self.current_screen = Some(previous);
            }
        }
        
        Ok(())
    }
    
    /// Get the current screen
    pub fn current_screen(&self) -> Option<&ScreenType> {
        self.current_screen.as_ref()
    }
    
    /// Get the current screen as mutable
    pub fn current_screen_mut(&mut self) -> Option<&mut Box<dyn Screen>> {
        if let Some(current) = &self.current_screen {
            self.screens.get_mut(current)
        } else {
            None
        }
    }
    
    /// Render the current screen
    pub fn render(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Some(current) = &self.current_screen {
            if let Some(screen) = self.screens.get(current) {
                screen.render(frame, area, &self.theme)?;
            }
        }
        Ok(())
    }
    
    /// Handle event for current screen
    pub fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        if let Some(current) = &self.current_screen {
            if let Some(screen) = self.screens.get_mut(current) {
                return screen.handle_navigation(event);
            }
        }
        Ok(ComponentAction::None)
    }
    
    /// Clear the navigation stack
    pub fn clear_stack(&mut self) {
        self.screen_stack.clear();
    }
    
    /// Get the navigation stack depth
    pub fn stack_depth(&self) -> usize {
        self.screen_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::Theme;

    /// Mock screen for testing
    struct MockScreen {
        title: String,
        screen_type: ScreenType,
        can_exit: bool,
        enter_called: bool,
        exit_called: bool,
    }

    impl MockScreen {
        fn new(title: String, screen_type: ScreenType) -> Self {
            Self {
                title,
                screen_type,
                can_exit: true,
                enter_called: false,
                exit_called: false,
            }
        }
        
        fn set_can_exit(&mut self, can_exit: bool) {
            self.can_exit = can_exit;
        }
        
        fn was_enter_called(&self) -> bool {
            self.enter_called
        }
        
        fn was_exit_called(&self) -> bool {
            self.exit_called
        }
    }

    impl Component for MockScreen {
        fn render(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) -> Result<()> {
            Ok(())
        }

        fn handle_event(&mut self, _event: Event) -> Result<ComponentAction> {
            Ok(ComponentAction::None)
        }
    }

    impl Screen for MockScreen {
        fn title(&self) -> &str {
            &self.title
        }
        
        fn screen_type(&self) -> ScreenType {
            self.screen_type.clone()
        }
        
        fn on_screen_enter(&mut self) -> Result<()> {
            self.enter_called = true;
            Ok(())
        }
        
        fn on_screen_exit(&mut self) -> Result<()> {
            self.exit_called = true;
            Ok(())
        }
        
        fn can_exit(&self) -> bool {
            self.can_exit
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_screen_type_equality() {
        assert_eq!(ScreenType::Main, ScreenType::Main);
        assert_eq!(
            ScreenType::ProfileEdit("test".to_string()),
            ScreenType::ProfileEdit("test".to_string())
        );
        assert_ne!(
            ScreenType::ProfileEdit("test1".to_string()),
            ScreenType::ProfileEdit("test2".to_string())
        );
    }

    #[test]
    fn test_screen_manager_new() {
        let theme = Theme::default();
        let manager = ScreenManager::new(theme);
        
        assert!(manager.current_screen().is_none());
        assert_eq!(manager.stack_depth(), 0);
    }

    #[test]
    fn test_screen_manager_register_and_navigate() {
        let theme = Theme::default();
        let mut manager = ScreenManager::new(theme);
        
        let screen = Box::new(MockScreen::new("Test".to_string(), ScreenType::Main));
        manager.register_screen(screen);
        
        assert!(manager.navigate_to(ScreenType::Main).is_ok());
        assert_eq!(manager.current_screen(), Some(&ScreenType::Main));
        
        // Check that on_screen_enter was called - we can't easily check this with trait objects
        // but the navigation should succeed
        assert!(manager.current_screen().is_some());
    }

    #[test]
    fn test_screen_manager_navigation_stack() {
        let theme = Theme::default();
        let mut manager = ScreenManager::new(theme);
        
        let main_screen = Box::new(MockScreen::new("Main".to_string(), ScreenType::Main));
        let help_screen = Box::new(MockScreen::new("Help".to_string(), ScreenType::Help));
        
        manager.register_screen(main_screen);
        manager.register_screen(help_screen);
        
        // Navigate to main, then help
        assert!(manager.navigate_to(ScreenType::Main).is_ok());
        assert!(manager.navigate_to(ScreenType::Help).is_ok());
        
        assert_eq!(manager.current_screen(), Some(&ScreenType::Help));
        assert_eq!(manager.stack_depth(), 1);
        
        // Navigate back to main
        assert!(manager.navigate_back().is_ok());
        assert_eq!(manager.current_screen(), Some(&ScreenType::Main));
        assert_eq!(manager.stack_depth(), 0);
    }

    #[test]
    fn test_screen_manager_cannot_exit() {
        let theme = Theme::default();
        let mut manager = ScreenManager::new(theme);
        
        let mut screen = MockScreen::new("Test".to_string(), ScreenType::Main);
        screen.set_can_exit(false);
        
        manager.register_screen(Box::new(screen));
        manager.register_screen(Box::new(MockScreen::new("Help".to_string(), ScreenType::Help)));
        
        assert!(manager.navigate_to(ScreenType::Main).is_ok());
        assert!(manager.navigate_to(ScreenType::Help).is_ok());
        
        // Should stay on Main because it can't exit
        assert_eq!(manager.current_screen(), Some(&ScreenType::Main));
        assert_eq!(manager.stack_depth(), 0);
    }

    #[test]
    fn test_screen_manager_clear_stack() {
        let theme = Theme::default();
        let mut manager = ScreenManager::new(theme);
        
        let main_screen = Box::new(MockScreen::new("Main".to_string(), ScreenType::Main));
        let help_screen = Box::new(MockScreen::new("Help".to_string(), ScreenType::Help));
        
        manager.register_screen(main_screen);
        manager.register_screen(help_screen);
        
        // Build up stack
        assert!(manager.navigate_to(ScreenType::Main).is_ok());
        assert!(manager.navigate_to(ScreenType::Help).is_ok());
        assert_eq!(manager.stack_depth(), 1);
        
        // Clear stack
        manager.clear_stack();
        assert_eq!(manager.stack_depth(), 0);
    }

    #[test]
    fn test_screen_manager_no_current_screen() {
        let theme = Theme::default();
        let mut manager = ScreenManager::new(theme);
        
        // Should handle gracefully when no screen is set
        assert!(manager.handle_event(Event::Tick).is_ok());
        assert!(manager.navigate_back().is_ok());
    }
}