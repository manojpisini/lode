use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub warn: Color,
    pub error: Color,
    pub success: Color,
    pub border: Color,
    pub text: Color,
    pub dim: Color,
    pub highlight: Color,
}

pub fn dark_theme() -> Theme {
    Theme {
        bg: Color::Rgb(18, 18, 24),
        fg: Color::Rgb(220, 220, 230),
        accent: Color::Rgb(100, 180, 255),
        warn: Color::Rgb(255, 180, 50),
        error: Color::Rgb(255, 80, 80),
        success: Color::Rgb(80, 220, 120),
        border: Color::Rgb(60, 60, 80),
        text: Color::Rgb(180, 180, 200),
        dim: Color::Rgb(100, 100, 120),
        highlight: Color::Rgb(140, 200, 255),
    }
}

impl Theme {
    pub fn bg_style(&self) -> Style {
        Style::default().bg(self.bg)
    }

    pub fn fg_style(&self) -> Style {
        Style::default().fg(self.fg)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warn_style(&self) -> Style {
        Style::default().fg(self.warn)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.dim)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn block_style(&self) -> ratatui::widgets::Block<'static> {
        ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(self.border_style())
            .style(self.bg_style())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_theme_has_correct_colors() {
        let t = dark_theme();
        assert_eq!(t.bg, Color::Rgb(18, 18, 24));
        assert_eq!(t.fg, Color::Rgb(220, 220, 230));
        assert_eq!(t.accent, Color::Rgb(100, 180, 255));
        assert_eq!(t.warn, Color::Rgb(255, 180, 50));
        assert_eq!(t.error, Color::Rgb(255, 80, 80));
        assert_eq!(t.success, Color::Rgb(80, 220, 120));
        assert_eq!(t.border, Color::Rgb(60, 60, 80));
        assert_eq!(t.text, Color::Rgb(180, 180, 200));
        assert_eq!(t.dim, Color::Rgb(100, 100, 120));
        assert_eq!(t.highlight, Color::Rgb(140, 200, 255));
    }

    #[test]
    fn bg_style_has_no_foreground() {
        let t = dark_theme();
        let s = t.bg_style();
        assert_eq!(s.bg, Some(t.bg));
    }

    #[test]
    fn accent_style_is_bold() {
        let t = dark_theme();
        let s = t.accent_style();
        assert_eq!(s.fg, Some(t.accent));
        assert!(s.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn highlight_style_is_bold() {
        let t = dark_theme();
        let s = t.highlight_style();
        assert_eq!(s.fg, Some(t.highlight));
        assert!(s.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn title_style_uses_accent_color() {
        let t = dark_theme();
        let s = t.title_style();
        assert_eq!(s.fg, Some(t.accent));
        assert!(s.add_modifier.contains(Modifier::BOLD));
    }
}
