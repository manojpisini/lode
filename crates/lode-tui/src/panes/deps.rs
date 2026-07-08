use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);

    render_audit_status(frame, chunks[0], app);
    render_outdated_list(frame, chunks[1], app);
}

fn render_audit_status(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let vulns = app.project_data.deps_vulns;
    let ok = app.project_data.audit_ok;

    let status = if ok { "PASSED" } else { "FAILED" };
    let style = if ok {
        theme.success_style()
    } else {
        theme.error_style()
    };

    let line = Line::from(vec![
        Span::styled("  Audit: ", theme.dim_style()),
        Span::styled(status, style),
        Span::styled("  |  Vulnerabilities: ", theme.dim_style()),
        Span::styled(
            vulns.to_string(),
            if vulns > 0 {
                theme.error_style()
            } else {
                theme.success_style()
            },
        ),
    ]);

    let block = theme.block_style().title("Audit");
    let widget = ratatui::widgets::Paragraph::new(line).block(block);
    frame.render_widget(widget, area);
}

fn render_outdated_list(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let outdated = &app.project_data.deps_outdated;

    let items: Vec<Row> = outdated
        .iter()
        .map(|dep| {
            Row::new(vec![
                Cell::from(dep.as_str()),
                Cell::from(Span::styled("outdated", theme.warn_style())),
            ])
        })
        .collect();

    let widths = [Constraint::Percentage(60), Constraint::Percentage(40)];

    let table = Table::new(items, widths)
        .header(
            Row::new(vec![Cell::from("Package"), Cell::from("Status")]).style(theme.accent_style()),
        )
        .block(theme.block_style().title("Outdated Packages"));

    frame.render_widget(table, area);
}
