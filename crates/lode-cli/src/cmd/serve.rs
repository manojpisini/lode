#![deny(unsafe_code)]

use std::io::IsTerminal;
use std::time::Duration;
use std::{env, fs, io};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lode_core::{audit_project, default_config, load_global_config, load_registry, LodeError};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

pub fn serve_impl(
    no_color: bool,
    no_live: bool,
    initial_pane: Option<&str>,
) -> lode_core::Result<()> {
    if no_live || !io::stdout().is_terminal() {
        return serve_dashboard_snapshot(no_color, initial_pane);
    }

    enable_raw_mode().map_err(crate::terminal_error)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(crate::terminal_error)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(crate::terminal_error)?;

    let result = run_live_dashboard(&mut terminal, no_color, initial_pane);

    disable_raw_mode().map_err(crate::terminal_error)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(crate::terminal_error)?;
    terminal.show_cursor().map_err(crate::terminal_error)?;

    result
}

fn run_live_dashboard(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    no_color: bool,
    initial_pane: Option<&str>,
) -> lode_core::Result<()> {
    let mut selected = dashboard_pane_index(initial_pane)?;
    loop {
        let data = dashboard_data(no_color)?;
        terminal
            .draw(|frame| draw_live_dashboard(frame, &data, selected, no_color))
            .map_err(crate::terminal_error)?;

        if event::poll(Duration::from_millis(750)).map_err(crate::terminal_error)? {
            if let Event::Key(key) = event::read().map_err(crate::terminal_error)? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Tab | KeyCode::Down | KeyCode::Right => selected = (selected + 1) % 8,
                    KeyCode::BackTab | KeyCode::Up | KeyCode::Left => {
                        selected = selected.checked_sub(1).unwrap_or(7);
                    }
                    KeyCode::Char('1') => selected = 0,
                    KeyCode::Char('2') => selected = 1,
                    KeyCode::Char('3') => selected = 2,
                    KeyCode::Char('4') => selected = 3,
                    KeyCode::Char('5') => selected = 4,
                    KeyCode::Char('6') => selected = 5,
                    KeyCode::Char('7') => selected = 6,
                    KeyCode::Char('8') => selected = 7,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct DashboardData {
    project: String,
    env_name: String,
    score: u8,
    convention_violations: usize,
    secret_findings: usize,
    license_present: bool,
    env_example_present: bool,
    readme_present: bool,
    daemon_state: String,
    events: Vec<String>,
    registry: Vec<String>,
    package_manager: String,
    toolchains: String,
    rust_version: String,
    git_version: String,
    time_total: String,
    time_sessions: usize,
}

fn dashboard_data(no_color: bool) -> lode_core::Result<DashboardData> {
    let cwd = crate::current_dir()?;
    let project = cwd
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "project".to_string());
    let config = load_global_config().unwrap_or_else(|_| default_config());
    let audit = audit_project(&cwd, &config)?;
    let registry = load_registry().unwrap_or_default();
    let daemon_state = fs::read_to_string(crate::daemon_state_path()?)
        .unwrap_or_else(|_| "inactive".to_string())
        .trim()
        .to_string();
    let daemon_runtime = crate::load_daemon_runtime_state().unwrap_or_default();
    let daemon_log = fs::read_to_string(crate::daemon_log_path()?).unwrap_or_default();
    let time_log = crate::load_time_log().unwrap_or_default();
    let color = Palette::new(no_color);
    let registry = if registry.projects.is_empty() {
        vec!["No registered projects".to_string()]
    } else {
        registry
            .projects
            .iter()
            .take(8)
            .map(|project| {
                format!(
                    "{}  {}  {}",
                    project.name,
                    if project.path.exists() {
                        color.green("HEALTHY")
                    } else {
                        color.red("MISSING")
                    },
                    project.path
                )
            })
            .collect()
    };

    Ok(DashboardData {
        project,
        env_name: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
        score: audit.score,
        convention_violations: audit.convention_violations,
        secret_findings: audit.secret_findings,
        license_present: audit.license_present,
        env_example_present: audit.env_example_present,
        readme_present: audit.readme_present,
        daemon_state,
        events: recent_log_lines(&daemon_runtime.recent_events, &daemon_log),
        registry,
        package_manager: crate::detect_package_manager().unwrap_or_else(|| "unknown".to_string()),
        toolchains: crate::detect_toolchains().join(", "),
        rust_version: crate::command_version("rustc").unwrap_or_else(|| "missing".to_string()),
        git_version: crate::command_version("git").unwrap_or_else(|| "missing".to_string()),
        time_total: crate::format_seconds(crate::total_seconds(&time_log.sessions)),
        time_sessions: time_log.sessions.len(),
    })
}

fn draw_live_dashboard(frame: &mut Frame, data: &DashboardData, selected: usize, no_color: bool) {
    let theme = DashboardTheme::new(no_color);
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("◇ lode serve", theme.accent.add_modifier(Modifier::BOLD)),
        Span::raw(format!(
            "  {}  env:{}  health:{}",
            data.project, data.env_name, data.score
        )),
    ]))
    .block(Block::default().borders(Borders::ALL).style(theme.panel));
    frame.render_widget(title, vertical[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(50)])
        .split(vertical[1]);

    let nav_items = [
        "Overview", "Health", "Metrics", "Events", "Deps", "Registry", "Config", "Logs",
    ]
    .iter()
    .enumerate()
    .map(|(index, label)| {
        let marker = if selected == index { "›" } else { " " };
        let style = if selected == index {
            theme.accent.add_modifier(Modifier::BOLD)
        } else {
            theme.text
        };
        ListItem::new(Line::from(vec![
            Span::styled(marker, style),
            Span::raw(format!(" {} [{}]", label, index + 1)),
        ]))
    })
    .collect::<Vec<_>>();
    frame.render_widget(
        List::new(nav_items).block(
            Block::default()
                .title(" NAVIGATION ")
                .borders(Borders::ALL)
                .style(theme.panel),
        ),
        body[0],
    );

    match selected {
        0 => draw_overview(frame, body[1], data, &theme),
        1 => draw_health(frame, body[1], data, &theme),
        2 => draw_metrics_panel(frame, body[1], data, &theme),
        3 => draw_lines_panel(frame, body[1], " LIVE DAEMON EVENTS ", &data.events, &theme),
        4 => draw_deps(frame, body[1], data, &theme),
        5 => draw_lines_panel(
            frame,
            body[1],
            " CROSS-PROJECT REGISTRY ",
            &data.registry,
            &theme,
        ),
        6 => draw_config_panel(frame, body[1], data, &theme),
        _ => draw_lines_panel(frame, body[1], " LOGS ", &data.events, &theme),
    }

    let footer = Paragraph::new(" ↑↓/Tab move  1-8 jump  q quit  auto-refresh 750ms ")
        .style(theme.dim)
        .block(Block::default().borders(Borders::ALL).style(theme.panel));
    frame.render_widget(footer, vertical[2]);
}

#[derive(Debug, Clone, Copy)]
struct DashboardTheme {
    panel: Style,
    text: Style,
    dim: Style,
    accent: Style,
    good: Style,
    warn: Style,
    bad: Style,
}

impl DashboardTheme {
    fn new(no_color: bool) -> Self {
        if no_color {
            Self {
                panel: Style::default(),
                text: Style::default(),
                dim: Style::default(),
                accent: Style::default().add_modifier(Modifier::BOLD),
                good: Style::default(),
                warn: Style::default(),
                bad: Style::default(),
            }
        } else {
            Self {
                panel: Style::default().fg(Color::Rgb(194, 202, 204)),
                text: Style::default().fg(Color::Rgb(222, 226, 226)),
                dim: Style::default().fg(Color::Rgb(116, 126, 128)),
                accent: Style::default().fg(Color::Rgb(91, 223, 207)),
                good: Style::default().fg(Color::Rgb(118, 220, 151)),
                warn: Style::default().fg(Color::Rgb(238, 197, 104)),
                bad: Style::default().fg(Color::Rgb(238, 109, 109)),
            }
        }
    }
}

fn draw_overview(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Min(6),
        ])
        .split(area);
    frame.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(" PROJECT HEALTH ")
                    .borders(Borders::ALL),
            )
            .gauge_style(if data.score >= 85 {
                theme.good
            } else if data.score >= 60 {
                theme.warn
            } else {
                theme.bad
            })
            .percent(data.score as u16),
        chunks[0],
    );
    draw_health(frame, chunks[1], data, theme);
    draw_lines_panel(frame, chunks[2], " RECENT EVENTS ", &data.events, theme);
}

fn draw_health(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(vec![
            Span::raw("Convention      "),
            Span::styled(
                status_count_plain(data.convention_violations),
                if data.convention_violations == 0 {
                    theme.good
                } else {
                    theme.warn
                },
            ),
        ]),
        Line::from(vec![
            Span::raw("Secrets         "),
            Span::styled(
                status_count_plain(data.secret_findings),
                if data.secret_findings == 0 {
                    theme.good
                } else {
                    theme.bad
                },
            ),
        ]),
        Line::from(format!("License         {}", yes_no(data.license_present))),
        Line::from(format!(
            "Env example     {}",
            yes_no(data.env_example_present)
        )),
        Line::from(format!("Readme          {}", yes_no(data.readme_present))),
    ];
    frame.render_widget(
        Paragraph::new(lines).style(theme.text).block(
            Block::default()
                .title(" HEALTH CHECKS ")
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn draw_metrics_panel(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(format!("Score           {}", data.score)),
        Line::from(format!("Time today      {}", data.time_total)),
        Line::from(format!("Sessions        {}", data.time_sessions)),
        Line::from(format!("Daemon          {}", data.daemon_state)),
        Line::from(format!("Toolchains      {}", data.toolchains)),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.text)
            .block(
                Block::default()
                    .title(" METRICS TRENDS ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_deps(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(format!("Package manager {}", data.package_manager)),
        Line::from(format!("Rust            {}", data.rust_version)),
        Line::from(format!("Git             {}", data.git_version)),
        Line::from("Policy          strict"),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.text)
            .block(
                Block::default()
                    .title(" DEPENDENCY STATUS ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_config_panel(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(format!("Project         {}", data.project)),
        Line::from(format!("Environment     {}", data.env_name)),
        Line::from(format!("Daemon state    {}", data.daemon_state)),
        Line::from(format!("Package manager {}", data.package_manager)),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.text)
            .block(
                Block::default()
                    .title(" CONFIG SUMMARY ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_lines_panel(
    frame: &mut Frame,
    area: Rect,
    title: &'static str,
    lines: &[String],
    theme: &DashboardTheme,
) {
    let text = lines
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(text)
            .style(theme.text)
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn status_count_plain(count: usize) -> String {
    if count == 0 {
        "0 OK".to_string()
    } else {
        format!("{count} WARN")
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "OK"
    } else {
        "MISSING"
    }
}

fn serve_dashboard_snapshot(no_color: bool, initial_pane: Option<&str>) -> lode_core::Result<()> {
    let data = dashboard_data(no_color)?;
    let color = Palette::new(no_color);
    let selected = dashboard_pane_index(initial_pane)?;
    let selected_name = DASHBOARD_PANES[selected];

    println!("{}", color.cyan("◇ lode serve"));
    println!(
        "{}",
        rule(&format!(
            " Project: {} | Env: {} | Pane: {} | Health: {} | Warn: {} | Fail: {} ",
            color.cyan(&data.project),
            color.cyan(&data.env_name),
            color.cyan(selected_name),
            color.green(&data.score.to_string()),
            color.yellow(&data.convention_violations.to_string()),
            color.red(&data.secret_findings.to_string())
        ))
    );
    println!();
    println!(
        "{}  {}",
        pane(
            "NAVIGATION",
            &[
                &color.cyan("› Overview        [1]"),
                "  Health          [2]",
                "  Metrics         [3]",
                "  Events          [4]",
                "  Dependencies    [5]",
                "  Registry        [6]",
                "  Config          [7]",
                "  Logs            [8]",
            ],
            30
        ),
        pane(
            "1. PROJECT HEALTH",
            &[
                &format!("Overall Status  {}", health_label(data.score, &color)),
                &format!("Score           {}", color.cyan(&data.score.to_string())),
                &format!(
                    "Convention      {}",
                    status_count(data.convention_violations, &color)
                ),
                &format!(
                    "Secrets         {}",
                    status_count(data.secret_findings, &color)
                ),
                &format!(
                    "License         {}",
                    bool_label(data.license_present, &color)
                ),
                &format!(
                    "Env Example     {}",
                    bool_label(data.env_example_present, &color)
                ),
                &format!(
                    "Readme          {}",
                    bool_label(data.readme_present, &color)
                ),
            ],
            56
        )
    );
    println!(
        "{}  {}",
        pane(
            "2. METRICS TRENDS",
            &[
                &format!("Health      {} {}", color.cyan("████████░░"), data.score),
                &format!(
                    "Checks      {}",
                    color.green("convention · secrets · license · env")
                ),
                &format!("Toolchain   {}", data.toolchains),
                &format!("Package     {}", data.package_manager),
            ],
            56
        ),
        pane(
            "3. DAEMON / TIME",
            &[
                &format!("Daemon State  {}", color.cyan(&data.daemon_state)),
                &format!("Active Session  {} session(s)", data.time_sessions),
                &format!("Today           {}", data.time_total),
                "Focus Score     derived metrics pending",
            ],
            56
        )
    );
    println!(
        "{}  {}",
        pane(
            "4. LIVE DAEMON EVENTS",
            &data.events.iter().map(String::as_str).collect::<Vec<_>>(),
            70
        ),
        pane(
            "5. DEPENDENCY STATUS",
            &[
                &format!("Manager  {}", data.package_manager),
                &format!("Rust     {}", data.rust_version),
                &format!("Git      {}", data.git_version),
                "Policy   strict",
            ],
            42
        )
    );
    println!(
        "{}",
        pane(
            "6. CROSS-PROJECT REGISTRY",
            &data.registry.iter().map(String::as_str).collect::<Vec<_>>(),
            116
        )
    );
    println!(
        "{}",
        rule(" ↑↓ Move   Tab Next   Enter Open   r Refresh   q Quit   Auto-refresh: OFF ")
    );
    Ok(())
}

const DASHBOARD_PANES: [&str; 8] = [
    "overview", "health", "metrics", "activity", "deps", "registry", "config", "logs",
];

fn dashboard_pane_index(pane: Option<&str>) -> lode_core::Result<usize> {
    let Some(pane) = pane else {
        return Ok(0);
    };
    DASHBOARD_PANES
        .iter()
        .position(|candidate| *candidate == pane)
        .ok_or_else(|| LodeError::Message(format!("unsupported dashboard pane: {pane}")))
}

struct Palette {
    enabled: bool,
}

impl Palette {
    fn new(no_color: bool) -> Self {
        Self { enabled: !no_color }
    }

    fn cyan(&self, text: &str) -> String {
        self.paint("36", text)
    }

    fn green(&self, text: &str) -> String {
        self.paint("32", text)
    }

    fn yellow(&self, text: &str) -> String {
        self.paint("33", text)
    }

    fn red(&self, text: &str) -> String {
        self.paint("31", text)
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

fn pane(title: &str, lines: &[&str], width: usize) -> String {
    let inner = width.saturating_sub(2);
    let mut output = String::new();
    let title_inner = format!(" {title} ");
    output.push_str(&format!("┌{:<inner$}┐\n", title_inner, inner = inner));
    for line in lines {
        output.push_str(&format!(
            "│ {:<pad$}│\n",
            truncate_ansi(line, inner.saturating_sub(1)),
            pad = inner.saturating_sub(1)
        ));
    }
    output.push_str(&format!("└{:<inner$}┘", "", inner = inner));
    output
}

fn rule(text: &str) -> String {
    format!("┤{text}├")
}

fn truncate_ansi(text: &str, width: usize) -> String {
    let plain_len = text.chars().filter(|ch| *ch != '\x1b').count();
    if plain_len <= width {
        text.to_string()
    } else {
        text.chars()
            .take(width.saturating_sub(1))
            .collect::<String>()
            + "…"
    }
}

fn health_label(score: u8, color: &Palette) -> String {
    if score >= 85 {
        color.green("● HEALTHY")
    } else if score >= 60 {
        color.yellow("● WARN")
    } else {
        color.red("● FAIL")
    }
}

fn status_count(count: usize, color: &Palette) -> String {
    if count == 0 {
        color.green("0 OK")
    } else {
        color.yellow(&format!("{count} WARN"))
    }
}

fn bool_label(value: bool, color: &Palette) -> String {
    if value {
        color.green("OK")
    } else {
        color.red("MISSING")
    }
}

fn recent_log_lines(recent_events: &[crate::DaemonEvent], log: &str) -> Vec<String> {
    if !recent_events.is_empty() {
        return recent_events
            .iter()
            .rev()
            .take(6)
            .map(|event| event.message.clone())
            .rev()
            .collect();
    }
    let mut lines = log
        .lines()
        .rev()
        .take(6)
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines.reverse();
    if lines.is_empty() {
        vec!["No daemon events yet".to_string()]
    } else {
        lines
    }
}
