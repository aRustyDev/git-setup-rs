use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub styles: ThemeStyles,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub border: Color,
    pub highlight: Color,
    pub muted: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeStyles {
    pub base: Style,
    pub title: Style,
    pub border: Style,
    pub highlight: Style,
    pub selected: Style,
    pub success: Style,
    pub warning: Style,
    pub error: Style,
    pub info: Style,
    pub help: Style,
    pub key: Style,
}

impl Theme {
    /// Default dark theme
    pub fn dark() -> Self {
        let colors = ThemeColors {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Cyan,
            secondary: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            border: Color::Gray,
            highlight: Color::LightCyan,
            muted: Color::DarkGray,
        };

        let styles = ThemeStyles {
            base: Style::default()
                .fg(colors.foreground)
                .bg(colors.background),
            title: Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
            border: Style::default()
                .fg(colors.border),
            highlight: Style::default()
                .fg(colors.background)
                .bg(colors.highlight),
            selected: Style::default()
                .fg(colors.background)
                .bg(colors.primary)
                .add_modifier(Modifier::BOLD),
            success: Style::default()
                .fg(colors.success),
            warning: Style::default()
                .fg(colors.warning),
            error: Style::default()
                .fg(colors.error)
                .add_modifier(Modifier::BOLD),
            info: Style::default()
                .fg(colors.info),
            help: Style::default()
                .fg(colors.muted)
                .add_modifier(Modifier::ITALIC),
            key: Style::default()
                .fg(colors.secondary)
                .add_modifier(Modifier::BOLD),
        };

        Self {
            name: "Dark".to_string(),
            colors,
            styles,
        }
    }

    /// Light theme
    pub fn light() -> Self {
        let colors = ThemeColors {
            background: Color::White,
            foreground: Color::Black,
            primary: Color::Blue,
            secondary: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
            border: Color::Gray,
            highlight: Color::LightBlue,
            muted: Color::DarkGray,
        };

        let styles = ThemeStyles {
            base: Style::default()
                .fg(colors.foreground)
                .bg(colors.background),
            title: Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
            border: Style::default()
                .fg(colors.border),
            highlight: Style::default()
                .fg(colors.background)
                .bg(colors.highlight),
            selected: Style::default()
                .fg(colors.background)
                .bg(colors.primary)
                .add_modifier(Modifier::BOLD),
            success: Style::default()
                .fg(colors.success),
            warning: Style::default()
                .fg(colors.warning),
            error: Style::default()
                .fg(colors.error)
                .add_modifier(Modifier::BOLD),
            info: Style::default()
                .fg(colors.info),
            help: Style::default()
                .fg(colors.muted)
                .add_modifier(Modifier::ITALIC),
            key: Style::default()
                .fg(colors.secondary)
                .add_modifier(Modifier::BOLD),
        };

        Self {
            name: "Light".to_string(),
            colors,
            styles,
        }
    }

    /// High contrast theme for accessibility
    pub fn high_contrast() -> Self {
        let colors = ThemeColors {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Yellow,
            secondary: Color::Cyan,
            success: Color::LightGreen,
            warning: Color::Yellow,
            error: Color::LightRed,
            info: Color::LightBlue,
            border: Color::White,
            highlight: Color::Yellow,
            muted: Color::Gray,
        };

        let styles = ThemeStyles {
            base: Style::default()
                .fg(colors.foreground)
                .bg(colors.background),
            title: Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            border: Style::default()
                .fg(colors.border)
                .add_modifier(Modifier::BOLD),
            highlight: Style::default()
                .fg(colors.background)
                .bg(colors.highlight)
                .add_modifier(Modifier::BOLD),
            selected: Style::default()
                .fg(colors.background)
                .bg(colors.primary)
                .add_modifier(Modifier::BOLD),
            success: Style::default()
                .fg(colors.success)
                .add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(colors.warning)
                .add_modifier(Modifier::BOLD),
            error: Style::default()
                .fg(colors.error)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            info: Style::default()
                .fg(colors.info)
                .add_modifier(Modifier::BOLD),
            help: Style::default()
                .fg(colors.muted),
            key: Style::default()
                .fg(colors.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        };

        Self {
            name: "High Contrast".to_string(),
            colors,
            styles,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let dark = Theme::dark();
        assert_eq!(dark.name, "Dark");
        assert_eq!(dark.colors.background, Color::Black);
        assert_eq!(dark.colors.foreground, Color::White);

        let light = Theme::light();
        assert_eq!(light.name, "Light");
        assert_eq!(light.colors.background, Color::White);
        assert_eq!(light.colors.foreground, Color::Black);

        let high_contrast = Theme::high_contrast();
        assert_eq!(high_contrast.name, "High Contrast");
        assert_eq!(high_contrast.colors.primary, Color::Yellow);
    }

    #[test]
    fn test_theme_default() {
        let default_theme = Theme::default();
        assert_eq!(default_theme.name, "Dark");
    }

}
