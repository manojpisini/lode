mod activity;
mod deps;
mod files;
mod metrics;
mod overview;
mod registry;
mod time;

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::{App, Pane};

pub fn render_pane(pane: Pane, frame: &mut Frame, area: Rect, app: &App) {
    match pane {
        Pane::Overview => overview::render(frame, area, app),
        Pane::Metrics => metrics::render(frame, area, app),
        Pane::Time => time::render(frame, area, app),
        Pane::Activity => activity::render(frame, area, app),
        Pane::Deps => deps::render(frame, area, app),
        Pane::Files => files::render(frame, area, app),
        Pane::Registry => registry::render(frame, area, app),
    }
}
