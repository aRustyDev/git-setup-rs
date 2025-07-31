//! Enhanced table component for the TUI application.
//!
//! This module provides a table component with support for selection, sorting,
//! filtering, and pagination.

use crate::{
    error::Result,
    tui::{Component, ComponentAction, Event, Theme, UIHelpers},
};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, Row, Table, TableState as RatatuiTableState, Cell, Paragraph},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::cmp::Ordering;

/// Column definition for the table
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub title: String,
    pub width: Constraint,
    pub sortable: bool,
    pub alignment: Alignment,
}

impl TableColumn {
    /// Create a new table column
    pub fn new(title: &str, width: Constraint) -> Self {
        Self {
            title: title.to_string(),
            width,
            sortable: false,
            alignment: Alignment::Left,
        }
    }

    /// Make the column sortable
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Set column alignment
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Table row data
#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
    pub metadata: Option<String>,
}

impl TableRow {
    /// Create a new table row
    pub fn new(cells: Vec<String>) -> Self {
        Self {
            cells,
            metadata: None,
        }
    }

    /// Set metadata for the row
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// Toggle sort direction
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// Table state management
#[derive(Debug)]
pub struct TableState {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub filtered_rows: Vec<usize>,
    pub selected: Option<usize>,
    pub sort_column: Option<usize>,
    pub sort_direction: SortDirection,
    pub scroll_offset: usize,
    pub page_size: usize,
    pub filter_text: String,
    pub show_filter: bool,
    pub multi_select: bool,
    pub selected_rows: Vec<usize>,
}

impl TableState {
    /// Create a new table state
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            filtered_rows: Vec::new(),
            selected: None,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            scroll_offset: 0,
            page_size: 10,
            filter_text: String::new(),
            show_filter: false,
            multi_select: false,
            selected_rows: Vec::new(),
        }
    }

    /// Set columns
    pub fn set_columns(&mut self, columns: Vec<TableColumn>) {
        self.columns = columns;
    }

    /// Set rows
    pub fn set_rows(&mut self, rows: Vec<TableRow>) {
        self.rows = rows;
        self.apply_filter();
    }

    /// Add a row
    pub fn add_row(&mut self, row: TableRow) {
        self.rows.push(row);
        self.apply_filter();
    }

    /// Remove a row
    pub fn remove_row(&mut self, index: usize) {
        if index < self.rows.len() {
            self.rows.remove(index);
            self.apply_filter();
        }
    }

    /// Get the currently selected row
    pub fn selected_row(&self) -> Option<&TableRow> {
        self.selected
            .and_then(|selected| self.filtered_rows.get(selected))
            .and_then(|&row_index| self.rows.get(row_index))
    }

    /// Get the currently selected row index in the original data
    pub fn selected_row_index(&self) -> Option<usize> {
        self.selected
            .and_then(|selected| self.filtered_rows.get(selected))
            .copied()
    }

    /// Select next row
    pub fn select_next(&mut self) {
        if self.filtered_rows.is_empty() {
            return;
        }

        let current = self.selected.unwrap_or(0);
        let next = (current + 1).min(self.filtered_rows.len() - 1);
        self.selected = Some(next);
        self.ensure_selected_visible();
    }

    /// Select previous row
    pub fn select_previous(&mut self) {
        if self.filtered_rows.is_empty() {
            return;
        }

        let current = self.selected.unwrap_or(0);
        let prev = if current > 0 { current - 1 } else { 0 };
        self.selected = Some(prev);
        self.ensure_selected_visible();
    }

    /// Select first row
    pub fn select_first(&mut self) {
        if !self.filtered_rows.is_empty() {
            self.selected = Some(0);
            self.scroll_offset = 0;
        }
    }

    /// Select last row
    pub fn select_last(&mut self) {
        if !self.filtered_rows.is_empty() {
            self.selected = Some(self.filtered_rows.len() - 1);
            self.ensure_selected_visible();
        }
    }

    /// Page down
    pub fn page_down(&mut self) {
        if let Some(selected) = self.selected {
            let new_selected = (selected + self.page_size).min(self.filtered_rows.len() - 1);
            self.selected = Some(new_selected);
            self.ensure_selected_visible();
        }
    }

    /// Page up
    pub fn page_up(&mut self) {
        if let Some(selected) = self.selected {
            let new_selected = selected.saturating_sub(self.page_size);
            self.selected = Some(new_selected);
            self.ensure_selected_visible();
        }
    }

    /// Toggle row selection (for multi-select)
    pub fn toggle_row_selection(&mut self) {
        if !self.multi_select {
            return;
        }

        if let Some(selected) = self.selected {
            if let Some(row_index) = self.filtered_rows.get(selected) {
                if let Some(pos) = self.selected_rows.iter().position(|&x| x == *row_index) {
                    self.selected_rows.remove(pos);
                } else {
                    self.selected_rows.push(*row_index);
                }
            }
        }
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        self.selected_rows.clear();
    }

    /// Sort by column
    pub fn sort_by_column(&mut self, column_index: usize) {
        if column_index >= self.columns.len() || !self.columns[column_index].sortable {
            return;
        }

        // Toggle direction if sorting by same column
        if self.sort_column == Some(column_index) {
            self.sort_direction = self.sort_direction.toggle();
        } else {
            self.sort_column = Some(column_index);
            self.sort_direction = SortDirection::Ascending;
        }

        self.apply_sort();
    }

    /// Set filter text
    pub fn set_filter(&mut self, filter: &str) {
        self.filter_text = filter.to_string();
        self.apply_filter();
    }

    /// Toggle filter visibility
    pub fn toggle_filter(&mut self) {
        self.show_filter = !self.show_filter;
        if !self.show_filter {
            self.filter_text.clear();
            self.apply_filter();
        }
    }

    /// Apply current filter
    fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_rows = (0..self.rows.len()).collect();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_rows = self.rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row.cells.iter().any(|cell| {
                        cell.to_lowercase().contains(&filter_lower)
                    })
                })
                .map(|(index, _)| index)
                .collect();
        }

        // Reset selection if current selection is no longer valid
        if let Some(selected) = self.selected {
            if selected >= self.filtered_rows.len() {
                self.selected = if self.filtered_rows.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
        }

        self.apply_sort();
    }

    /// Apply current sort
    fn apply_sort(&mut self) {
        if let Some(sort_column) = self.sort_column {
            if sort_column < self.columns.len() {
                self.filtered_rows.sort_by(|&a, &b| {
                    let row_a = &self.rows[a];
                    let row_b = &self.rows[b];
                    
                    let default_cell = String::new();
                    let cell_a = row_a.cells.get(sort_column).unwrap_or(&default_cell);
                    let cell_b = row_b.cells.get(sort_column).unwrap_or(&default_cell);
                    
                    let cmp = cell_a.cmp(cell_b);
                    match self.sort_direction {
                        SortDirection::Ascending => cmp,
                        SortDirection::Descending => cmp.reverse(),
                    }
                });
            }
        }

        self.ensure_selected_visible();
    }

    /// Ensure selected row is visible
    fn ensure_selected_visible(&mut self) {
        if let Some(selected) = self.selected {
            if selected < self.scroll_offset {
                self.scroll_offset = selected;
            } else if selected >= self.scroll_offset + self.page_size {
                self.scroll_offset = selected.saturating_sub(self.page_size - 1);
            }
        }
    }

    /// Get visible rows for current page
    pub fn visible_rows(&self) -> Vec<&TableRow> {
        let start = self.scroll_offset;
        let end = (start + self.page_size).min(self.filtered_rows.len());
        
        self.filtered_rows[start..end]
            .iter()
            .map(|&index| &self.rows[index])
            .collect()
    }

    /// Get total number of filtered rows
    pub fn total_rows(&self) -> usize {
        self.filtered_rows.len()
    }

    /// Check if a row is selected (for multi-select)
    pub fn is_row_selected(&self, row_index: usize) -> bool {
        self.selected_rows.contains(&row_index)
    }
}

/// Table component
pub struct TableComponent {
    state: TableState,
    title: String,
    show_header: bool,
    show_row_numbers: bool,
    show_status: bool,
    filter_cursor: usize,
}

impl TableComponent {
    /// Create a new table component
    pub fn new(title: &str) -> Self {
        Self {
            state: TableState::new(),
            title: title.to_string(),
            show_header: true,
            show_row_numbers: false,
            show_status: true,
            filter_cursor: 0,
        }
    }

    /// Set table columns
    pub fn set_columns(&mut self, columns: Vec<TableColumn>) -> &mut Self {
        self.state.set_columns(columns);
        self
    }

    /// Set table rows
    pub fn set_rows(&mut self, rows: Vec<TableRow>) -> &mut Self {
        self.state.set_rows(rows);
        self
    }

    /// Enable multi-select
    pub fn enable_multi_select(&mut self) -> &mut Self {
        self.state.multi_select = true;
        self
    }

    /// Show row numbers
    pub fn show_row_numbers(&mut self) -> &mut Self {
        self.show_row_numbers = true;
        self
    }

    /// Get table state
    pub fn state(&self) -> &TableState {
        &self.state
    }

    /// Get mutable table state
    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    /// Handle filter input
    fn handle_filter_input(&mut self, ch: char) -> Result<ComponentAction> {
        if self.filter_cursor <= self.state.filter_text.len() {
            self.state.filter_text.insert(self.filter_cursor, ch);
            self.filter_cursor += 1;
            self.state.apply_filter();
        }
        Ok(ComponentAction::None)
    }

    /// Handle filter backspace
    fn handle_filter_backspace(&mut self) -> Result<ComponentAction> {
        if self.filter_cursor > 0 {
            self.state.filter_text.remove(self.filter_cursor - 1);
            self.filter_cursor -= 1;
            self.state.apply_filter();
        }
        Ok(ComponentAction::None)
    }

    /// Render the table
    fn render_table(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        let visible_rows = self.state.visible_rows();
        let table_rows: Vec<Row> = visible_rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let row_index = self.state.scroll_offset + i;
                let actual_row_index = self.state.filtered_rows.get(row_index).copied().unwrap_or(0);
                
                let mut cells: Vec<Cell> = row.cells.iter().map(|c| Cell::from(c.as_str())).collect();
                
                // Add row number if enabled
                if self.show_row_numbers {
                    cells.insert(0, Cell::from(format!("{}", actual_row_index + 1)));
                }
                
                let style = if self.state.multi_select && self.state.is_row_selected(actual_row_index) {
                    theme.styles.selected
                } else {
                    theme.styles.base
                };
                
                Row::new(cells).style(style)
            })
            .collect();

        // Prepare column constraints
        let mut constraints = self.state.columns.iter().map(|col| col.width).collect::<Vec<_>>();
        if self.show_row_numbers {
            constraints.insert(0, Constraint::Length(4));
        }

        // Prepare header
        let mut header_cells: Vec<Cell> = self.state.columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let mut title = col.title.clone();
                if let Some(sort_col) = self.state.sort_column {
                    if sort_col == i {
                        let arrow = match self.state.sort_direction {
                            SortDirection::Ascending => "↑",
                            SortDirection::Descending => "↓",
                        };
                        title = format!("{} {}", title, arrow);
                    }
                }
                Cell::from(title)
            })
            .collect();

        if self.show_row_numbers {
            header_cells.insert(0, Cell::from("#"));
        }

        let header = Row::new(header_cells)
            .style(theme.styles.header)
            .height(1);

        // Create table
        let table = Table::new(table_rows, constraints)
            .header(if self.show_header { header } else { Row::new(vec![]) })
            .block(Block::default().borders(Borders::ALL).title(self.title.clone()))
            .highlight_style(theme.styles.selected)
            .highlight_symbol("> ");

        // Render table
        let mut table_state = RatatuiTableState::default();
        if let Some(selected) = self.state.selected {
            table_state.select(Some(selected.saturating_sub(self.state.scroll_offset)));
        }

        frame.render_stateful_widget(table, area, &mut table_state);

        Ok(())
    }

    /// Render status bar
    fn render_status(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        let total = self.state.total_rows();
        let selected = self.state.selected.map(|s| s + 1).unwrap_or(0);
        
        let status_text = if self.state.multi_select {
            format!("{}/{} rows | {} selected", selected, total, self.state.selected_rows.len())
        } else {
            format!("{}/{} rows", selected, total)
        };

        let status = Paragraph::new(status_text)
            .style(theme.styles.status)
            .alignment(Alignment::Center);

        frame.render_widget(status, area);
        Ok(())
    }

    /// Render filter bar
    fn render_filter(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        let filter_text = if self.state.filter_text.is_empty() {
            "Type to filter..."
        } else {
            &self.state.filter_text
        };

        let filter = Paragraph::new(filter_text)
            .block(Block::default().borders(Borders::ALL).title("Filter"))
            .style(theme.styles.base);

        frame.render_widget(filter, area);

        // Show cursor
        let cursor_x = area.x + 1 + self.filter_cursor as u16;
        let cursor_y = area.y + 1;
        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor(cursor_x, cursor_y);
        }

        Ok(())
    }
}

impl Component for TableComponent {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<()> {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                if self.state.show_filter { Constraint::Length(3) } else { Constraint::Length(0) },
                Constraint::Min(0),
                if self.show_status { Constraint::Length(1) } else { Constraint::Length(0) },
            ])
            .split(area);

        // Render filter if visible
        if self.state.show_filter {
            self.render_filter(frame, chunks[0], theme)?;
        }

        // Render table
        self.render_table(frame, chunks[1], theme)?;

        // Render status bar
        if self.show_status {
            self.render_status(frame, chunks[2], theme)?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<ComponentAction> {
        match event {
            Event::Key(key_event) => {
                if self.state.show_filter {
                    match key_event.code {
                        KeyCode::Char(ch) => {
                            return self.handle_filter_input(ch);
                        }
                        KeyCode::Backspace => {
                            return self.handle_filter_backspace();
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            self.state.toggle_filter();
                            self.filter_cursor = 0;
                            return Ok(ComponentAction::None);
                        }
                        _ => {}
                    }
                } else {
                    match key_event.code {
                        KeyCode::Up => {
                            self.state.select_previous();
                        }
                        KeyCode::Down => {
                            self.state.select_next();
                        }
                        KeyCode::Home => {
                            self.state.select_first();
                        }
                        KeyCode::End => {
                            self.state.select_last();
                        }
                        KeyCode::PageUp => {
                            self.state.page_up();
                        }
                        KeyCode::PageDown => {
                            self.state.page_down();
                        }
                        KeyCode::Enter => {
                            if let Some(row_index) = self.state.selected_row_index() {
                                return Ok(ComponentAction::Return(format!("row_selected:{}", row_index)));
                            }
                        }
                        KeyCode::Char(' ') => {
                            if self.state.multi_select {
                                self.state.toggle_row_selection();
                            }
                        }
                        KeyCode::Char('a') => {
                            if self.state.multi_select {
                                // Select all visible rows
                                self.state.selected_rows = self.state.filtered_rows.clone();
                            }
                        }
                        KeyCode::Char('d') => {
                            if self.state.multi_select {
                                self.state.clear_selections();
                            }
                        }
                        KeyCode::Char('f') => {
                            self.state.toggle_filter();
                            self.filter_cursor = self.state.filter_text.len();
                        }
                        KeyCode::Char('s') => {
                            // Toggle sort on first column
                            if !self.state.columns.is_empty() {
                                self.state.sort_by_column(0);
                            }
                        }
                        KeyCode::Char(ch) if ch.is_ascii_digit() => {
                            // Sort by column number
                            if let Some(column_index) = ch.to_digit(10) {
                                let column_index = column_index as usize;
                                if column_index > 0 && column_index <= self.state.columns.len() {
                                    self.state.sort_by_column(column_index - 1);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(ComponentAction::None)
            }
            _ => Ok(ComponentAction::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_column_creation() {
        let column = TableColumn::new("Name", Constraint::Length(20))
            .sortable()
            .alignment(Alignment::Center);

        assert_eq!(column.title, "Name");
        assert!(column.sortable);
        assert_eq!(column.alignment, Alignment::Center);
    }

    #[test]
    fn test_table_row_creation() {
        let row = TableRow::new(vec!["John".to_string(), "Doe".to_string()])
            .with_metadata("user_id:123".to_string());

        assert_eq!(row.cells.len(), 2);
        assert_eq!(row.cells[0], "John");
        assert_eq!(row.metadata, Some("user_id:123".to_string()));
    }

    #[test]
    fn test_table_state_navigation() {
        let mut state = TableState::new();
        state.set_rows(vec![
            TableRow::new(vec!["Row 1".to_string()]),
            TableRow::new(vec!["Row 2".to_string()]),
            TableRow::new(vec!["Row 3".to_string()]),
        ]);

        // Initial state
        assert_eq!(state.selected, None);

        // Select first
        state.select_first();
        assert_eq!(state.selected, Some(0));

        // Select next
        state.select_next();
        assert_eq!(state.selected, Some(1));

        // Select last
        state.select_last();
        assert_eq!(state.selected, Some(2));

        // Select previous
        state.select_previous();
        assert_eq!(state.selected, Some(1));
    }

    #[test]
    fn test_table_state_sorting() {
        let mut state = TableState::new();
        state.set_columns(vec![
            TableColumn::new("Name", Constraint::Length(20)).sortable(),
            TableColumn::new("Age", Constraint::Length(10)).sortable(),
        ]);
        state.set_rows(vec![
            TableRow::new(vec!["John".to_string(), "30".to_string()]),
            TableRow::new(vec!["Alice".to_string(), "25".to_string()]),
            TableRow::new(vec!["Bob".to_string(), "35".to_string()]),
        ]);

        // Sort by first column (Name)
        state.sort_by_column(0);
        assert_eq!(state.sort_column, Some(0));
        assert_eq!(state.sort_direction, SortDirection::Ascending);

        // Verify sorting
        let visible_rows = state.visible_rows();
        assert_eq!(visible_rows[0].cells[0], "Alice");
        assert_eq!(visible_rows[1].cells[0], "Bob");
        assert_eq!(visible_rows[2].cells[0], "John");

        // Toggle sort direction
        state.sort_by_column(0);
        assert_eq!(state.sort_direction, SortDirection::Descending);
        
        let visible_rows = state.visible_rows();
        assert_eq!(visible_rows[0].cells[0], "John");
        assert_eq!(visible_rows[1].cells[0], "Bob");
        assert_eq!(visible_rows[2].cells[0], "Alice");
    }

    #[test]
    fn test_table_state_filtering() {
        let mut state = TableState::new();
        state.set_rows(vec![
            TableRow::new(vec!["John Doe".to_string()]),
            TableRow::new(vec!["Jane Smith".to_string()]),
            TableRow::new(vec!["Bob Johnson".to_string()]),
        ]);

        // No filter - all rows visible
        assert_eq!(state.total_rows(), 3);

        // Filter by "John"
        state.set_filter("John");
        assert_eq!(state.total_rows(), 2); // John Doe and Bob Johnson

        // Filter by "Jane"
        state.set_filter("Jane");
        assert_eq!(state.total_rows(), 1); // Jane Smith

        // Clear filter
        state.set_filter("");
        assert_eq!(state.total_rows(), 3);
    }

    #[test]
    fn test_table_state_multi_select() {
        let mut state = TableState::new();
        state.multi_select = true;
        state.set_rows(vec![
            TableRow::new(vec!["Row 1".to_string()]),
            TableRow::new(vec!["Row 2".to_string()]),
            TableRow::new(vec!["Row 3".to_string()]),
        ]);

        assert_eq!(state.selected_rows.len(), 0);

        // Select first row
        state.select_first();
        state.toggle_row_selection();
        assert_eq!(state.selected_rows.len(), 1);
        assert!(state.is_row_selected(0));

        // Select second row
        state.select_next();
        state.toggle_row_selection();
        assert_eq!(state.selected_rows.len(), 2);
        assert!(state.is_row_selected(1));

        // Deselect first row
        state.select_first();
        state.toggle_row_selection();
        assert_eq!(state.selected_rows.len(), 1);
        assert!(!state.is_row_selected(0));
        assert!(state.is_row_selected(1));

        // Clear all selections
        state.clear_selections();
        assert_eq!(state.selected_rows.len(), 0);
    }

    #[test]
    fn test_table_component_creation() {
        let mut table = TableComponent::new("Test Table");
        table.set_columns(vec![
            TableColumn::new("Name", Constraint::Length(20)),
            TableColumn::new("Value", Constraint::Length(10)),
        ]);
        table.set_rows(vec![
            TableRow::new(vec!["Test".to_string(), "123".to_string()]),
        ]);

        assert_eq!(table.title, "Test Table");
        assert_eq!(table.state.columns.len(), 2);
        assert_eq!(table.state.total_rows(), 1);
    }

    #[test]
    fn test_sort_direction_toggle() {
        let mut direction = SortDirection::Ascending;
        direction = direction.toggle();
        assert_eq!(direction, SortDirection::Descending);

        direction = direction.toggle();
        assert_eq!(direction, SortDirection::Ascending);
    }
}