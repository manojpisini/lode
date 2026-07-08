use ratatui::layout::{Constraint, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let registry = &app.project_data.registry;

    let rows: Vec<Row> = registry
        .iter()
        .map(|entry| {
            let score_style = if entry.score >= 90 {
                theme.success_style()
            } else if entry.score >= 70 {
                theme.warn_style()
            } else {
                theme.error_style()
            };

            Row::new(vec![
                Cell::from(entry.name.as_str()),
                Cell::from(Span::styled(entry.score.to_string(), score_style)),
                Cell::from(entry.status.as_str()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(40),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from("Project"),
                Cell::from("Score"),
                Cell::from("Status"),
            ])
            .style(theme.accent_style()),
        )
        .block(theme.block_style().title("Registry"));

    frame.render_widget(table, area);
}
