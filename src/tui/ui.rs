use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint},
    widgets::{Block, Borders, Paragraph},
};
use crate::error::Result;
use super::theme::Theme;

/// UI rendering utilities
pub struct UI;

impl UI {
    /// Create a standard layout with header, content, and footer
    pub fn standard_layout(area: Rect) -> (Rect, Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Footer/Status bar
            ])
            .split(area);

        (chunks[0], chunks[1], chunks[2])
    }

    /// Create a two-column layout
    pub fn two_column_layout(area: Rect, left_percentage: u16) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(left_percentage),
                Constraint::Percentage(100 - left_percentage),
            ])
            .split(area);

        (chunks[0], chunks[1])
    }

    /// Create a layout with sidebar
    pub fn sidebar_layout(area: Rect, sidebar_width: u16) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width),
                Constraint::Min(0),
            ])
            .split(area);

        (chunks[0], chunks[1])
    }

    /// Draw a title bar
    pub fn draw_title(f: &mut Frame, area: Rect, title: &str, theme: &Theme) {
        let title_block = Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(theme.styles.border)
            .title_style(theme.styles.title);

        f.render_widget(title_block, area);
    }

    /// Draw a status bar
    pub fn draw_status_bar(f: &mut Frame, area: Rect, status: &str, theme: &Theme) {
        let status_bar = Paragraph::new(status)
            .style(theme.styles.help)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.styles.border)
            );

        f.render_widget(status_bar, area);
    }

    /// Draw an error message
    pub fn draw_error(f: &mut Frame, area: Rect, error: &str, theme: &Theme) {
        let error_widget = Paragraph::new(format!(" Error: {} ", error))
            .style(theme.styles.error)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.styles.error)
                    .title(" Error ")
                    .title_style(theme.styles.error)
            );

        f.render_widget(error_widget, area);
    }

    /// Draw a loading indicator
    pub fn draw_loading(f: &mut Frame, area: Rect, message: &str, theme: &Theme) {
        let loading = Paragraph::new(format!(" {} ", message))
            .style(theme.styles.info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.styles.info)
            );

        f.render_widget(loading, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_layout() {
        let area = Rect::new(0, 0, 80, 24);
        let (header, content, footer) = UI::standard_layout(area);

        assert_eq!(header.height, 3);
        assert_eq!(footer.height, 3);
        assert_eq!(content.height, 18); // 24 - 3 - 3
        assert_eq!(header.y, 0);
        assert_eq!(content.y, 3);
        assert_eq!(footer.y, 21);
    }

    #[test]
    fn test_two_column_layout() {
        let area = Rect::new(0, 0, 100, 24);
        let (left, right) = UI::two_column_layout(area, 30);

        assert_eq!(left.width, 30);
        assert_eq!(right.width, 70);
        assert_eq!(left.x, 0);
        assert_eq!(right.x, 30);
    }

    #[test]
    fn test_sidebar_layout() {
        let area = Rect::new(0, 0, 100, 24);
        let (sidebar, main) = UI::sidebar_layout(area, 20);

        assert_eq!(sidebar.width, 20);
        assert_eq!(main.width, 80);
        assert_eq!(sidebar.x, 0);
        assert_eq!(main.x, 20);
    }
}
