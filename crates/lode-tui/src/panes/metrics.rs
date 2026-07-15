use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Paragraph, Sparkline};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    render_sparkline(frame, chunks[0], app);
    render_coverage(frame, chunks[1], app);
    render_issues(frame, chunks[2], app);
}

fn render_sparkline(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let data = &app.project_data.metrics.sparkline;

    let block = theme.block_style().title("Trend");
    let spark = Sparkline::default()
        .block(block)
        .data(data)
        .style(theme.accent_style());
    frame.render_widget(spark, area);
}

fn render_coverage(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let coverage = app.project_data.metrics.coverage;

    let label = format!("{:.1}%", coverage);
    let ratio = (coverage / 100.0).clamp(0.0, 1.0);

    let gauge = Gauge::default()
        .block(theme.block_style().title("Coverage"))
        .gauge_style(Style::default().fg(theme.success))
        .ratio(ratio)
        .label(Span::styled(label, theme.fg_style()));
    frame.render_widget(gauge, area);
}

fn render_issues(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let issues = app.project_data.metrics.issues;
    let warnings = app.project_data.metrics.warnings;

    let lines = vec![
        Line::from(vec![
            Span::styled("  Issues: ", theme.dim_style()),
            Span::styled(
                issues.to_string(),
                if issues > 0 {
                    theme.error_style()
                } else {
                    theme.success_style()
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("  Warnings: ", theme.dim_style()),
            Span::styled(
                warnings.to_string(),
                if warnings > 0 {
                    theme.warn_style()
                } else {
                    theme.success_style()
                },
            ),
        ]),
    ];

    let block = theme.block_style().title("Issues");
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}
