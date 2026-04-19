use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crossterm::{
    cursor, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::list_picker;

pub fn select_model(models: &[String]) -> io::Result<Option<String>> {
    if models.is_empty() {
        println!("Select model: ");
        return Ok(None);
    }

    let models = ordered_models(models);
    let selected = list_picker::select_index(models.len(), |stdout, selected, offset| {
        draw(stdout, &models, selected, offset)
    })?;

    Ok(selected.map(|index| models[index].model.clone()))
}

pub fn print_model_list(models: &[String]) -> io::Result<()> {
    let mut stdout = io::stdout();

    for model in ordered_models(models) {
        if model.installed {
            writeln!(stdout, "{} (installed)", model.model)?;
        } else {
            writeln!(stdout, "{}", model.model)?;
        }
    }

    Ok(())
}

fn draw(
    stdout: &mut io::Stdout,
    models: &[ModelEntry],
    selected: usize,
    offset: usize,
) -> io::Result<()> {
    let (_, rows) = terminal::size()?;
    let visible_rows = rows.saturating_sub(2).max(1) as usize;
    let end = (offset + visible_rows).min(models.len());
    let max_width = terminal::size()?.0 as usize;

    queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::CurrentLine))?;
    writeln!(stdout, "Select model: ")?;

    for (index, model) in models[offset..end].iter().enumerate() {
        let absolute_index = offset + index;
        let y = (index + 1) as u16;
        let prefix = if absolute_index == selected {
            "> "
        } else {
            "  "
        };
        let suffix = if model.installed {
            " (installed)"
        } else {
            ""
        };
        let line = format_model_line(prefix, &model.model, suffix, max_width);

        queue!(stdout, cursor::MoveTo(0, y), Clear(ClearType::CurrentLine))?;

        if absolute_index == selected {
            let foreground = if model.installed {
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
        } else if model.installed {
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

    for y in (end - offset + 1)..=visible_rows {
        queue!(stdout, cursor::MoveTo(0, y as u16), Clear(ClearType::CurrentLine))?;
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

fn ordered_models(models: &[String]) -> Vec<ModelEntry> {
    let mut entries: Vec<ModelEntry> = models
        .iter()
        .map(|model| ModelEntry {
            model: model.clone(),
            installed: model_is_installed(model),
        })
        .collect();

    sort_model_entries(&mut entries);
    entries
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
        roots.push(
            PathBuf::from(home)
                .join(".cache")
                .join("huggingface")
                .join("hub"),
        );
    }

    roots
}

fn repo_cache_dir_name(model: &str) -> String {
    format!("models--{}", model.replace('/', "--"))
}

fn sort_model_entries(entries: &mut [ModelEntry]) {
    entries.sort_by(|a, b| b.installed.cmp(&a.installed));
}

#[derive(Clone)]
struct ModelEntry {
    model: String,
    installed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_models_put_installed_items_first() {
        let mut entries = vec![
            ModelEntry {
                model: "org/uninstalled-a".to_string(),
                installed: false,
            },
            ModelEntry {
                model: "org/installed-a".to_string(),
                installed: true,
            },
            ModelEntry {
                model: "org/uninstalled-b".to_string(),
                installed: false,
            },
            ModelEntry {
                model: "org/installed-b".to_string(),
                installed: true,
            },
        ];

        sort_model_entries(&mut entries);

        let ordered: Vec<_> = entries.into_iter().map(|entry| entry.model).collect();

        assert_eq!(
            ordered,
            vec![
                "org/installed-a".to_string(),
                "org/installed-b".to_string(),
                "org/uninstalled-a".to_string(),
                "org/uninstalled-b".to_string(),
            ]
        );
    }
}
