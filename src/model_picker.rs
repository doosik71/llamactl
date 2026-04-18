use std::io::{self, Write};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn select_model(models: &[String]) -> io::Result<Option<String>> {
    if models.is_empty() {
        println!("Select model: ");
        return Ok(None);
    }

    let installed = installed_flags(models);
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let result = run_picker(&mut stdout, models, &installed);

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result
}

fn run_picker(
    stdout: &mut io::Stdout,
    models: &[String],
    installed: &[bool],
) -> io::Result<Option<String>> {
    let mut selected = 0usize;
    let mut offset = 0usize;

    loop {
        draw(stdout, models, installed, selected, offset)?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key_event) = event::read()? else {
            continue;
        };

        match key_event.code {
            KeyCode::Up => {
                selected = selected.saturating_sub(1);
                if selected < offset {
                    offset = selected;
                }
            }
            KeyCode::Down => {
                if selected + 1 < models.len() {
                    selected += 1;
                }
                let visible_rows = visible_rows()?;
                if selected >= offset + visible_rows {
                    offset = selected + 1 - visible_rows;
                }
            }
            KeyCode::PageUp => {
                let visible_rows = visible_rows()?;
                selected = selected.saturating_sub(visible_rows);
                offset = offset.saturating_sub(visible_rows);
                if selected < offset {
                    offset = selected;
                }
            }
            KeyCode::PageDown => {
                let visible_rows = visible_rows()?;
                selected = (selected + visible_rows).min(models.len() - 1);
                if selected >= offset + visible_rows {
                    offset = selected + 1 - visible_rows;
                }
            }
            KeyCode::Enter => {
                let chosen = models[selected].clone();
                return Ok(Some(chosen));
            }
            KeyCode::Esc => {
                return Ok(None);
            }
            _ => {}
        }
    }
}

fn draw(
    stdout: &mut io::Stdout,
    models: &[String],
    installed: &[bool],
    selected: usize,
    offset: usize,
) -> io::Result<()> {
    let (_, rows) = terminal::size()?;
    let visible_rows = rows.saturating_sub(2).max(1) as usize;
    let end = (offset + visible_rows).min(models.len());
    let max_width = terminal::size()?.0 as usize;

    queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    writeln!(stdout, "Select model: ")?;

    for (index, model) in models[offset..end].iter().enumerate() {
        let absolute_index = offset + index;
        let y = (index + 1) as u16;
        let prefix = if absolute_index == selected { "> " } else { "  " };
        let suffix = if installed.get(absolute_index).copied().unwrap_or(false) {
            " (installed)"
        } else {
            ""
        };
        let line = format_model_line(prefix, model, suffix, max_width);

        queue!(
            stdout,
            cursor::MoveTo(0, y),
            Clear(ClearType::CurrentLine)
        )?;

        if absolute_index == selected {
            let foreground = if installed.get(absolute_index).copied().unwrap_or(false) {
                Color::Green
            } else {
                Color::Reset
            };
            queue!(
                stdout,
                SetAttribute(Attribute::Reverse),
                SetForegroundColor(foreground),
                Print(line),
                SetAttribute(Attribute::Reset),
                ResetColor
            )?;
        } else if installed.get(absolute_index).copied().unwrap_or(false) {
            queue!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(line),
                ResetColor
            )?;
        } else {
            queue!(stdout, Print(line))?;
        }
    }

    if rows > 1 {
        queue!(
            stdout,
            cursor::MoveTo(0, rows - 1),
            Clear(ClearType::CurrentLine)
        )?;
        write!(stdout, "↑/↓ to move, Enter to select, Esc to exit")?;
    }

    stdout.flush()?;
    Ok(())
}

fn visible_rows() -> io::Result<usize> {
    let (_, rows) = terminal::size()?;
    Ok(rows.saturating_sub(2).max(1) as usize)
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let char_count = text.chars().count();
    if char_count <= max_width {
        return text.to_string();
    }

    if max_width == 1 {
        return "…".to_string();
    }

    let mut result = String::new();
    for ch in text.chars().take(max_width.saturating_sub(1)) {
        result.push(ch);
    }
    result.push('…');
    result
}

fn format_model_line(prefix: &str, model: &str, suffix: &str, max_width: usize) -> String {
    let prefix_width = prefix.chars().count();
    let suffix_width = suffix.chars().count();

    if max_width <= prefix_width + suffix_width {
        return truncate_to_width(prefix, max_width);
    }

    let available = max_width - prefix_width - suffix_width;
    let model_part = truncate_to_width(model, available);
    let mut line = String::with_capacity(prefix.len() + model_part.len() + suffix.len());
    line.push_str(prefix);
    line.push_str(&model_part);
    line.push_str(suffix);
    line
}

fn installed_flags(models: &[String]) -> Vec<bool> {
    models.iter().map(|model| model_is_installed(model)).collect()
}

fn model_is_installed(model: &str) -> bool {
    cache_roots()
        .into_iter()
        .map(|root| root.join(repo_cache_dir_name(model)).join("snapshots"))
        .any(|snapshots| snapshots.is_dir() && has_entries(&snapshots))
}

fn has_entries(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => false,
    }
}

fn cache_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(path) = std::env::var_os("HUGGINGFACE_HUB_CACHE") {
        roots.push(PathBuf::from(path));
    }

    if let Some(path) = std::env::var_os("HF_HOME") {
        roots.push(PathBuf::from(path).join("hub"));
    }

    if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
        roots.push(PathBuf::from(path).join("huggingface").join("hub"));
    }

    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".cache").join("huggingface").join("hub"));
    }

    roots
}

fn repo_cache_dir_name(model: &str) -> String {
    format!("models--{}", model.replace('/', "--"))
}
