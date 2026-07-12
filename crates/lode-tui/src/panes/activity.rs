use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem};
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let events = &app.project_data.events;
    let theme = &app.theme;

    let items: Vec<ListItem> = events
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .map(|ev| {
            let text = format_event(ev, theme);
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items).block(theme.block_style().title("Live Events"));
    frame.render_widget(list, area);
}

fn format_event(ev: &crate::ipc::DaemonEvent, theme: &Theme) -> Line<'static> {
    match ev {
        crate::ipc::DaemonEvent::FileChanged { path } => Line::from(vec![
            Span::styled("  FILE ", theme.accent_style()),
            Span::raw(path.clone()),
        ]),
        crate::ipc::DaemonEvent::ConventionViolation { file, rule } => Line::from(vec![
            Span::styled("  VIOL ", theme.error_style()),
            Span::raw(format!("{}: {}", file, rule)),
        ]),
        crate::ipc::DaemonEvent::SecretFound { file, line } => Line::from(vec![
            Span::styled("  SEC  ", theme.error_style()),
            Span::raw(format!("{}:{}", file, line)),
        ]),
        crate::ipc::DaemonEvent::BuildStarted => Line::from(vec![
            Span::styled("  BUILD ", theme.warn_style()),
            Span::raw("started"),
        ]),
        crate::ipc::DaemonEvent::BuildFinished { success } => {
            let style = if *success {
                theme.success_style()
            } else {
                theme.error_style()
            };
            let label = if *success { "ok" } else { "FAILED" };
            Line::from(vec![Span::styled("  BUILD ", style), Span::raw(label)])
        }
        crate::ipc::DaemonEvent::TestRan { passed, failed } => Line::from(vec![
            Span::styled("  TEST  ", theme.accent_style()),
            Span::raw(format!("{} passed, {} failed", passed, failed)),
        ]),
        crate::ipc::DaemonEvent::LintReported { errors, warnings } => Line::from(vec![
            Span::styled("  LINT  ", theme.warn_style()),
            Span::raw(format!("{} errors, {} warnings", errors, warnings)),
        ]),
        crate::ipc::DaemonEvent::HealthChecked { score } => Line::from(vec![
            Span::styled("  SCORE ", theme.success_style()),
            Span::raw(format!("{}", score)),
        ]),
    }
}
