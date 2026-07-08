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

pub fn light_theme() -> Theme {
    Theme {
        bg: Color::Rgb(245, 245, 250),
        fg: Color::Rgb(30, 30, 40),
        accent: Color::Rgb(40, 100, 200),
        warn: Color::Rgb(200, 140, 20),
        error: Color::Rgb(200, 50, 50),
        success: Color::Rgb(40, 160, 80),
        border: Color::Rgb(180, 180, 200),
        text: Color::Rgb(60, 60, 80),
        dim: Color::Rgb(150, 150, 170),
        highlight: Color::Rgb(60, 140, 220),
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
