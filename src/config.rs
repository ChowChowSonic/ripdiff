use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub added: Color,
    pub removed: Color,
    pub padding: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            added: Color::Green,
            removed: Color::Red,
            padding: Color::DarkGray,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default_colors() {
        let theme = Theme::default();
        assert_eq!(theme.added, Color::Green);
        assert_eq!(theme.removed, Color::Red);
        assert_eq!(theme.padding, Color::DarkGray);
    }

    #[test]
    fn test_theme_clone() {
        let theme = Theme::default();
        let cloned = theme.clone();
        assert_eq!(theme.added, cloned.added);
        assert_eq!(theme.removed, cloned.removed);
        assert_eq!(theme.padding, cloned.padding);
    }

    #[test]
    fn test_theme_debug_format() {
        let theme = Theme::default();
        let debug = format!("{:?}", theme);
        assert!(debug.contains("Green"));
        assert!(debug.contains("Red"));
        assert!(debug.contains("DarkGray"));
    }
}
