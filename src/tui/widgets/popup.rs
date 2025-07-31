use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    style::Style,
    widgets::{Block, Borders, Paragraph, Clear, Wrap},
};
use crate::tui::{UIHelpers, Theme};

/// Type of popup to display
#[derive(Debug, Clone, PartialEq)]
pub enum PopupType {
    Info,
    Warning,
    Error,
    Confirm,
}

/// Popup widget for modal dialogs
pub struct PopupWidget<'a> {
    title: &'a str,
    content: &'a str,
    popup_type: PopupType,
    width_percent: u16,
    height_percent: u16,
}

impl<'a> PopupWidget<'a> {
    pub fn new(title: &'a str, content: &'a str, popup_type: PopupType) -> Self {
        Self {
            title,
            content,
            popup_type,
            width_percent: 60,
            height_percent: 20,
        }
    }

    pub fn info(title: &'a str, content: &'a str) -> Self {
        Self::new(title, content, PopupType::Info)
    }

    pub fn warning(title: &'a str, content: &'a str) -> Self {
        Self::new(title, content, PopupType::Warning)
    }

    pub fn error(title: &'a str, content: &'a str) -> Self {
        Self::new(title, content, PopupType::Error)
    }

    pub fn confirm(title: &'a str, content: &'a str) -> Self {
        Self::new(title, content, PopupType::Confirm)
    }

    pub fn size(mut self, width_percent: u16, height_percent: u16) -> Self {
        self.width_percent = width_percent.min(100);
        self.height_percent = height_percent.min(100);
        self
    }

    pub fn render(&self, f: &mut Frame, theme: &Theme) {
        let area = UIHelpers::centered_rect(self.width_percent, self.height_percent, f.area());

        // Clear the background
        f.render_widget(Clear, area);

        // Determine style based on popup type
        let (border_style, title_style) = match self.popup_type {
            PopupType::Info => (theme.styles.info, theme.styles.info),
            PopupType::Warning => (theme.styles.warning, theme.styles.warning),
            PopupType::Error => (theme.styles.error, theme.styles.error),
            PopupType::Confirm => (theme.styles.info, theme.styles.info),
        };

        // Create the popup block
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_style(title_style);

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        // Split inner area for content and buttons (if confirm type)
        let chunks = if self.popup_type == PopupType::Confirm {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(inner_area)
        } else {
            std::rc::Rc::from([inner_area].as_slice())
        };

        // Render content
        let content = Paragraph::new(self.content)
            .style(theme.styles.base)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        f.render_widget(content, chunks[0]);

        // Render buttons for confirm popup
        if self.popup_type == PopupType::Confirm && chunks.len() > 1 {
            let button_text = "[Y]es  [N]o";
            let buttons = Paragraph::new(button_text)
                .style(theme.styles.base)
                .alignment(Alignment::Center);

            f.render_widget(buttons, chunks[1]);
        }
    }
}

/// Helper to render a simple message popup
pub fn show_message(f: &mut Frame, title: &str, message: &str, theme: &Theme) {
    PopupWidget::info(title, message).render(f, theme);
}

/// Helper to render an error popup
pub fn show_error(f: &mut Frame, title: &str, error: &str, theme: &Theme) {
    PopupWidget::error(title, error).render(f, theme);
}

/// Helper to render a confirmation popup
pub fn show_confirmation(f: &mut Frame, title: &str, question: &str, theme: &Theme) {
    PopupWidget::confirm(title, question).render(f, theme);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_type_equality() {
        assert_eq!(PopupType::Info, PopupType::Info);
        assert_ne!(PopupType::Info, PopupType::Error);
    }

    #[test]
    fn test_popup_widget_creation() {
        let popup = PopupWidget::info("Test", "Content");
        assert_eq!(popup.title, "Test");
        assert_eq!(popup.content, "Content");
        assert_eq!(popup.popup_type, PopupType::Info);
        assert_eq!(popup.width_percent, 60);
        assert_eq!(popup.height_percent, 20);
    }

    #[test]
    fn test_popup_widget_size() {
        let popup = PopupWidget::info("Test", "Content")
            .size(80, 40);
        assert_eq!(popup.width_percent, 80);
        assert_eq!(popup.height_percent, 40);

        // Test clamping
        let popup = PopupWidget::info("Test", "Content")
            .size(120, 150);
        assert_eq!(popup.width_percent, 100);
        assert_eq!(popup.height_percent, 100);
    }

    #[test]
    fn test_popup_type_constructors() {
        let info = PopupWidget::info("Title", "Content");
        assert_eq!(info.popup_type, PopupType::Info);

        let warning = PopupWidget::warning("Title", "Content");
        assert_eq!(warning.popup_type, PopupType::Warning);

        let error = PopupWidget::error("Title", "Content");
        assert_eq!(error.popup_type, PopupType::Error);

        let confirm = PopupWidget::confirm("Title", "Content");
        assert_eq!(confirm.popup_type, PopupType::Confirm);
    }
}
