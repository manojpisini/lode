#![deny(unsafe_code)]

use std::io::{self, BufRead, Write};

// ── ANSI color helpers ──

fn ansi(code: &str, text: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

pub fn green(text: &str) -> String { ansi("32", text) }
pub fn red(text: &str) -> String { ansi("31", text) }
pub fn yellow(text: &str) -> String { ansi("33", text) }
pub fn cyan(text: &str) -> String { ansi("36", text) }
pub fn bold(text: &str) -> String { ansi("1", text) }
pub fn dim(text: &str) -> String { ansi("2", text) }
pub fn white(text: &str) -> String { ansi("37", text) }

// ── Status symbols ──

pub fn ok(text: &str) -> String { format!("{} {}", green("✔"), text) }
pub fn fail(text: &str) -> String { format!("{} {}", red("✘"), text) }
pub fn warn(text: &str) -> String { format!("{} {}", yellow("⚠"), text) }
pub fn info(text: &str) -> String { format!("{} {}", cyan("ℹ"), text) }
pub fn step_done(n: usize, kind: &str, run: &str) -> String {
    format!("  {} {} [{}] {}", green("✔"), dim(&format!("{n}.")), cyan(kind), run)
}
pub fn step_fail(n: usize, kind: &str, run: &str) -> String {
    format!("  {} {} [{}] {}", red("✘"), dim(&format!("{n}.")), cyan(kind), run)
}
pub fn step_running(n: usize, kind: &str, run: &str) -> String {
    format!("  {} {} [{}] {}", cyan("▶"), dim(&format!("{n}.")), cyan(kind), run)
}
pub fn step_skip(n: usize, kind: &str, run: &str) -> String {
    format!("  {} {} [{}] {}", dim("−"), dim(&format!("{n}.")), dim(kind), dim(run))
}

// ── Section header ──

pub fn section(title: &str) -> String {
    let line = "─".repeat(60.min(80usize.saturating_sub(title.len() + 2)));
    format!("\n {} {} {}", bold(title), dim("─"), dim(&line))
}

pub fn sub_section(title: &str) -> String {
    format!(" {} {}", cyan("▸"), bold(title))
}

// ── Simple table ──

pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() || rows.is_empty() {
        return String::new();
    }
    let col_count = headers.len();
    let col_widths: Vec<usize> = (0..col_count)
        .map(|c| {
            headers[c]
                .len()
                .max(rows.iter().filter_map(|r| r.get(c)).map(|r| r.len()).max().unwrap_or(0))
        })
        .collect();

    let sep = |left: &str, mid: &str, right: &str, fill: &str| -> String {
        let mut s = left.to_string();
        for (i, w) in col_widths.iter().enumerate() {
            if i > 0 { s.push_str(mid); }
            s.push_str(&fill.repeat(*w + 2));
        }
        s.push_str(right);
        s
    };

    let mut out = String::new();
    out.push_str(&sep("┌", "┬", "┐", "─"));
    out.push('\n');

    for (i, h) in headers.iter().enumerate() {
        if i == 0 { out.push_str("│"); }
        out.push_str(&format!(" {:<width$} ", bold(h), width = col_widths[i]));
        out.push_str("│");
    }
    out.push('\n');
    out.push_str(&sep("├", "┼", "┤", "─"));
    out.push('\n');

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i == 0 { out.push_str("│"); }
            out.push_str(&format!(" {:<width$} ", cell, width = col_widths[i]));
            out.push_str("│");
        }
        out.push('\n');
    }

    out.push_str(&sep("└", "┴", "┘", "─"));
    out.push('\n');
    out
}

// ── Structured error ──

pub fn format_error(title: &str, details: &[&str]) -> String {
    let mut out = format!("{} {}\n", red("✘"), bold(title));
    for detail in details {
        out.push_str(&format!("  {} {}\n", dim("│"), detail));
    }
    out
}

pub fn format_io_error(path: &str, source: &str) -> String {
    format!(
        "{} {} {}\n  {} {} {}",
        red("✘"),
        bold("IO error:"),
        cyan(path),
        dim("└─"),
        dim("cause:"),
        dim(source)
    )
}

// ── Prompts ──

pub fn confirm(prompt: &str) -> io::Result<bool> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        write!(stdout, "{} {} {} ", cyan("?"), bold(prompt), dim("[y/N]"))?;
        stdout.flush()?;
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => {
                writeln!(stdout, "  {} please type 'y' or 'n'", yellow("⚠"))?;
            }
        }
    }
}

pub fn input(prompt: &str) -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    write!(stdout, "{} {} ", cyan("▶"), bold(prompt))?;
    stdout.flush()?;
    let mut input = String::new();
    stdin.lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn select(prompt: &str, options: &[&str]) -> io::Result<usize> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    writeln!(stdout, "{} {}", cyan("?"), bold(prompt))?;
    for (i, opt) in options.iter().enumerate() {
        writeln!(stdout, "  {} {} {}", dim(&format!("{}.", i + 1)), cyan(&format!("{}", i + 1)), opt)?;
    }
    loop {
        write!(stdout, "  {} {} ", cyan("▸"), dim("choice (1-{}):"))?;
        stdout.flush()?;
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        if let Ok(n) = input.trim().parse::<usize>() {
            if n >= 1 && n <= options.len() {
                return Ok(n - 1);
            }
        }
    }
}

pub fn multi_select(prompt: &str, options: &[&str]) -> io::Result<Vec<usize>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    writeln!(stdout, "{} {} {}", cyan("?"), bold(prompt), dim("(comma-separated)"))?;
    for (i, opt) in options.iter().enumerate() {
        writeln!(stdout, "  {} {} {}", dim(&format!("{}.", i + 1)), cyan(&format!("{}", i + 1)), opt)?;
    }
    loop {
        write!(stdout, "  {} {} ", cyan("▸"), dim("choices (e.g. 1,3,5):"))?;
        stdout.flush()?;
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let indices: Vec<usize> = input
            .trim()
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .filter(|n| *n >= 1 && *n <= options.len())
            .map(|n| n - 1)
            .collect();
        if !indices.is_empty() {
            return Ok(indices);
        }
    }
}

// ── Progress bar ──

pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
    message: String,
}

impl ProgressBar {
    pub fn new(total: usize, message: &str) -> Self {
        let pb = Self {
            total,
            current: 0,
            width: 30,
            message: message.to_string(),
        };
        pb.render();
        pb
    }

    pub fn inc(&mut self, delta: usize) {
        self.current = self.current.saturating_add(delta).min(self.total);
        self.render();
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.render();
    }

    pub fn finish(&mut self) {
        self.current = self.total;
        self.render();
        println!();
    }

    fn render(&self) {
        let pct = if self.total == 0 {
            100.0
        } else {
            self.current as f64 / self.total as f64 * 100.0
        };
        let filled = ((pct / 100.0) * self.width as f64).round() as usize;
        let empty = self.width.saturating_sub(filled);
        let bar = format!(
            "\r{} {}% {}{} {}",
            cyan("■"),
            dim(&format!("{:3.0}", pct)),
            green(&"█".repeat(filled)),
            dim(&"░".repeat(empty)),
            dim(&self.message)
        );
        let _ = print!("{bar}");
        let _ = io::stdout().flush();
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if self.current < self.total {
            println!();
        }
    }
}

// ── Spinner ──

pub struct Spinner {
    message: String,
    done: bool,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let s = Self {
            message: message.to_string(),
            done: false,
        };
        print!("{} {} ... ", cyan("⟳"), s.message);
        let _ = io::stdout().flush();
        s
    }

    pub fn done(&mut self) {
        if !self.done {
            self.done = true;
            println!("{}", green("done"));
        }
    }

    pub fn fail(&mut self, detail: &str) {
        if !self.done {
            self.done = true;
            println!("{} {}", red("failed"), dim(detail));
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if !self.done {
            println!("{}", dim("(interrupted)"));
        }
    }
}
