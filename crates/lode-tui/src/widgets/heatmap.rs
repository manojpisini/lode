use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::theme::Theme;

pub struct Heatmap<'a> {
    data: &'a [[u8; 7]; 4],
    style: Style,
    block: Option<ratatui::widgets::Block<'a>>,
    theme: Option<&'a Theme>,
}

impl<'a> Heatmap<'a> {
    pub fn new(data: &'a [[u8; 7]; 4]) -> Self {
        Self {
            data,
            style: Style::default(),
            block: None,
            theme: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn block(mut self, block: ratatui::widgets::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn theme(mut self, theme: &'a Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    fn cell_char(val: u8) -> &'static str {
        match val {
            0 => " ",
            1 => "░",
            2 => "▒",
            3 => "▓",
            _ => "█",
        }
    }

    fn cell_style(&self, val: u8) -> Style {
        if let Some(t) = self.theme {
            match val {
                0 => t.dim_style(),
                1 => t.dim_style(),
                2 => t.accent_style(),
                3 => t.warn_style(),
                _ => t.success_style(),
            }
        } else {
            self.style
        }
    }
}

impl<'a> Widget for Heatmap<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width < 7 || inner.height < 4 {
            return;
        }

        for (week, row) in self.data.iter().enumerate() {
            for (day, &val) in row.iter().enumerate() {
                let x = inner.x + (day as u16) * 2;
                let y = inner.y + week as u16;

                let ch = Self::cell_char(val);
                let style = self.cell_style(val);

                buf.cell_mut((x, y)).unwrap().set_symbol(ch);
                buf.cell_mut((x, y)).unwrap().set_style(style);
            }
        }
    }
}
