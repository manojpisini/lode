use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let block = theme.block_style().title("Live Events");
    let text = Line::from(Span::styled(
        "  No live events (IPC unavailable)",
        theme.dim_style(),
    ));
    let para = Paragraph::new(text).block(block);
    frame.render_widget(para, area);
}
