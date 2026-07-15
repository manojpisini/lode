#![deny(unsafe_code)]

use std::io::{self, BufRead, Write};

// ── ANSI color helpers ──

fn ansi(code: &str, text: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

pub fn green(text: &str) -> String {
    ansi("32", text)
}
pub fn red(text: &str) -> String {
    ansi("31", text)
}
pub fn yellow(text: &str) -> String {
    ansi("33", text)
}
pub fn cyan(text: &str) -> String {
    ansi("36", text)
}
pub fn bold(text: &str) -> String {
    ansi("1", text)
}
pub fn dim(text: &str) -> String {
    ansi("2", text)
}
// ── Status symbols ──

pub fn ok(text: &str) -> String {
    format!("{} {}", green("✔"), text)
}
pub fn fail(text: &str) -> String {
    format!("{} {}", red("✘"), text)
}
pub fn warn(text: &str) -> String {
    format!("{} {}", yellow("⚠"), text)
}
pub fn info(text: &str) -> String {
    format!("{} {}", cyan("ℹ"), text)
}

// ── Section header ──

pub fn section(title: &str) -> String {
    let line = "─".repeat(60.min(80usize.saturating_sub(title.len() + 2)));
    format!("\n {} {} {}", bold(title), dim("─"), dim(&line))
}

// ── Simple table ──

pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() || rows.is_empty() {
        return String::new();
    }
    let col_count = headers.len();
    let col_widths: Vec<usize> = (0..col_count)
        .map(|c| {
            headers[c].len().max(
                rows.iter()
                    .filter_map(|r| r.get(c))
                    .map(|r| r.len())
                    .max()
                    .unwrap_or(0),
            )
        })
        .collect();

    let sep = |left: &str, mid: &str, right: &str, fill: &str| -> String {
        let mut s = left.to_string();
        for (i, w) in col_widths.iter().enumerate() {
            if i > 0 {
                s.push_str(mid);
            }
            s.push_str(&fill.repeat(*w + 2));
        }
        s.push_str(right);
        s
    };

    let mut out = String::new();
    out.push_str(&sep("┌", "┬", "┐", "─"));
    out.push('\n');

    for (i, h) in headers.iter().enumerate() {
        if i == 0 {
            out.push_str("│");
        }
        out.push_str(&format!(" {:<width$} ", bold(h), width = col_widths[i]));
        out.push_str("│");
    }
    out.push('\n');
    out.push_str(&sep("├", "┼", "┤", "─"));
    out.push('\n');

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i == 0 {
                out.push_str("│");
            }
            out.push_str(&format!(" {:<width$} ", cell, width = col_widths[i]));
            out.push_str("│");
        }
        out.push('\n');
    }

    out.push_str(&sep("└", "┴", "┘", "─"));
    out.push('\n');
    out
}

// ── Simple input prompt ──

pub fn input(prompt: &str) -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    write!(stdout, "{} {} ", cyan("▶"), bold(prompt))?;
    stdout.flush()?;
    let mut input = String::new();
    stdin.lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
