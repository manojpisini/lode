use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Terminal;

use lode_tui::app::{App, AppResult, Pane};
use lode_tui::panes;
use lode_tui::widgets::status_bar::StatusBar;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let project_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "lode".to_string());

    let mut app = App::new(&project_name);

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(size);

            render_tab_bar(f, main_chunks[0], app);
            panes::render_pane(app.active_pane, f, main_chunks[1], app);
            render_status_bar(f, main_chunks[2], app);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.handle_key(key) {
                    AppResult::Quit => return Ok(()),
                    AppResult::Continue => {}
                }
            }
        }

        app.tick();
    }
}

fn render_tab_bar(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let titles: Vec<ratatui::text::Line> = Pane::all()
        .iter()
        .map(|p| {
            let style = if *p == app.active_pane {
                app.theme.highlight_style()
            } else {
                app.theme.dim_style()
            };
            ratatui::text::Line::from(ratatui::text::Span::styled(
                format!(" [{}] {} ", p.key_hint(), p.label()),
                style,
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(app.theme.border_style()),
        )
        .select(
            Pane::all()
                .iter()
                .position(|p| *p == app.active_pane)
                .unwrap_or(0),
        )
        .style(app.theme.bg_style())
        .highlight_style(app.theme.accent_style());

    f.render_widget(tabs, area);
}

fn render_status_bar(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let bar = StatusBar::new(app.active_pane, &app.theme);
    f.render_widget(bar, area);
}
