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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyValueField {
    Key,
    Value,
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
            key,
            value,
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
                map.insert(entry.key.clone(), entry.value.clone());
            }
        }
        map
    }

    pub fn to_header_map(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for entry in &self.entries {
            if entry.enabled {
                if let (Ok(name), Ok(value)) = (
                    reqwest::header::HeaderName::from_bytes(entry.key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(&entry.value),
                ) {
                    headers.insert(name, value);
                }
            }
        }
        headers
    }
}

/// Widget for rendering key-value entries in a two-column layout
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

    /// Render the key-value widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Split area into two columns: Key (50%) | Value (50%)
        let columns = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Render Key column
        self.render_column(f, columns[0], KeyValueField::Key);

        // Render Value column
        self.render_column(f, columns[1], KeyValueField::Value);
    }

    fn render_column(&self, f: &mut Frame, area: Rect, field: KeyValueField) {
        let mut lines = Vec::new();

        // Column header
        let header_text = match field {
            KeyValueField::Key => "Key",
            KeyValueField::Value => "Value",
        };
        lines.push(Line::from(Span::styled(
            header_text,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        // Render entries
        for (idx, entry) in self.entries.entries.iter().enumerate() {
            let is_selected = idx == self.entries.focused_index;
            let is_active_field = self.entries.focused_field == field;
            let text = match field {
                KeyValueField::Key => &entry.key,
                KeyValueField::Value => &entry.value,
            };

            let mut style = Style::default();

            // Highlight selected row
            if is_selected && self.is_focused {
                style = style.bg(Color::DarkGray);
            }

            // Highlight active field with cursor indicator
            if is_selected && is_active_field && self.is_editing {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
            }

            // Show disabled entries in gray
            if !entry.enabled {
                style = style.fg(Color::Gray);
            }

            // Add checkbox indicator for enabled/disabled
            let checkbox = if entry.enabled { "☑" } else { "☐" };
            let display_text = if matches!(field, KeyValueField::Key) {
                format!("{} {}", checkbox, text)
            } else {
                text.to_string()
            };

            // Add cursor indicator for active field
            let final_text = if is_selected && is_active_field && self.is_editing {
                format!("{}_", display_text)
            } else {
                display_text
            };

            lines.push(Line::from(Span::styled(final_text, style)));
        }

        // Show empty row placeholder
        if self.entries.entries.is_empty() {
            lines.push(Line::from(Span::styled(
                "(empty - press Enter to add)",
                Style::default().fg(Color::DarkGray),
            )));
        } else if self.is_focused && self.entries.focused_index == self.entries.entries.len() {
            // User is on the "add new" row
            let style = Style::default().bg(Color::DarkGray);
            lines.push(Line::from(Span::styled("(add new entry)", style)));
        }

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, area);
    }
}
