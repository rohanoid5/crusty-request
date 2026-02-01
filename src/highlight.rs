use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

/// Holds the syntax highlighting configuration
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    /// Create a new Highlighter with default syntect assets
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    /// Highlight JSON text and return styled ratatui Lines
    pub fn highlight_json<'a>(&self, text: &'a str) -> Vec<Line<'a>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_extension("json")
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines = Vec::new();

        for line in LinesWithEndings::from(text) {
            let highlighted = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();

            let spans: Vec<Span> = highlighted
                .into_iter()
                .map(|(style, content)| {
                    Span::styled(content.to_string(), convert_syntect_style(style))
                })
                .collect();

            lines.push(Line::from(spans));
        }

        lines
    }
}

/// Convert syntect Style to ratatui Style
fn convert_syntect_style(syntect_style: syntect::highlighting::Style) -> Style {
    let fg = syntect_style.foreground;
    let fg_color = Color::Rgb(fg.r, fg.g, fg.b);

    let mut style = Style::default().fg(fg_color);

    // Apply font styles
    if syntect_style.font_style.contains(FontStyle::BOLD) {
        style = style.add_modifier(Modifier::BOLD);
    }
    if syntect_style.font_style.contains(FontStyle::ITALIC) {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if syntect_style.font_style.contains(FontStyle::UNDERLINE) {
        style = style.add_modifier(Modifier::UNDERLINED);
    }

    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_json() {
        let highlighter = Highlighter::new();
        let json = r#"{"key": "value", "number": 42}"#;
        let lines = highlighter.highlight_json(json);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_highlight_empty_string() {
        let highlighter = Highlighter::new();
        let lines = highlighter.highlight_json("");
        assert!(lines.is_empty());
    }
}
