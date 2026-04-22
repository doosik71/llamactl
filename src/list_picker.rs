use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Clone, Debug)]
pub struct PickerItem {
    pub display: String,
    pub value: String,
    pub color: Option<Color>,
}

pub fn select_value(items: &[PickerItem], title: &str) -> io::Result<Option<String>> {
    if items.is_empty() {
        return Ok(None);
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let result = run_picker(&mut stdout, items, title);

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result
}

fn run_picker(
    stdout: &mut io::Stdout,
    items: &[PickerItem],
    title: &str,
) -> io::Result<Option<String>> {
    let mut query = String::new();
    let mut filtered_indices = filter_items(items, &query);
    let mut selected = 0usize;
    let mut offset = 0usize;

    draw(
        stdout,
        title,
        &query,
        items,
        &filtered_indices,
        selected,
        offset,
    )?;

    loop {
        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key_event) = event::read()? else {
            continue;
        };

        let mut changed = false;
        match key_event.code {
            KeyCode::Up => {
                if selected > 0 {
                    selected -= 1;
                    if selected < offset {
                        offset = selected;
                    }
                    changed = true;
                }
            }
            KeyCode::Down => {
                if selected + 1 < filtered_indices.len() {
                    selected += 1;
                    let visible_rows = visible_rows()?;
                    if selected >= offset + visible_rows {
                        offset = selected + 1 - visible_rows;
                    }
                    changed = true;
                }
            }
            KeyCode::PageUp => {
                if !filtered_indices.is_empty() {
                    let visible_rows = visible_rows()?;
                    let next_selected = selected.saturating_sub(visible_rows);
                    if next_selected != selected {
                        selected = next_selected;
                        offset = offset.saturating_sub(visible_rows);
                        if selected < offset {
                            offset = selected;
                        }
                        changed = true;
                    }
                }
            }
            KeyCode::PageDown => {
                if !filtered_indices.is_empty() {
                    let visible_rows = visible_rows()?;
                    let next_selected = (selected + visible_rows).min(filtered_indices.len() - 1);
                    if next_selected != selected {
                        selected = next_selected;
                        if selected >= offset + visible_rows {
                            offset = selected + 1 - visible_rows;
                        }
                        changed = true;
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(index) = filtered_indices.get(selected) {
                    return Ok(Some(items[*index].value.clone()));
                }
            }
            KeyCode::Esc => {
                return Ok(None);
            }
            KeyCode::Backspace => {
                if query.pop().is_some() {
                    filtered_indices = filter_items(items, &query);
                    selected = 0;
                    offset = 0;
                    changed = true;
                }
            }
            KeyCode::Char(ch)
                if !key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && !key_event.modifiers.contains(KeyModifiers::ALT) =>
            {
                query.push(ch);
                filtered_indices = filter_items(items, &query);
                selected = 0;
                offset = 0;
                changed = true;
            }
            _ => {}
        }

        if changed {
            draw(
                stdout,
                title,
                &query,
                items,
                &filtered_indices,
                selected,
                offset,
            )?;
        }
    }
}

fn draw(
    stdout: &mut io::Stdout,
    title: &str,
    query: &str,
    items: &[PickerItem],
    filtered_indices: &[usize],
    selected: usize,
    offset: usize,
) -> io::Result<()> {
    let (_, rows) = terminal::size()?;
    let visible_rows = rows.saturating_sub(2).max(1) as usize;
    let end = (offset + visible_rows).min(filtered_indices.len());
    let max_width = terminal::size()?.0 as usize;

    queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::CurrentLine))?;
    let header = format!("{} {}", title, query);
    writeln!(stdout, "{}", truncate_to_width(&header, max_width))?;

    for (index, item_index) in filtered_indices[offset..end].iter().enumerate() {
        let absolute_index = offset + index;
        let y = (index + 1) as u16;
        let item = &items[*item_index];
        let prefix = if absolute_index == selected {
            "> "
        } else {
            "  "
        };
        let line = format_line(prefix, &item.display, max_width);

        queue!(stdout, cursor::MoveTo(0, y), Clear(ClearType::CurrentLine))?;

        if absolute_index == selected {
            queue!(stdout, SetAttribute(Attribute::Reverse))?;
        }

        if let Some(color) = item.color {
            queue!(stdout, SetForegroundColor(color))?;
        }

        queue!(
            stdout,
            Print(line),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;
    }

    let rendered_rows = end.saturating_sub(offset);
    for y in (rendered_rows + 1)..=visible_rows {
        queue!(
            stdout,
            cursor::MoveTo(0, y as u16),
            Clear(ClearType::CurrentLine)
        )?;
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

fn filter_items(items: &[PickerItem], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }

    let query_lower = query.to_lowercase();
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if item.display.to_lowercase().contains(&query_lower) {
                Some(index)
            } else {
                None
            }
        })
        .collect()
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

fn format_line(prefix: &str, text: &str, max_width: usize) -> String {
    let prefix_width = prefix.chars().count();
    if max_width <= prefix_width {
        return truncate_to_width(prefix, max_width);
    }

    let available = max_width - prefix_width;
    let text_part = truncate_to_width(text, available);
    let mut line = String::with_capacity(prefix.len() + text_part.len());
    line.push_str(prefix);
    line.push_str(&text_part);
    line
}

#[cfg(test)]
mod tests {
    use super::{PickerItem, filter_items};

    #[test]
    fn filter_items_matches_case_insensitive_substrings() {
        let items = vec![
            PickerItem {
                display: "Llama-3".to_string(),
                value: "llama-3".to_string(),
                color: None,
            },
            PickerItem {
                display: "Mistral".to_string(),
                value: "mistral".to_string(),
                color: None,
            },
            PickerItem {
                display: "Phi".to_string(),
                value: "phi".to_string(),
                color: None,
            },
        ];

        let filtered = filter_items(&items, "ll");

        assert_eq!(filtered, vec![0]);
    }

    #[test]
    fn filter_items_returns_all_items_for_empty_query() {
        let items = vec![
            PickerItem {
                display: "Llama-3".to_string(),
                value: "llama-3".to_string(),
                color: None,
            },
            PickerItem {
                display: "Mistral".to_string(),
                value: "mistral".to_string(),
                color: None,
            },
            PickerItem {
                display: "Phi".to_string(),
                value: "phi".to_string(),
                color: None,
            },
        ];

        let filtered = filter_items(&items, "");

        assert_eq!(filtered, vec![0, 1, 2]);
    }
}
