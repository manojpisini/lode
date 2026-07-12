use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::widgets::Widget;

use crate::theme::Theme;

pub struct StatusBar<'a> {
    theme: &'a Theme,
}

impl<'a> StatusBar<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hints = [
            ("1-7", "panes"),
            ("Tab", "next"),
            ("Shift-Tab", "prev"),
            ("q", "quit"),
        ];

        let mut spans = Vec::new();

        for (i, (key, label)) in hints.iter().enumerate() {
            if i > 0 {
                spans.push(ratatui::text::Span::raw("  "));
            }
            spans.push(ratatui::text::Span::styled(
                format!(" {} ", key),
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            ));
            spans.push(ratatui::text::Span::styled(
                format!("{} ", label),
                self.theme.dim_style(),
            ));
        }

        let line = ratatui::text::Line::from(spans);
        let para = ratatui::widgets::Paragraph::new(line).style(self.theme.bg_style());

        buf.set_style(area, self.theme.bg_style());
        para.render(area, buf);
    }
}
