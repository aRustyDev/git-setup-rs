use crossterm::{
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen, SetTitle,
    },
    cursor::{Hide, Show},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal as RatatuiTerminal,
};
use std::io::{self, Stdout};
use crate::error::{Result, GitSetupError};

pub type Terminal = RatatuiTerminal<CrosstermBackend<Stdout>>;

/// Manages terminal state and restoration
pub struct TerminalManager {
    terminal: Terminal,
    original_hook: Option<Box<dyn Fn(&std::panic::PanicInfo<'_>) + 'static + Sync + Send>>,
}

impl TerminalManager {
    pub fn new() -> Result<Self> {
        // Set up panic hook to restore terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // Restore terminal before printing panic
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);

            // Call original panic hook
            original_hook(panic_info);
        }));

        // Enter raw mode and alternate screen
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide, SetTitle("Git Setup"))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = RatatuiTerminal::new(backend)?;

        Ok(Self {
            terminal,
            original_hook: None, // We can't actually store the original hook due to type limitations
        })
    }

    pub fn terminal(&mut self) -> &mut Terminal {
        &mut self.terminal
    }

    pub fn restore(&mut self) -> Result<()> {
        // Restore terminal state
        disable_raw_mode()?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            Show
        )?;

        Ok(())
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        // Ensure terminal is restored even if not explicitly called
        let _ = self.restore();
    }
}

/// Terminal utilities
pub struct TerminalUtils;

impl TerminalUtils {
    /// Check if terminal supports colors
    pub fn supports_color() -> bool {
        // Check common environment variables
        if let Ok(term) = std::env::var("TERM") {
            !term.is_empty() && term != "dumb"
        } else {
            // Default to true on most systems
            cfg!(not(target_os = "windows")) || std::env::var("ANSICON").is_ok()
        }
    }

    /// Check if terminal supports unicode
    pub fn supports_unicode() -> bool {
        // Check locale for UTF-8 support
        if let Ok(lang) = std::env::var("LANG") {
            lang.to_lowercase().contains("utf-8") || lang.to_lowercase().contains("utf8")
        } else if let Ok(lc_all) = std::env::var("LC_ALL") {
            lc_all.to_lowercase().contains("utf-8") || lc_all.to_lowercase().contains("utf8")
        } else {
            // Default to true on most modern systems
            true
        }
    }

    /// Get terminal size
    pub fn size() -> Result<(u16, u16)> {
        Ok(crossterm::terminal::size()?)
    }

    /// Check if we're in a terminal (not piped)
    pub fn is_tty() -> bool {
        atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stdin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_utils_color_support() {
        // Save original TERM
        let original_term = std::env::var("TERM").ok();

        // Test with color terminal
        unsafe { std::env::set_var("TERM", "xterm-256color"); }
        assert!(TerminalUtils::supports_color());

        // Test with dumb terminal
        unsafe { std::env::set_var("TERM", "dumb"); }
        assert!(!TerminalUtils::supports_color());

        // Test with no TERM
        unsafe { std::env::remove_var("TERM"); }
        let supports = TerminalUtils::supports_color();
        // Should depend on the OS
        #[cfg(not(target_os = "windows"))]
        assert!(supports);

        // Restore original TERM
        if let Some(term) = original_term {
            unsafe { std::env::set_var("TERM", term); }
        }
    }

    #[test]
    fn test_terminal_utils_unicode_support() {
        // Save original locale
        let original_lang = std::env::var("LANG").ok();

        // Test with UTF-8 locale
        unsafe { std::env::set_var("LANG", "en_US.UTF-8"); }
        assert!(TerminalUtils::supports_unicode());

        // Test with non-UTF-8 locale
        unsafe { std::env::set_var("LANG", "C"); }
        assert!(!TerminalUtils::supports_unicode());

        // Restore original locale
        if let Some(lang) = original_lang {
            unsafe { std::env::set_var("LANG", lang); }
        } else {
            unsafe { std::env::remove_var("LANG"); }
        }
    }

    #[test]
    fn test_terminal_size() {
        // This test might fail in CI environments without a terminal
        let result = TerminalUtils::size();
        if result.is_ok() {
            let (width, height) = result.unwrap();
            assert!(width > 0);
            assert!(height > 0);
        }
    }
}
