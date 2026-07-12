use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let tree = &app.project_data.file_tree;
    let violations = &app.project_data.violations;

    let items: Vec<ListItem> = tree
        .iter()
        .map(|node| {
            let prefix = if node.is_dir { " [DIR] " } else { " [FILE] " };
            let name = &node.name;

            let mut spans = vec![Span::raw(prefix.to_string())];

            if node.violation {
                spans.push(Span::styled(name.clone(), theme.error_style()));
            } else if node.signed {
                spans.push(Span::styled(name.clone(), theme.success_style()));
            } else {
                spans.push(Span::styled(name.clone(), theme.fg_style()));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut all_items = items;

    if !violations.is_empty() {
        all_items.push(ListItem::new(Line::from("")));
        all_items.push(ListItem::new(Line::from(Span::styled(
            "  Violations:",
            theme.error_style().add_modifier(Modifier::BOLD),
        ))));
        for v in violations {
            all_items.push(ListItem::new(Line::from(vec![
                Span::styled("    ✗ ", theme.error_style()),
                Span::raw(v.clone()),
            ])));
        }
    }

    let list = List::new(all_items).block(theme.block_style().title("Files"));
    frame.render_widget(list, area);
}
