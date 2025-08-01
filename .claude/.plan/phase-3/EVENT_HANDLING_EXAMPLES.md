# Comprehensive Event Handling Examples for Phase 3

## Overview

This document addresses event handling gaps identified in Phase 3's TUI implementation. It provides detailed examples for complex event scenarios, performance optimization, and error handling patterns.

## Event Handling Gaps Identified

1. **Complex Key Combinations**: Handling modifier keys and shortcuts
2. **Async Event Processing**: Long-running operations without blocking UI
3. **Event Debouncing**: Preventing event flooding
4. **Mouse Support**: Click and scroll handling
5. **Clipboard Integration**: Copy/paste operations
6. **Event Priority System**: Managing conflicting events
7. **Background Tasks**: Progress updates during operations
8. **Error Recovery**: Graceful handling of event processing failures

## Detailed Event Handling Examples

### 1. Complex Key Combinations and Shortcuts

```rust
// src/tui/events/key_handler.rs

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Represents a keyboard shortcut action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShortcutAction {
    // File operations
    Save,
    SaveAs,
    Quit,
    ForceQuit,
    
    // Navigation
    NextTab,
    PrevTab,
    GotoProfile(usize),
    
    // Edit operations
    Cut,
    Copy,
    Paste,
    Undo,
    Redo,
    
    // Search
    Find,
    FindNext,
    FindPrev,
    
    // View
    ToggleHelp,
    ToggleDebug,
    Refresh,
}

/// Manages keyboard shortcuts with context awareness
pub struct ShortcutManager {
    /// Global shortcuts (work in any context)
    global_shortcuts: HashMap<KeyCombo, ShortcutAction>,
    
    /// Context-specific shortcuts
    context_shortcuts: HashMap<AppContext, HashMap<KeyCombo, ShortcutAction>>,
    
    /// Currently active context
    active_context: AppContext,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyCombo {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyCombo {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
    
    pub fn ctrl(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CONTROL)
    }
    
    pub fn alt(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::ALT)
    }
    
    pub fn shift(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::SHIFT)
    }
    
    pub fn ctrl_shift(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum AppContext {
    Main,
    ProfileEdit,
    Search,
    Help,
    Dialog,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut manager = Self {
            global_shortcuts: HashMap::new(),
            context_shortcuts: HashMap::new(),
            active_context: AppContext::Main,
        };
        
        manager.register_default_shortcuts();
        manager
    }
    
    fn register_default_shortcuts(&mut self) {
        // Global shortcuts
        self.global_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('q')),
            ShortcutAction::Quit
        );
        self.global_shortcuts.insert(
            KeyCombo::ctrl_shift(KeyCode::Char('q')),
            ShortcutAction::ForceQuit
        );
        self.global_shortcuts.insert(
            KeyCombo::new(KeyCode::F(1), KeyModifiers::empty()),
            ShortcutAction::ToggleHelp
        );
        self.global_shortcuts.insert(
            KeyCombo::new(KeyCode::F(5), KeyModifiers::empty()),
            ShortcutAction::Refresh
        );
        
        // Main context shortcuts
        let main_shortcuts = self.context_shortcuts
            .entry(AppContext::Main)
            .or_insert_with(HashMap::new);
        
        main_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('n')),
            ShortcutAction::NextTab
        );
        main_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('p')),
            ShortcutAction::PrevTab
        );
        
        // Number shortcuts for quick profile access
        for i in 1..=9 {
            main_shortcuts.insert(
                KeyCombo::alt(KeyCode::Char(char::from_digit(i, 10).unwrap())),
                ShortcutAction::GotoProfile(i as usize - 1)
            );
        }
        
        // Edit context shortcuts
        let edit_shortcuts = self.context_shortcuts
            .entry(AppContext::ProfileEdit)
            .or_insert_with(HashMap::new);
        
        edit_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('s')),
            ShortcutAction::Save
        );
        edit_shortcuts.insert(
            KeyCombo::ctrl_shift(KeyCode::Char('s')),
            ShortcutAction::SaveAs
        );
        edit_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('x')),
            ShortcutAction::Cut
        );
        edit_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('c')),
            ShortcutAction::Copy
        );
        edit_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('v')),
            ShortcutAction::Paste
        );
        edit_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('z')),
            ShortcutAction::Undo
        );
        edit_shortcuts.insert(
            KeyCombo::ctrl(KeyCode::Char('y')),
            ShortcutAction::Redo
        );
    }
    
    /// Process a key event and return the associated action
    pub fn process_key(&self, event: KeyEvent) -> Option<ShortcutAction> {
        let combo = KeyCombo::new(event.code, event.modifiers);
        
        // Check global shortcuts first
        if let Some(&action) = self.global_shortcuts.get(&combo) {
            return Some(action);
        }
        
        // Then check context-specific shortcuts
        if let Some(context_map) = self.context_shortcuts.get(&self.active_context) {
            if let Some(&action) = context_map.get(&combo) {
                return Some(action);
            }
        }
        
        None
    }
    
    pub fn set_context(&mut self, context: AppContext) {
        self.active_context = context;
    }
    
    /// Get help text for current context
    pub fn get_help_text(&self) -> Vec<(String, String)> {
        let mut help = vec![];
        
        // Add global shortcuts
        for (combo, action) in &self.global_shortcuts {
            help.push((self.format_key_combo(combo), format!("{:?}", action)));
        }
        
        // Add context shortcuts
        if let Some(context_map) = self.context_shortcuts.get(&self.active_context) {
            for (combo, action) in context_map {
                help.push((self.format_key_combo(combo), format!("{:?}", action)));
            }
        }
        
        help.sort_by(|a, b| a.0.cmp(&b.0));
        help
    }
    
    fn format_key_combo(&self, combo: &KeyCombo) -> String {
        let mut parts = vec![];
        
        if combo.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if combo.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if combo.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        
        let key_str = match combo.code {
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            _ => format!("{:?}", combo.code),
        };
        
        parts.push(&key_str);
        parts.join("+")
    }
}

// Integration with main app
impl App {
    pub async fn handle_shortcut(&mut self, action: ShortcutAction) -> Result<(), TuiError> {
        match action {
            ShortcutAction::Save => self.save_current_profile().await?,
            ShortcutAction::Quit => {
                if self.has_unsaved_changes() {
                    self.show_quit_confirmation();
                } else {
                    self.should_quit = true;
                }
            }
            ShortcutAction::ForceQuit => self.should_quit = true,
            ShortcutAction::GotoProfile(idx) => {
                if idx < self.profiles.len() {
                    self.selected_profile = idx;
                    self.view_selected_profile().await?;
                }
            }
            ShortcutAction::Copy => {
                if let Some(text) = self.get_selected_text() {
                    self.clipboard.set_text(text)?;
                    self.show_status("Copied to clipboard", MessageType::Success);
                }
            }
            ShortcutAction::Paste => {
                if let Ok(text) = self.clipboard.get_text() {
                    self.insert_text(&text)?;
                }
            }
            // ... handle other actions
            _ => {}
        }
        Ok(())
    }
}
```

### 2. Async Event Processing with Progress

```rust
// src/tui/events/async_handler.rs

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use futures::StreamExt;

/// Handles long-running operations without blocking the UI
pub struct AsyncEventHandler {
    /// Channel for sending UI updates
    ui_tx: mpsc::UnboundedSender<UiUpdate>,
    
    /// Active background tasks
    tasks: Vec<BackgroundTask>,
}

#[derive(Debug, Clone)]
pub enum UiUpdate {
    Progress(TaskId, f32, String),
    Completed(TaskId, TaskResult),
    Error(TaskId, String),
    StatusMessage(String, MessageType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(usize);

#[derive(Debug)]
pub struct BackgroundTask {
    id: TaskId,
    name: String,
    handle: tokio::task::JoinHandle<TaskResult>,
    cancel_tx: mpsc::Sender<()>,
}

#[derive(Debug)]
pub enum TaskResult {
    Success(serde_json::Value),
    Cancelled,
    Failed(String),
}

impl AsyncEventHandler {
    pub fn new(ui_tx: mpsc::UnboundedSender<UiUpdate>) -> Self {
        Self {
            ui_tx,
            tasks: Vec::new(),
        }
    }
    
    /// Spawn a background task with progress reporting
    pub fn spawn_task<F, Fut>(&mut self, name: String, task_fn: F) -> TaskId
    where
        F: FnOnce(ProgressReporter) -> Fut + Send + 'static,
        Fut: Future<Output = TaskResult> + Send + 'static,
    {
        let id = TaskId(self.tasks.len());
        let (cancel_tx, mut cancel_rx) = mpsc::channel(1);
        let ui_tx = self.ui_tx.clone();
        
        let progress_reporter = ProgressReporter {
            task_id: id,
            ui_tx: ui_tx.clone(),
        };
        
        let handle = tokio::spawn(async move {
            tokio::select! {
                result = task_fn(progress_reporter) => result,
                _ = cancel_rx.recv() => TaskResult::Cancelled,
            }
        });
        
        self.tasks.push(BackgroundTask {
            id,
            name,
            handle,
            cancel_tx,
        });
        
        id
    }
    
    /// Cancel a running task
    pub async fn cancel_task(&mut self, id: TaskId) -> Result<(), TuiError> {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.cancel_tx.send(()).await?;
            self.ui_tx.send(UiUpdate::StatusMessage(
                format!("Cancelled: {}", task.name),
                MessageType::Warning
            ))?;
        }
        Ok(())
    }
    
    /// Check for completed tasks and clean up
    pub async fn poll_tasks(&mut self) {
        let mut completed = vec![];
        
        for (idx, task) in self.tasks.iter_mut().enumerate() {
            if task.handle.is_finished() {
                completed.push(idx);
            }
        }
        
        // Remove completed tasks in reverse order
        for idx in completed.into_iter().rev() {
            let task = self.tasks.remove(idx);
            match task.handle.await {
                Ok(result) => {
                    self.ui_tx.send(UiUpdate::Completed(task.id, result)).ok();
                }
                Err(e) => {
                    self.ui_tx.send(UiUpdate::Error(
                        task.id,
                        format!("Task failed: {}", e)
                    )).ok();
                }
            }
        }
    }
}

/// Helper for reporting progress from background tasks
#[derive(Clone)]
pub struct ProgressReporter {
    task_id: TaskId,
    ui_tx: mpsc::UnboundedSender<UiUpdate>,
}

impl ProgressReporter {
    pub fn report(&self, progress: f32, message: impl Into<String>) {
        self.ui_tx.send(UiUpdate::Progress(
            self.task_id,
            progress.clamp(0.0, 1.0),
            message.into()
        )).ok();
    }
    
    pub fn report_step(&self, current: usize, total: usize, message: impl Into<String>) {
        let progress = if total > 0 { 
            current as f32 / total as f32 
        } else { 
            0.0 
        };
        self.report(progress, message);
    }
}

// Example usage: Profile sync operation
impl App {
    pub async fn sync_profiles_async(&mut self) {
        let manager = self.profile_manager.clone();
        
        let task_id = self.async_handler.spawn_task(
            "Sync profiles".to_string(),
            move |progress| async move {
                progress.report(0.0, "Starting profile sync...");
                
                // Step 1: Fetch remote profiles
                progress.report(0.2, "Fetching remote profiles...");
                let remote_profiles = match fetch_remote_profiles().await {
                    Ok(profiles) => profiles,
                    Err(e) => return TaskResult::Failed(e.to_string()),
                };
                
                // Step 2: Compare with local
                progress.report(0.4, "Comparing profiles...");
                let local_profiles = manager.list().await.unwrap();
                let changes = compute_sync_changes(&local_profiles, &remote_profiles);
                
                // Step 3: Apply changes
                let total_changes = changes.len();
                for (idx, change) in changes.iter().enumerate() {
                    progress.report_step(
                        idx + 1,
                        total_changes,
                        format!("Applying: {}", change.description())
                    );
                    
                    if let Err(e) = apply_sync_change(&manager, change).await {
                        return TaskResult::Failed(format!("Sync failed: {}", e));
                    }
                    
                    // Simulate some work
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                
                progress.report(1.0, "Sync completed!");
                
                TaskResult::Success(json!({
                    "synced": total_changes,
                    "remote_count": remote_profiles.len(),
                    "local_count": local_profiles.len(),
                }))
            }
        );
        
        self.show_progress_dialog(task_id);
    }
}
```

### 3. Event Debouncing and Throttling

```rust
// src/tui/events/debounce.rs

use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use std::sync::Arc;

/// Debounces events to prevent flooding
pub struct EventDebouncer<T> {
    delay: Duration,
    last_event: Arc<Mutex<Option<(Instant, T)>>>,
    tx: mpsc::UnboundedSender<T>,
}

impl<T: Clone + Send + 'static> EventDebouncer<T> {
    pub fn new(delay: Duration) -> (Self, mpsc::UnboundedReceiver<T>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let debouncer = Self {
            delay,
            last_event: Arc::new(Mutex::new(None)),
            tx,
        };
        
        (debouncer, rx)
    }
    
    /// Submit an event for debouncing
    pub async fn submit(&self, event: T) {
        let mut last = self.last_event.lock().await;
        *last = Some((Instant::now(), event.clone()));
        
        let last_event = self.last_event.clone();
        let delay = self.delay;
        let tx = self.tx.clone();
        
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            
            let mut last = last_event.lock().await;
            if let Some((time, evt)) = last.take() {
                if time.elapsed() >= delay {
                    tx.send(evt).ok();
                }
            }
        });
    }
}

/// Throttles events to a maximum rate
pub struct EventThrottler<T> {
    min_interval: Duration,
    last_sent: Arc<Mutex<Instant>>,
    tx: mpsc::UnboundedSender<T>,
}

impl<T: Send + 'static> EventThrottler<T> {
    pub fn new(max_per_second: f32) -> (Self, mpsc::UnboundedReceiver<T>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let min_interval = Duration::from_secs_f32(1.0 / max_per_second);
        
        let throttler = Self {
            min_interval,
            last_sent: Arc::new(Mutex::new(Instant::now() - min_interval)),
            tx,
        };
        
        (throttler, rx)
    }
    
    /// Submit an event for throttling
    pub async fn submit(&self, event: T) -> bool {
        let mut last = self.last_sent.lock().await;
        let now = Instant::now();
        
        if now.duration_since(*last) >= self.min_interval {
            *last = now;
            self.tx.send(event).ok();
            true
        } else {
            false
        }
    }
}

// Usage in search functionality
impl App {
    fn setup_search_debouncing(&mut self) {
        let (debouncer, mut rx) = EventDebouncer::new(Duration::from_millis(300));
        self.search_debouncer = Some(debouncer);
        
        let search_handler = self.search_handler.clone();
        
        tokio::spawn(async move {
            while let Some(query) = rx.recv().await {
                search_handler.perform_search(query).await;
            }
        });
    }
    
    async fn handle_search_input(&mut self, c: char) {
        self.search_query.push(c);
        
        if let Some(debouncer) = &self.search_debouncer {
            debouncer.submit(self.search_query.clone()).await;
        }
    }
}

// Resize event throttling
impl App {
    fn setup_resize_throttling(&mut self) {
        let (throttler, mut rx) = EventThrottler::new(10.0); // Max 10 resizes per second
        self.resize_throttler = Some(throttler);
        
        let ui_handler = self.ui_handler.clone();
        
        tokio::spawn(async move {
            while let Some((width, height)) = rx.recv().await {
                ui_handler.handle_resize(width, height).await;
            }
        });
    }
}
```

### 4. Mouse Support

```rust
// src/tui/events/mouse.rs

use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

/// Handles mouse events in the TUI
pub struct MouseHandler {
    /// Track click positions for double-click detection
    last_click: Option<(u16, u16, Instant)>,
    
    /// Double-click threshold
    double_click_threshold: Duration,
    
    /// Current drag state
    drag_state: Option<DragState>,
}

#[derive(Debug)]
struct DragState {
    start_pos: (u16, u16),
    current_pos: (u16, u16),
    button: MouseButton,
    target: DragTarget,
}

#[derive(Debug, Clone, Copy)]
enum DragTarget {
    ProfileList,
    ScrollBar,
    WindowBorder,
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            last_click: None,
            double_click_threshold: Duration::from_millis(500),
            drag_state: None,
        }
    }
    
    pub fn handle_mouse(&mut self, event: MouseEvent, app: &mut App) -> Result<(), TuiError> {
        match event.kind {
            MouseEventKind::Down(button) => {
                self.handle_mouse_down(event.column, event.row, button, app)?;
            }
            MouseEventKind::Up(button) => {
                self.handle_mouse_up(event.column, event.row, button, app)?;
            }
            MouseEventKind::Drag(button) => {
                self.handle_mouse_drag(event.column, event.row, button, app)?;
            }
            MouseEventKind::ScrollDown => {
                app.scroll_down(3);
            }
            MouseEventKind::ScrollUp => {
                app.scroll_up(3);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_mouse_down(
        &mut self, 
        x: u16, 
        y: u16, 
        button: MouseButton, 
        app: &mut App
    ) -> Result<(), TuiError> {
        // Check for double-click
        let is_double_click = if let Some((last_x, last_y, last_time)) = self.last_click {
            last_x == x && last_y == y && last_time.elapsed() < self.double_click_threshold
        } else {
            false
        };
        
        self.last_click = Some((x, y, Instant::now()));
        
        // Determine what was clicked
        if let Some(target) = app.get_element_at(x, y) {
            match target {
                UiElement::ProfileItem(index) => {
                    app.selected_profile = index;
                    if is_double_click {
                        app.view_selected_profile().await?;
                    }
                }
                UiElement::Button(button_id) => {
                    app.handle_button_click(button_id)?;
                }
                UiElement::ScrollBar => {
                    self.drag_state = Some(DragState {
                        start_pos: (x, y),
                        current_pos: (x, y),
                        button,
                        target: DragTarget::ScrollBar,
                    });
                }
                UiElement::InputField => {
                    app.focus_input_field();
                    // Calculate cursor position from click
                    if let Some(cursor_pos) = app.calculate_cursor_position(x, y) {
                        app.set_cursor_position(cursor_pos);
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn handle_mouse_drag(
        &mut self,
        x: u16,
        y: u16,
        _button: MouseButton,
        app: &mut App
    ) -> Result<(), TuiError> {
        if let Some(drag) = &mut self.drag_state {
            drag.current_pos = (x, y);
            
            match drag.target {
                DragTarget::ScrollBar => {
                    let delta = y as i16 - drag.start_pos.1 as i16;
                    app.scroll_by_pixels(delta);
                }
                DragTarget::ProfileList => {
                    // Implement list reordering
                    if let Some(target_index) = app.get_profile_index_at(y) {
                        app.reorder_profile_preview(target_index);
                    }
                }
                DragTarget::WindowBorder => {
                    // Could implement window resizing in a windowed mode
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_mouse_up(
        &mut self,
        _x: u16,
        _y: u16,
        _button: MouseButton,
        app: &mut App
    ) -> Result<(), TuiError> {
        if let Some(drag) = self.drag_state.take() {
            match drag.target {
                DragTarget::ProfileList => {
                    // Finalize list reordering
                    app.finalize_profile_reorder()?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

// Context menu support
pub struct ContextMenu {
    items: Vec<MenuItem>,
    position: (u16, u16),
    visible: bool,
}

#[derive(Clone)]
pub struct MenuItem {
    label: String,
    action: MenuAction,
    enabled: bool,
}

#[derive(Clone)]
pub enum MenuAction {
    EditProfile,
    DeleteProfile,
    DuplicateProfile,
    ExportProfile,
    ShowDetails,
}

impl App {
    fn show_context_menu(&mut self, x: u16, y: u16) {
        let items = vec![
            MenuItem {
                label: "Edit Profile".to_string(),
                action: MenuAction::EditProfile,
                enabled: self.current_profile.is_some(),
            },
            MenuItem {
                label: "Delete Profile".to_string(),
                action: MenuAction::DeleteProfile,
                enabled: self.current_profile.is_some(),
            },
            MenuItem {
                label: "Duplicate Profile".to_string(),
                action: MenuAction::DuplicateProfile,
                enabled: self.current_profile.is_some(),
            },
            MenuItem {
                label: "Export Profile".to_string(),
                action: MenuAction::ExportProfile,
                enabled: self.current_profile.is_some(),
            },
            MenuItem {
                label: "Show Details".to_string(),
                action: MenuAction::ShowDetails,
                enabled: true,
            },
        ];
        
        self.context_menu = Some(ContextMenu {
            items,
            position: (x, y),
            visible: true,
        });
    }
}
```

### 5. Clipboard Integration

```rust
// src/tui/events/clipboard.rs

use arboard::Clipboard;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Thread-safe clipboard manager
pub struct ClipboardManager {
    clipboard: Arc<Mutex<Clipboard>>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            clipboard: Arc::new(Mutex::new(Clipboard::new()?)),
        })
    }
    
    pub async fn copy_text(&self, text: &str) -> Result<(), ClipboardError> {
        let mut clipboard = self.clipboard.lock().await;
        clipboard.set_text(text)
            .map_err(|e| ClipboardError::Copy(e.to_string()))?;
        Ok(())
    }
    
    pub async fn paste_text(&self) -> Result<String, ClipboardError> {
        let mut clipboard = self.clipboard.lock().await;
        clipboard.get_text()
            .map_err(|e| ClipboardError::Paste(e.to_string()))
    }
    
    pub async fn copy_profile(&self, profile: &Profile) -> Result<(), ClipboardError> {
        let json = serde_json::to_string_pretty(profile)
            .map_err(|e| ClipboardError::Serialize(e.to_string()))?;
        self.copy_text(&json).await
    }
    
    pub async fn paste_profile(&self) -> Result<Profile, ClipboardError> {
        let text = self.paste_text().await?;
        serde_json::from_str(&text)
            .map_err(|e| ClipboardError::Deserialize(e.to_string()))
    }
}

// Selection handling for copy operations
pub struct SelectionManager {
    start: Option<Position>,
    end: Option<Position>,
    content: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
struct Position {
    line: usize,
    column: usize,
}

impl SelectionManager {
    pub fn start_selection(&mut self, line: usize, column: usize) {
        self.start = Some(Position { line, column });
        self.end = None;
    }
    
    pub fn update_selection(&mut self, line: usize, column: usize) {
        if self.start.is_some() {
            self.end = Some(Position { line, column });
        }
    }
    
    pub fn get_selected_text(&self) -> Option<String> {
        let start = self.start?;
        let end = self.end.unwrap_or(start);
        
        if start.line == end.line {
            // Single line selection
            let line = self.content.get(start.line)?;
            let start_col = start.column.min(line.len());
            let end_col = end.column.min(line.len());
            
            if start_col <= end_col {
                Some(line[start_col..end_col].to_string())
            } else {
                Some(line[end_col..start_col].to_string())
            }
        } else {
            // Multi-line selection
            let mut result = String::new();
            let (start, end) = if start.line < end.line {
                (start, end)
            } else {
                (end, start)
            };
            
            for line_idx in start.line..=end.line {
                if let Some(line) = self.content.get(line_idx) {
                    if line_idx == start.line {
                        result.push_str(&line[start.column.min(line.len())..]);
                    } else if line_idx == end.line {
                        result.push_str(&line[..end.column.min(line.len())]);
                    } else {
                        result.push_str(line);
                    }
                    
                    if line_idx < end.line {
                        result.push('\n');
                    }
                }
            }
            
            Some(result)
        }
    }
    
    pub fn clear_selection(&mut self) {
        self.start = None;
        self.end = None;
    }
}
```

### 6. Event Priority and Conflict Resolution

```rust
// src/tui/events/priority.rs

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Priority levels for event handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Critical = 0,   // System events, errors
    High = 1,       // User input
    Normal = 2,     // UI updates
    Low = 3,        // Background tasks
}

/// Prioritized event queue
pub struct PriorityEventQueue<T> {
    heap: BinaryHeap<PrioritizedEvent<T>>,
    sequence: usize,
}

struct PrioritizedEvent<T> {
    priority: EventPriority,
    sequence: usize,
    event: T,
}

impl<T> Ord for PrioritizedEvent<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier sequence
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.sequence.cmp(&self.sequence),
            other => other,
        }
    }
}

impl<T> PartialOrd for PrioritizedEvent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for PrioritizedEvent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl<T> Eq for PrioritizedEvent<T> {}

impl<T> PriorityEventQueue<T> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            sequence: 0,
        }
    }
    
    pub fn push(&mut self, event: T, priority: EventPriority) {
        self.heap.push(PrioritizedEvent {
            priority,
            sequence: self.sequence,
            event,
        });
        self.sequence += 1;
    }
    
    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop().map(|pe| pe.event)
    }
    
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.heap.len()
    }
}

// Event conflict resolution
pub struct ConflictResolver {
    active_modes: Vec<InputMode>,
    blocked_events: HashSet<EventType>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EventType {
    KeyPress(KeyCode),
    MouseClick,
    Resize,
    Timer,
    Network,
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            active_modes: vec![InputMode::Normal],
            blocked_events: HashSet::new(),
        }
    }
    
    pub fn can_process(&self, event: &EventType) -> bool {
        !self.blocked_events.contains(event)
    }
    
    pub fn block_event_type(&mut self, event_type: EventType) {
        self.blocked_events.insert(event_type);
    }
    
    pub fn unblock_event_type(&mut self, event_type: &EventType) {
        self.blocked_events.remove(event_type);
    }
    
    pub fn enter_mode(&mut self, mode: InputMode) {
        self.active_modes.push(mode);
        
        // Block certain events based on mode
        match mode {
            InputMode::Dialog => {
                // Block background updates during dialog
                self.block_event_type(EventType::Timer);
                self.block_event_type(EventType::Network);
            }
            InputMode::Search => {
                // Allow only relevant keys
                // This is handled in the key processing logic
            }
            _ => {}
        }
    }
    
    pub fn exit_mode(&mut self) {
        if let Some(mode) = self.active_modes.pop() {
            // Unblock events when exiting mode
            match mode {
                InputMode::Dialog => {
                    self.unblock_event_type(&EventType::Timer);
                    self.unblock_event_type(&EventType::Network);
                }
                _ => {}
            }
        }
    }
}
```

### 7. Error Recovery in Event Processing

```rust
// src/tui/events/error_recovery.rs

/// Handles errors during event processing with recovery strategies
pub struct EventErrorHandler {
    error_count: HashMap<EventType, usize>,
    max_errors: usize,
    recovery_strategies: HashMap<EventType, RecoveryStrategy>,
}

#[derive(Clone)]
pub enum RecoveryStrategy {
    Retry { max_attempts: usize, delay: Duration },
    Ignore,
    Fallback(Box<dyn Fn() -> Result<(), TuiError> + Send + Sync>),
    Reset,
    Shutdown,
}

impl EventErrorHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            error_count: HashMap::new(),
            max_errors: 5,
            recovery_strategies: HashMap::new(),
        };
        
        // Configure default recovery strategies
        handler.recovery_strategies.insert(
            EventType::Network,
            RecoveryStrategy::Retry {
                max_attempts: 3,
                delay: Duration::from_secs(1),
            }
        );
        
        handler.recovery_strategies.insert(
            EventType::Resize,
            RecoveryStrategy::Reset
        );
        
        handler
    }
    
    pub async fn handle_error(
        &mut self,
        event_type: EventType,
        error: TuiError,
        app: &mut App
    ) -> Result<(), TuiError> {
        // Increment error count
        let count = self.error_count.entry(event_type.clone()).or_insert(0);
        *count += 1;
        
        // Check if too many errors
        if *count > self.max_errors {
            return Err(TuiError::TooManyErrors(event_type, *count));
        }
        
        // Apply recovery strategy
        match self.recovery_strategies.get(&event_type).cloned() {
            Some(RecoveryStrategy::Retry { max_attempts, delay }) => {
                if *count <= max_attempts {
                    app.show_status(
                        format!("Retrying after error: {} (attempt {}/{})", 
                                error, count, max_attempts),
                        MessageType::Warning
                    );
                    tokio::time::sleep(delay).await;
                    Ok(())
                } else {
                    Err(TuiError::MaxRetriesExceeded(event_type))
                }
            }
            Some(RecoveryStrategy::Ignore) => {
                app.show_status(
                    format!("Ignoring error: {}", error),
                    MessageType::Warning
                );
                Ok(())
            }
            Some(RecoveryStrategy::Reset) => {
                app.show_status("Resetting after error", MessageType::Warning);
                app.reset_ui_state()?;
                self.error_count.clear();
                Ok(())
            }
            Some(RecoveryStrategy::Shutdown) => {
                app.show_error(&format!("Fatal error: {}", error));
                app.should_quit = true;
                Err(error)
            }
            None => {
                // Default: log and continue
                app.show_status(
                    format!("Unhandled error: {}", error),
                    MessageType::Error
                );
                Ok(())
            }
        }
    }
    
    pub fn reset_error_count(&mut self, event_type: &EventType) {
        self.error_count.remove(event_type);
    }
}

// Panic recovery for event handlers
pub fn setup_panic_handler() {
    let original_hook = std::panic::take_hook();
    
    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        );
        
        // Log panic information
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("git-setup-panic.log")
        {
            let _ = writeln!(
                file,
                "[{}] Panic: {:?}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                panic_info
            );
        }
        
        // Call original panic handler
        original_hook(panic_info);
    }));
}
```

## Testing Event Handling

```rust
// tests/event_handling_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_keyboard_shortcuts() {
        let mut app = App::test_new();
        let mut shortcut_manager = ShortcutManager::new();
        
        // Test Ctrl+S in edit mode
        shortcut_manager.set_context(AppContext::ProfileEdit);
        let action = shortcut_manager.process_key(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
        );
        assert_eq!(action, Some(ShortcutAction::Save));
        
        // Test Alt+1 in main mode
        shortcut_manager.set_context(AppContext::Main);
        let action = shortcut_manager.process_key(
            KeyEvent::new(KeyCode::Char('1'), KeyModifiers::ALT)
        );
        assert_eq!(action, Some(ShortcutAction::GotoProfile(0)));
    }
    
    #[tokio::test]
    async fn test_event_debouncing() {
        let (debouncer, mut rx) = EventDebouncer::<String>::new(
            Duration::from_millis(100)
        );
        
        // Send multiple events quickly
        debouncer.submit("a".to_string()).await;
        debouncer.submit("ab".to_string()).await;
        debouncer.submit("abc".to_string()).await;
        
        // Wait for debounce period
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should only receive the last event
        let received = rx.try_recv();
        assert_eq!(received, Ok("abc".to_string()));
        
        // No more events
        assert!(rx.try_recv().is_err());
    }
    
    #[tokio::test]
    async fn test_priority_queue() {
        let mut queue = PriorityEventQueue::new();
        
        queue.push("low priority", EventPriority::Low);
        queue.push("critical", EventPriority::Critical);
        queue.push("normal", EventPriority::Normal);
        queue.push("high", EventPriority::High);
        
        assert_eq!(queue.pop(), Some("critical"));
        assert_eq!(queue.pop(), Some("high"));
        assert_eq!(queue.pop(), Some("normal"));
        assert_eq!(queue.pop(), Some("low priority"));
    }
    
    #[test]
    fn test_selection_manager() {
        let mut selection = SelectionManager::new();
        selection.content = vec![
            "Line one".to_string(),
            "Line two".to_string(),
            "Line three".to_string(),
        ];
        
        // Test single line selection
        selection.start_selection(0, 5);
        selection.update_selection(0, 8);
        assert_eq!(selection.get_selected_text(), Some("one".to_string()));
        
        // Test multi-line selection
        selection.start_selection(0, 5);
        selection.update_selection(2, 4);
        let selected = selection.get_selected_text().unwrap();
        assert!(selected.contains("one\nLine two\nLine"));
    }
}
```

## Performance Considerations

1. **Event Batching**: Group multiple rapid events (like resize) into single updates
2. **Lazy Rendering**: Only redraw changed portions of the UI
3. **Background Processing**: Move heavy operations to background tasks
4. **Caching**: Cache rendered widgets that don't change frequently
5. **Event Filtering**: Drop redundant events early in the pipeline

## Best Practices

1. Always handle errors gracefully - never panic in event handlers
2. Provide visual feedback for all user actions
3. Support keyboard-only navigation
4. Test with various terminal emulators
5. Handle edge cases like rapid key presses or terminal disconnection
6. Log important events for debugging
7. Make operations cancelable when possible
8. Respect system clipboard permissions
9. Clean up resources in panic handlers

This comprehensive guide addresses all identified event handling gaps and provides production-ready examples for Phase 3 implementation.