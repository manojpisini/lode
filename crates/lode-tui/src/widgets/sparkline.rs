use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

#[derive(Default)]
pub struct Sparkline<'a> {
    data: Vec<u64>,
    style: Style,
    block: Option<ratatui::widgets::Block<'a>>,
}

impl<'a> Sparkline<'a> {
    pub fn data(mut self, data: &[u64]) -> Self {
        self.data = data.to_vec();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn block(mut self, block: ratatui::widgets::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for Sparkline<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width == 0 || inner.height == 0 || self.data.is_empty() {
            return;
        }

        let max = self.data.iter().copied().max().unwrap_or(1).max(1);
        let width = inner.width as usize;

        let downsampled = if self.data.len() > width {
            let step = self.data.len() as f64 / width as f64;
            (0..width)
                .map(|i| {
                    let start = (i as f64 * step) as usize;
                    let end = ((i + 1) as f64 * step) as usize;
                    let slice = &self.data[start..end.min(self.data.len())];
                    slice.iter().copied().max().unwrap_or(0)
                })
                .collect::<Vec<_>>()
        } else {
            self.data.clone()
        };

        for (i, &val) in downsampled.iter().enumerate() {
            let normalized = ((val as f64 / max as f64) * 7.0) as usize;
            let idx = normalized.min(7);
            let ch = BLOCKS[idx];
            let x = inner.x + i as u16;
            if let Some(cell) = buf.cell_mut((x, inner.y)) {
                cell.set_symbol(&ch.to_string());
                cell.set_style(self.style);
            }
        }
    }
}
