use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::widgets::bar_chart::BarChart;
use crate::widgets::heatmap::Heatmap;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),
            Constraint::Length(6),
            Constraint::Length(9),
        ])
        .split(area);

    render_sessions(frame, chunks[0], app);
    render_dir_bar(frame, chunks[1], app);
    render_heatmap(frame, chunks[2], app);
}

fn render_sessions(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let sessions = &app.project_data.session_today;

    let header = Row::new(vec![
        Cell::from("Time"),
        Cell::from("Duration"),
        Cell::from("Files"),
    ])
    .style(theme.accent_style());

    let rows: Vec<Row> = sessions
        .iter()
        .map(|s| {
            Row::new(vec![
                Cell::from(s.time.as_str()),
                Cell::from(s.duration.as_str()),
                Cell::from(s.files_changed.to_string()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(theme.block_style().title("Sessions Today"));

    frame.render_widget(table, area);
}

fn render_dir_bar(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let data: Vec<(String, u64)> = app.project_data.dir_bar.clone();

    let bar = BarChart::new(data).theme(theme);
    let block = theme.block_style().title("Directory Activity");
    frame.render_widget(bar.block(block), area);
}

fn render_heatmap(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let hm = Heatmap::new(&app.project_data.heatmap).theme(theme);
    let block = theme.block_style().title("4-Week Activity");
    frame.render_widget(hm.block(block), area);
}
