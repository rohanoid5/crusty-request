use crate::app::{App, FocusedPane, InputMode, RequestTab};
use crate::highlight::Highlighter;
use crate::key_value::KeyValueWidget;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn ui(f: &mut Frame, app: &App) {
    // 1. Split Screen: Request (Top), Response (Middle), Footer (Bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Request
            Constraint::Min(5),         // Response
            Constraint::Length(3),      // Footer
        ])
        .split(f.area());

    let request_area = chunks[0];
    let response_area = chunks[1];
    let footer_area = chunks[2];

    // --- Request Section ---
    let request_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Method + URL
            Constraint::Min(0),    // Headers/Body
        ])
        .split(request_area);

    let top_row = request_chunks[0];
    let details_area = request_chunks[1];

    // Method + URL Row
    let url_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Method Dropdown
            Constraint::Min(0),     // URL Input
        ])
        .split(top_row);

    // Render Method
    let method_str = format!(" {} ", app.method); // Pad for looks
    let method_block = Block::default()
        .borders(Borders::ALL)
        .title("Method")
        .style(if app.focused_pane == FocusedPane::Method {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let method_p = Paragraph::new(method_str).block(method_block);
    f.render_widget(method_p, url_chunks[0]);

    // Render URL
    let url_block = Block::default().borders(Borders::ALL).title("URL").style(
        if app.focused_pane == FocusedPane::Url {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        },
    );
    let url_p = Paragraph::new(app.url_input.as_str()).block(url_block);
    f.render_widget(url_p, url_chunks[1]);

    // Request / Body Area
    let details_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(details_area);

    // Request Key-Value Entries
    let request_block = Block::default()
        .borders(Borders::ALL)
        .title("Request")
        .style(if app.focused_pane == FocusedPane::RequestDetails {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    // Split request area vertically: tab bar at top, content below
    let request_inner = request_block.inner(details_chunks[0]);
    f.render_widget(request_block, details_chunks[0]);

    let request_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Key-value content
        ])
        .split(request_inner);

    // Render tab bar with all three tabs
    let tabs = vec![
        ("Headers", RequestTab::Headers),
        ("Params", RequestTab::Params),
        ("Auth", RequestTab::Authorization),
    ];

    let mut tab_spans = Vec::new();
    for (i, (label, tab)) in tabs.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::raw(" "));
        }

        let style = if *tab == app.active_request_tab {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        tab_spans.push(Span::styled(format!("[{}]", label), style));
    }

    let tab_line = Line::from(tab_spans);
    let tab_paragraph = Paragraph::new(tab_line);
    f.render_widget(tab_paragraph, request_sections[0]);

    // Render key-value widget for active tab
    let active_entries = match app.active_request_tab {
        RequestTab::Headers => &app.headers,
        RequestTab::Params => &app.params,
        RequestTab::Authorization => &app.authorization,
    };

    let is_editing =
        app.input_mode == InputMode::Editing && app.focused_pane == FocusedPane::RequestDetails;

    let kv_widget = KeyValueWidget::new(active_entries)
        .focused(app.focused_pane == FocusedPane::RequestDetails)
        .editing(is_editing);

    kv_widget.render(f, request_sections[1]);

    // Body - with validation error styling
    let has_error = app.validation_error.is_some();
    let body_style = if has_error {
        Style::default().fg(Color::Red)
    } else if app.focused_pane == FocusedPane::Body {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let body_block = Block::default()
        .borders(Borders::ALL)
        .title(app.get_validation_status())
        .style(body_style);

    let mut body_textarea = app.body_input.clone();
    body_textarea.set_block(body_block);
    f.render_widget(&body_textarea, details_chunks[1]);

    // --- Response Section ---
    let response_block = Block::default()
        .borders(Borders::ALL)
        .title(if let Some(status) = app.response_status {
            format!("Response (Status: {})", status)
        } else {
            "Response".to_string()
        })
        .style(if app.focused_pane == FocusedPane::Response {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let content = app.response_text.as_deref().unwrap_or("No response yet...");

    // Apply syntax highlighting for JSON responses
    let highlighted_content = if content != "No response yet..." && content != "Loading..." {
        let highlighter = Highlighter::new();
        let lines = highlighter.highlight_json(content);
        Text::from(lines)
    } else {
        Text::raw(content)
    };

    let response_p = Paragraph::new(highlighted_content)
        .block(response_block)
        .wrap(Wrap { trim: false })
        .scroll((app.response_scroll, 0));
    f.render_widget(response_p, response_area);

    // --- Footer Section ---
    let help_msg = match app.input_mode {
        InputMode::Normal => {
            " [Tab] Next Pane | [Space] Cycle Method | [i] Edit | [Enter] Send | [q] Quit "
        }
        InputMode::Editing => " [Esc] Finish Editing ",
    };
    let footer =
        Paragraph::new(help_msg).block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(footer, footer_area);
}
