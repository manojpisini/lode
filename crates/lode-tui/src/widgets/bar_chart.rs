use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::theme::Theme;

const BAR_WIDTH: u16 = 20;

pub struct BarChart {
    data: Vec<(String, u64)>,
    style: Style,
    block: Option<ratatui::widgets::Block<'static>>,
    theme: Option<Theme>,
}

impl BarChart {
    pub fn new(data: Vec<(String, u64)>) -> Self {
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

    pub fn block(mut self, block: ratatui::widgets::Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn theme(mut self, theme: &Theme) -> Self {
        self.theme = Some(theme.clone());
        self
    }
}

impl Widget for BarChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.height == 0 || self.data.is_empty() {
            return;
        }

        let max = self.data.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
        let bar_rows = inner.height.min(self.data.len() as u16);

        for (i, (label, value)) in self.data.iter().take(bar_rows as usize).enumerate() {
            let y = inner.y + i as u16;
            let bar_len = ((*value as f64 / max as f64) * BAR_WIDTH as f64) as u16;

            let label_width = (inner.width - BAR_WIDTH - 1).min(12);
            let truncated: String = label.chars().take(label_width as usize).collect();

            let label_style = self.theme.as_ref().map_or(self.style, |t| t.dim_style());
            let bar_style = self.theme.as_ref().map_or(self.style, |t| t.accent_style());

            buf.set_string(inner.x, y, &truncated, label_style);

            let bar_x = inner.x + label_width + 1;
            for bx in 0..bar_len {
                if bar_x + bx < inner.x + inner.width {
                    buf.cell_mut((bar_x + bx, y)).unwrap().set_symbol("█");
                    buf.cell_mut((bar_x + bx, y)).unwrap().set_style(bar_style);
                }
            }
        }
    }
}
