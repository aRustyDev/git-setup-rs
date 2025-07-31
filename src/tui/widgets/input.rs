use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget, StatefulWidget},
};
use unicode_width::UnicodeWidthStr;

/// State for input widget
#[derive(Debug, Default, Clone)]
pub struct InputState {
    pub content: String,
    pub cursor_position: usize,
    pub is_focused: bool,
    pub is_password: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_content(content: String) -> Self {
        let cursor_position = content.len();
        Self {
            content,
            cursor_position,
            ..Default::default()
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.content.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.content.len();
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
    }
}

/// Input widget for text entry
pub struct InputWidget<'a> {
    block: Option<Block<'a>>,
    style: Style,
    cursor_style: Style,
    placeholder: Option<&'a str>,
}

impl<'a> InputWidget<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            style: Style::default(),
            cursor_style: Style::default().bg(ratatui::style::Color::White),
            placeholder: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }
}

impl<'a> StatefulWidget for InputWidget<'a> {
    type State = InputState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Draw block if present
        let text_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_area.height < 1 {
            return;
        }

        // Determine what to display
        let (display_text, is_placeholder) = if state.content.is_empty() && self.placeholder.is_some() {
            (self.placeholder.unwrap().to_string(), true)
        } else if state.is_password {
            ("*".repeat(state.content.len()), false)
        } else {
            (state.content.clone(), false)
        };

        let style = if is_placeholder {
            self.style.fg(ratatui::style::Color::DarkGray)
        } else {
            self.style
        };

        // Calculate visible text window
        let text_width = display_text.width();
        let available_width = text_area.width as usize;

        let (start_byte, end_byte, cursor_offset) = if text_width <= available_width {
            (0, display_text.len(), state.cursor_position)
        } else {
            // Need to handle scrolling
            let cursor_pos = if state.is_password {
                state.cursor_position
            } else {
                state.content[..state.cursor_position].width()
            };

            if cursor_pos < available_width / 2 {
                // Cursor near start
                (0, display_text.len(), state.cursor_position)
            } else if cursor_pos > text_width - available_width / 2 {
                // Cursor near end
                let start = display_text.len().saturating_sub(available_width);
                (start, display_text.len(), state.cursor_position - start)
            } else {
                // Cursor in middle
                let start = state.cursor_position.saturating_sub(available_width / 2);
                (start, start + available_width, available_width / 2)
            }
        };

        // Render visible text
        let visible_text = &display_text[start_byte..end_byte.min(display_text.len())];
        for (i, ch) in visible_text.chars().enumerate() {
            let x = text_area.x + i as u16;
            if x < text_area.x + text_area.width {
                buf.get_mut(x, text_area.y)
                    .set_char(ch)
                    .set_style(style);
            }
        }

        // Draw cursor if focused and not showing placeholder
        if state.is_focused && !is_placeholder {
            let cursor_x = if state.is_password {
                cursor_offset
            } else {
                visible_text[..cursor_offset.min(visible_text.len())].width()
            };

            if cursor_x < available_width {
                let x = text_area.x + cursor_x as u16;
                if x < text_area.x + text_area.width {
                    buf.get_mut(x, text_area.y)
                        .set_style(self.cursor_style);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_new() {
        let state = InputState::new();
        assert_eq!(state.content, "");
        assert_eq!(state.cursor_position, 0);
        assert!(!state.is_focused);
        assert!(!state.is_password);
    }

    #[test]
    fn test_input_state_with_content() {
        let state = InputState::with_content("Hello".to_string());
        assert_eq!(state.content, "Hello");
        assert_eq!(state.cursor_position, 5);
    }

    #[test]
    fn test_input_state_insert_char() {
        let mut state = InputState::new();

        state.insert_char('H');
        assert_eq!(state.content, "H");
        assert_eq!(state.cursor_position, 1);

        state.insert_char('i');
        assert_eq!(state.content, "Hi");
        assert_eq!(state.cursor_position, 2);

        // Insert in middle
        state.cursor_position = 1;
        state.insert_char('e');
        assert_eq!(state.content, "Hei");
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_input_state_delete_char() {
        let mut state = InputState::with_content("Hello".to_string());

        // Delete from end
        state.delete_char();
        assert_eq!(state.content, "Hell");
        assert_eq!(state.cursor_position, 4);

        // Delete from middle
        state.cursor_position = 2;
        state.delete_char();
        assert_eq!(state.content, "Hll");
        assert_eq!(state.cursor_position, 1);

        // Try to delete at beginning
        state.cursor_position = 0;
        state.delete_char();
        assert_eq!(state.content, "Hll"); // No change
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_input_state_delete_char_forward() {
        let mut state = InputState::with_content("Hello".to_string());
        state.cursor_position = 0;

        state.delete_char_forward();
        assert_eq!(state.content, "ello");
        assert_eq!(state.cursor_position, 0);

        // Delete from middle
        state.cursor_position = 2;
        state.delete_char_forward();
        assert_eq!(state.content, "elo");
        assert_eq!(state.cursor_position, 2);

        // Try to delete at end
        state.cursor_position = 3;
        state.delete_char_forward();
        assert_eq!(state.content, "elo"); // No change
    }

    #[test]
    fn test_input_state_cursor_movement() {
        let mut state = InputState::with_content("Hello".to_string());
        assert_eq!(state.cursor_position, 5);

        // Move left
        state.move_cursor_left();
        assert_eq!(state.cursor_position, 4);

        // Move to start
        state.move_cursor_start();
        assert_eq!(state.cursor_position, 0);

        // Try to move left at start
        state.move_cursor_left();
        assert_eq!(state.cursor_position, 0);

        // Move right
        state.move_cursor_right();
        assert_eq!(state.cursor_position, 1);

        // Move to end
        state.move_cursor_end();
        assert_eq!(state.cursor_position, 5);

        // Try to move right at end
        state.move_cursor_right();
        assert_eq!(state.cursor_position, 5);
    }

    #[test]
    fn test_input_state_clear() {
        let mut state = InputState::with_content("Hello".to_string());
        state.clear();
        assert_eq!(state.content, "");
        assert_eq!(state.cursor_position, 0);
    }
}
