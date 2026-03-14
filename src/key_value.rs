use std::collections::HashMap;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::text_input::TextInput;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValueEntry {
    pub key: TextInput,
    pub value: TextInput,
    pub description: TextInput,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyValueField {
    Key,
    Value,
    Description,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValueEntries {
    pub entries: Vec<KeyValueEntry>,
    pub focused_index: usize,
    pub focused_field: KeyValueField,
}

impl KeyValueEntries {
    pub fn new() -> Self {
        Self {
            entries: vec![],
            focused_index: 0,
            focused_field: KeyValueField::Key,
        }
    }

    pub fn add_entry(&mut self, key: String, value: String) {
        self.entries.push(KeyValueEntry {
            key: TextInput::from(key),
            value: TextInput::from(value),
            description: TextInput::new(),
            enabled: true,
        })
    }

    pub fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
    }

    pub fn get_selected_mut(&mut self, index: usize) -> Option<&mut KeyValueEntry> {
        if index < self.entries.len() {
            Some(&mut self.entries[index])
        } else {
            None
        }
    }

    pub fn toggle_enabled(&mut self, index: usize) {
        if let Some(entry) = self.get_selected_mut(index) {
            entry.enabled = !entry.enabled;
        }
    }

    pub fn to_pairs(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for entry in &self.entries {
            if entry.enabled {
                map.insert(entry.key.as_str().to_string(), entry.value.as_str().to_string());
            }
        }
        map
    }

    pub fn to_header_map(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for entry in &self.entries {
            if entry.enabled {
                if let (Ok(name), Ok(value)) = (
                    reqwest::header::HeaderName::from_bytes(entry.key.as_str().as_bytes()),
                    reqwest::header::HeaderValue::from_str(entry.value.as_str()),
                ) {
                    headers.insert(name, value);
                }
            }
        }
        headers
    }

    /// Cycle to next field (Key -> Value -> Description -> Key)
    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            KeyValueField::Key => KeyValueField::Value,
            KeyValueField::Value => KeyValueField::Description,
            KeyValueField::Description => KeyValueField::Key,
        };
    }
}

/// Widget for rendering key-value entries in a table layout
/// Columns: [☑] Key | Value | Description
pub struct KeyValueWidget<'a> {
    entries: &'a KeyValueEntries,
    is_focused: bool,
    is_editing: bool,
}

impl<'a> KeyValueWidget<'a> {
    pub fn new(entries: &'a KeyValueEntries) -> Self {
        Self {
            entries,
            is_focused: false,
            is_editing: false,
        }
    }

    pub fn focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        self
    }

    pub fn editing(mut self, is_editing: bool) -> Self {
        self.is_editing = is_editing;
        self
    }

    /// Render the key-value table
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Split area into three columns: Key (35%) | Value (40%) | Description (25%)
        let columns = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(40),
                Constraint::Percentage(25),
            ])
            .split(area);

        self.render_column(f, columns[0], KeyValueField::Key);
        self.render_column(f, columns[1], KeyValueField::Value);
        self.render_column(f, columns[2], KeyValueField::Description);
    }

    fn render_column(&self, f: &mut Frame, area: Rect, field: KeyValueField) {
        let mut lines = Vec::new();

        // Column header with separator
        let header_text = match field {
            KeyValueField::Key => "  Key",
            KeyValueField::Value => "Value",
            KeyValueField::Description => "Description",
        };

        let header_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);

        lines.push(Line::from(Span::styled(header_text, header_style)));

        // Separator line
        let sep = "─".repeat(area.width as usize);
        lines.push(Line::from(Span::styled(
            sep,
            Style::default().fg(Color::DarkGray),
        )));

        // Render entries
        for (idx, entry) in self.entries.entries.iter().enumerate() {
            let is_selected = idx == self.entries.focused_index;
            let is_active_field = self.entries.focused_field == field;

            let text = match field {
                KeyValueField::Key => entry.key.as_str(),
                KeyValueField::Value => entry.value.as_str(),
                KeyValueField::Description => entry.description.as_str(),
            };

            let mut style = Style::default();

            // Highlight selected row
            if is_selected && self.is_focused {
                style = style.bg(Color::Rgb(40, 40, 50));
            }

            // Highlight active field with cursor indicator
            if is_selected && is_active_field && self.is_editing {
                style = style.fg(Color::Rgb(130, 170, 255)).add_modifier(Modifier::BOLD);
            }

            // Show disabled entries dimmed
            if !entry.enabled {
                style = style.fg(Color::DarkGray);
            }

            // Add checkbox for key column
            let checkbox = if entry.enabled { "☑ " } else { "☐ " };
            let is_editing_this_field = is_selected && is_active_field && self.is_editing;
            let display_text = if is_editing_this_field {
                let editing_text = match field {
                    KeyValueField::Key => entry.key.render_with_cursor(true),
                    KeyValueField::Value => entry.value.render_with_cursor(true),
                    KeyValueField::Description => entry.description.render_with_cursor(true),
                };
                if matches!(field, KeyValueField::Key) {
                    format!("{}{}", checkbox, editing_text)
                } else {
                    editing_text
                }
            } else {
                if matches!(field, KeyValueField::Key) {
                    format!("{}{}", checkbox, text)
                } else {
                    text.to_string()
                }
            };

            let final_text = if display_text.is_empty() && !matches!(field, KeyValueField::Key) && !is_editing_this_field {
                // Show placeholder for empty fields
                let placeholder = match field {
                    KeyValueField::Value => "Value",
                    KeyValueField::Description => "Description",
                    _ => "",
                };
                lines.push(Line::from(Span::styled(
                    placeholder,
                    Style::default().fg(Color::Rgb(60, 60, 70)),
                )));
                continue;
            } else {
                display_text
            };

            lines.push(Line::from(Span::styled(final_text, style)));
        }

        // Show empty row placeholder or "add new" row
        if self.entries.entries.is_empty() {
            let placeholder = match field {
                KeyValueField::Key => "  Key",
                KeyValueField::Value => "Value",
                KeyValueField::Description => "Description",
            };
            lines.push(Line::from(Span::styled(
                placeholder,
                Style::default().fg(Color::Rgb(60, 60, 70)),
            )));
        } else if self.is_focused && self.entries.focused_index == self.entries.entries.len() {
            let style = Style::default().fg(Color::Rgb(60, 60, 70));
            let text = match field {
                KeyValueField::Key => "  Key",
                KeyValueField::Value => "Value",
                KeyValueField::Description => "Description",
            };
            lines.push(Line::from(Span::styled(text, style)));
        }

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, area);
    }
}
