use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::widgets::score_ring::ScoreRing;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    render_health_ring(frame, chunks[0], app);
    render_checks_and_info(frame, chunks[1], app);
}

fn render_health_ring(frame: &mut Frame, area: Rect, app: &App) {
    let ring = ScoreRing::new(app.project_data.health_score)
        .label("Health")
        .theme(&app.theme);
    frame.render_widget(ring, area);
}

fn render_checks_and_info(frame: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_sub_checks(frame, cols[0], app);
    render_project_info(frame, cols[1], app);
}

fn render_sub_checks(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let checks = &app.project_data.checks;

    let items = vec![
        ("Convention", checks.convention),
        ("Secrets", checks.secrets),
        ("License", checks.license),
        ("Env", checks.env),
        ("Readme", checks.readme),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (name, ok) in items {
        let icon = if ok { "✓" } else { "✗" };
        let style = if ok {
            theme.success_style()
        } else {
            theme.error_style()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", icon), style),
            Span::styled(name, theme.fg_style()),
        ]));
    }

    let block = theme.block_style().title("Sub-checks");
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn render_project_info(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let data = &app.project_data;

    let lines = vec![
        Line::from(vec![
            Span::styled("  Name: ", theme.dim_style()),
            Span::styled(&data.name, theme.accent_style()),
        ]),
        Line::from(vec![
            Span::styled("  Path: ", theme.dim_style()),
            Span::styled(&data.path, theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled("  Profile: ", theme.dim_style()),
            Span::styled(&data.profile, theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled("  Languages: ", theme.dim_style()),
            Span::styled(data.languages.join(", "), theme.fg_style()),
        ]),
    ];

    let block = theme.block_style().title("Project Info");
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}
