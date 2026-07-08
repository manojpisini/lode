use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

pub struct ScoreRing {
    score: u8,
    label: String,
    style: Style,
    block: Option<ratatui::widgets::Block<'static>>,
    theme: Option<Theme>,
}

impl ScoreRing {
    pub fn new(score: u8) -> Self {
        Self {
            score,
            label: String::new(),
            style: Style::default(),
            block: None,
            theme: None,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn block(mut self, block: ratatui::widgets::Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn theme(mut self, theme: &Theme) -> Self {
        self.theme = Some(theme.clone());
        self
    }

    fn score_color(&self) -> Style {
        if let Some(t) = &self.theme {
            if self.score >= 90 {
                t.success_style()
            } else if self.score >= 70 {
                t.warn_style()
            } else {
                t.error_style()
            }
        } else {
            self.style
        }
    }
}

impl Widget for ScoreRing {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width < 7 || inner.height < 3 {
            return;
        }

        let cx = inner.x + inner.width / 2;
        let cy = inner.y + inner.height / 2;

        let filled = ((self.score as f64 / 100.0) * 8.0) as usize;
        let style = self.score_color();

        let segments = [
            (cx - 1, cy - 1, '╭'),
            (cx, cy - 1, '─'),
            (cx + 1, cy - 1, '╮'),
            (cx - 1, cy + 1, '╰'),
            (cx, cy + 1, '─'),
            (cx + 1, cy + 1, '╯'),
            (cx - 1, cy, '│'),
            (cx + 1, cy, '│'),
        ];

        for (i, (x, y, ch)) in segments.iter().enumerate() {
            if i < filled {
                buf.cell_mut((*x, *y)).unwrap().set_symbol(&ch.to_string());
                buf.cell_mut((*x, *y)).unwrap().set_style(style);
            } else {
                buf.cell_mut((*x, *y)).unwrap().set_symbol(&ch.to_string());
                buf.cell_mut((*x, *y))
                    .unwrap()
                    .set_style(self.theme.as_ref().map_or(self.style, |t| t.dim_style()));
            }
        }

        let score_text = format!("{}", self.score);
        let score_style = style.add_modifier(Modifier::BOLD);
        buf.set_string(
            cx - score_text.len() as u16 / 2,
            cy,
            &score_text,
            score_style,
        );

        if !self.label.is_empty() {
            let label_style = self.theme.as_ref().map_or(self.style, |t| t.dim_style());
            buf.set_string(
                cx - self.label.len() as u16 / 2,
                cy + 2,
                &self.label,
                label_style,
            );
        }
    }
}
