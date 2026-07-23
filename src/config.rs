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
