use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Modifier},
    widgets::{Block, Widget, StatefulWidget, List, ListItem},
};

/// State for selectable list widget
#[derive(Debug, Default, Clone)]
pub struct ListState {
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub offset: usize,
}

impl ListState {
    pub fn new(items: Vec<String>) -> Self {
        let selected = if items.is_empty() { None } else { Some(0) };
        Self {
            items,
            selected,
            offset: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        match self.selected {
            Some(current) => {
                if current + 1 < self.items.len() {
                    self.selected = Some(current + 1);
                }
            }
            None => self.selected = Some(0),
        }
    }

    pub fn select_previous(&mut self) {
        match self.selected {
            Some(current) => {
                if current > 0 {
                    self.selected = Some(current - 1);
                }
            }
            None => {
                if !self.items.is_empty() {
                    self.selected = Some(self.items.len() - 1);
                }
            }
        }
    }

    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.selected = Some(0);
            self.offset = 0;
        }
    }

    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.selected = Some(self.items.len() - 1);
        }
    }

    pub fn get_selected(&self) -> Option<&String> {
        self.selected.and_then(|i| self.items.get(i))
    }

    pub fn update_items(&mut self, items: Vec<String>) {
        self.items = items;
        // Reset selection if out of bounds
        if let Some(selected) = self.selected {
            if selected >= self.items.len() {
                self.selected = if self.items.is_empty() {
                    None
                } else {
                    Some(self.items.len() - 1)
                };
            }
        } else if !self.items.is_empty() {
            self.selected = Some(0);
        }
    }

    /// Calculate offset for scrolling
    pub fn calculate_offset(&mut self, visible_height: usize) {
        if let Some(selected) = self.selected {
            // Ensure selected item is visible
            if selected < self.offset {
                self.offset = selected;
            } else if selected >= self.offset + visible_height {
                self.offset = selected.saturating_sub(visible_height - 1);
            }
        }
    }
}

/// Selectable list widget
pub struct SelectableList<'a> {
    block: Option<Block<'a>>,
    style: Style,
    highlight_style: Style,
    highlight_symbol: &'a str,
}

impl<'a> SelectableList<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            style: Style::default(),
            highlight_style: Style::default().add_modifier(Modifier::REVERSED),
            highlight_symbol: "> ",
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

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = symbol;
        self
    }
}

impl<'a> StatefulWidget for SelectableList<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Calculate visible area
        let list_area = match &self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.clone().render(area, buf);
                inner_area
            }
            None => area,
        };

        if list_area.height == 0 || state.items.is_empty() {
            return;
        }

        // Update offset for scrolling
        state.calculate_offset(list_area.height as usize);

        // Create list items
        let items: Vec<ListItem> = state.items
            .iter()
            .skip(state.offset)
            .take(list_area.height as usize)
            .enumerate()
            .map(|(i, item)| {
                let global_index = i + state.offset;
                let is_selected = state.selected == Some(global_index);

                let content = if is_selected {
                    format!("{}{}", self.highlight_symbol, item)
                } else {
                    format!("{:width$}{}", "", item, width = self.highlight_symbol.len())
                };

                let style = if is_selected {
                    self.highlight_style
                } else {
                    self.style
                };

                ListItem::new(content).style(style)
            })
            .collect();

        // Render the list
        let list = List::new(items);
        Widget::render(list, list_area, buf);

        // Add scroll indicators if needed
        if state.offset > 0 {
            // Show up arrow
            let x = list_area.right().saturating_sub(1);
            let y = list_area.top();
            if x >= list_area.left() && y >= list_area.top() && y < list_area.bottom() {
                buf.get_mut(x, y)
                    .set_char('↑')
                    .set_style(self.style);
            }
        }

        if state.offset + (list_area.height as usize) < state.items.len() {
            // Show down arrow
            let x = list_area.right().saturating_sub(1);
            let y = list_area.bottom().saturating_sub(1);
            if x >= list_area.left() && y >= list_area.top() && y < list_area.bottom() {
                buf.get_mut(x, y)
                    .set_char('↓')
                    .set_style(self.style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_state_new() {
        let items = vec!["Item 1".to_string(), "Item 2".to_string()];
        let state = ListState::new(items.clone());

        assert_eq!(state.items, items);
        assert_eq!(state.selected, Some(0));
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn test_list_state_empty() {
        let state = ListState::new(vec![]);
        assert!(state.items.is_empty());
        assert_eq!(state.selected, None);
    }

    #[test]
    fn test_list_state_navigation() {
        let items = vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
        ];
        let mut state = ListState::new(items);

        // Initial state
        assert_eq!(state.selected, Some(0));

        // Move next
        state.select_next();
        assert_eq!(state.selected, Some(1));

        state.select_next();
        assert_eq!(state.selected, Some(2));

        // Try to move past end
        state.select_next();
        assert_eq!(state.selected, Some(2));

        // Move previous
        state.select_previous();
        assert_eq!(state.selected, Some(1));

        // Move to first
        state.select_first();
        assert_eq!(state.selected, Some(0));

        // Move to last
        state.select_last();
        assert_eq!(state.selected, Some(2));
    }

    #[test]
    fn test_list_state_get_selected() {
        let items = vec!["Item 1".to_string(), "Item 2".to_string()];
        let mut state = ListState::new(items);

        assert_eq!(state.get_selected(), Some(&"Item 1".to_string()));

        state.select_next();
        assert_eq!(state.get_selected(), Some(&"Item 2".to_string()));

        state.selected = None;
        assert_eq!(state.get_selected(), None);
    }

    #[test]
    fn test_list_state_update_items() {
        let mut state = ListState::new(vec!["Item 1".to_string()]);
        state.selected = Some(0);

        // Update with more items
        state.update_items(vec![
            "New 1".to_string(),
            "New 2".to_string(),
            "New 3".to_string(),
        ]);
        assert_eq!(state.selected, Some(0));
        assert_eq!(state.items.len(), 3);

        // Update with fewer items (selection out of bounds)
        state.selected = Some(2);
        state.update_items(vec!["Only".to_string()]);
        assert_eq!(state.selected, Some(0));

        // Update with empty
        state.update_items(vec![]);
        assert_eq!(state.selected, None);
    }

    #[test]
    fn test_list_state_offset_calculation() {
        let items: Vec<String> = (0..10).map(|i| format!("Item {}", i)).collect();
        let mut state = ListState::new(items);

        // Select item that requires scrolling
        state.selected = Some(7);
        state.calculate_offset(5); // Visible height of 5

        // Should scroll to show item 7
        assert!(state.offset <= 7);
        assert!(state.offset + 5 > 7);

        // Select first item
        state.selected = Some(0);
        state.calculate_offset(5);
        assert_eq!(state.offset, 0);
    }
}
