use crate::app::{App, FocusedPane, InputMode, RequestTab, ResponseTab};
use crate::highlight::Highlighter;
use crate::key_value::KeyValueWidget;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Color palette — dark theme inspired by Postman
const BG_DARK: Color = Color::Rgb(30, 30, 35);
const BG_SURFACE: Color = Color::Rgb(38, 38, 45);
const FG_PRIMARY: Color = Color::Rgb(220, 220, 230);
const FG_DIM: Color = Color::Rgb(100, 100, 115);
const ACCENT_BLUE: Color = Color::Rgb(70, 130, 255);
const ACCENT_GREEN: Color = Color::Rgb(72, 199, 142);
const ACCENT_ORANGE: Color = Color::Rgb(255, 165, 60);
const ACCENT_RED: Color = Color::Rgb(255, 85, 85);
const BORDER_COLOR: Color = Color::Rgb(55, 55, 65);
const ACTIVE_BORDER: Color = Color::Rgb(70, 130, 255);

/// Method badge color
fn method_color(method: &crate::app::HttpMethod) -> Color {
    match method {
        crate::app::HttpMethod::GET => ACCENT_GREEN,
        crate::app::HttpMethod::POST => ACCENT_ORANGE,
        crate::app::HttpMethod::PUT => ACCENT_BLUE,
        crate::app::HttpMethod::DELETE => ACCENT_RED,
        crate::app::HttpMethod::PATCH => Color::Rgb(180, 130, 255),
    }
}

/// Status code color
fn status_color(status: u16) -> Color {
    match status {
        200..=299 => ACCENT_GREEN,
        300..=399 => ACCENT_ORANGE,
        400..=499 => ACCENT_RED,
        500..=599 => Color::Rgb(255, 50, 50),
        _ => FG_DIM,
    }
}

pub fn ui(f: &mut Frame, app: &App) {
    // Main vertical layout
    // ┌───────────────────────────┐
    // │ Method + URL + Send bar   │  3 lines
    // ├───────────────────────────┤
    // │ Tab bar                   │  1 line
    // ├───────────────────────────┤
    // │ Tab content               │  ~40%
    // ├───────────────────────────┤
    // │ Response status bar       │  1 line
    // ├───────────────────────────┤
    // │ Response body             │  remaining
    // ├───────────────────────────┤
    // │ Footer / Controls         │  1 line
    // └───────────────────────────┘

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Method + URL + Send
            Constraint::Length(1),      // Tab bar
            Constraint::Percentage(35), // Tab content
            Constraint::Length(1),      // Response status bar
            Constraint::Min(5),         // Response body
            Constraint::Length(1),      // Footer
        ])
        .split(f.area());

    let url_bar_area = chunks[0];
    let tab_bar_area = chunks[1];
    let tab_content_area = chunks[2];
    let response_status_area = chunks[3];
    let response_body_area = chunks[4];
    let footer_area = chunks[5];

    // ─── URL Bar ───────────────────────────────────────────
    render_url_bar(f, app, url_bar_area);

    // ─── Tab Bar ───────────────────────────────────────────
    render_tab_bar(f, app, tab_bar_area);

    // ─── Tab Content ───────────────────────────────────────
    render_tab_content(f, app, tab_content_area);

    // ─── Response Status Bar ───────────────────────────────
    render_response_status_bar(f, app, response_status_area);

    // ─── Response Body ─────────────────────────────────────
    render_response_body(f, app, response_body_area);

    // ─── Footer ────────────────────────────────────────────
    render_footer(f, app, footer_area);
}

fn render_url_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),     // Method
            Constraint::Min(0),        // URL
            Constraint::Length(8),     // Send button
        ])
        .split(area);

    // Method badge
    let method_clr = method_color(&app.method);
    let method_border_style = if app.focused_pane == FocusedPane::Method {
        Style::default().fg(ACTIVE_BORDER)
    } else {
        Style::default().fg(BORDER_COLOR)
    };
    let method_block = Block::default()
        .borders(Borders::ALL)
        .border_style(method_border_style);
    let method_text = Paragraph::new(Line::from(Span::styled(
        format!(" {} ", app.method),
        Style::default().fg(method_clr).add_modifier(Modifier::BOLD),
    )))
    .block(method_block);
    f.render_widget(method_text, chunks[0]);

    // URL input
    let url_border_style = if app.focused_pane == FocusedPane::Url {
        Style::default().fg(ACTIVE_BORDER)
    } else {
        Style::default().fg(BORDER_COLOR)
    };
    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(url_border_style);

    let url_display = if app.url_input.is_empty() && app.input_mode != InputMode::Editing {
        Line::from(Span::styled(
            "Enter request URL...",
            Style::default().fg(FG_DIM),
        ))
    } else {
        let is_editing = app.focused_pane == FocusedPane::Url && app.input_mode == InputMode::Editing;
        Line::from(Span::styled(
            app.url_input.render_with_cursor(is_editing),
            Style::default().fg(FG_PRIMARY),
        ))
    };

    let url_text = Paragraph::new(url_display).block(url_block);
    f.render_widget(url_text, chunks[1]);

    // Send button
    let send_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT_BLUE));
    let send_text = Paragraph::new(Line::from(Span::styled(
        " Send ",
        Style::default()
            .fg(Color::White)
            .bg(ACCENT_BLUE)
            .add_modifier(Modifier::BOLD),
    )))
    .block(send_block);
    f.render_widget(send_text, chunks[2]);
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tabs = vec![
        ("Params", RequestTab::Params),
        ("Auth", RequestTab::Authorization),
        ("Headers", RequestTab::Headers),
        ("Body", RequestTab::Body),
    ];

    let mut spans = Vec::new();
    spans.push(Span::styled(" ", Style::default()));

    for (i, (label, tab)) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                "  ",
                Style::default().fg(BORDER_COLOR),
            ));
        }

        let is_active = app.active_request_tab == *tab;
        let has_content = app.tab_has_content(tab);

        let mut style = if is_active {
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(FG_DIM)
        };

        // Add underline for active tab
        if is_active {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        let label_text = if *tab == RequestTab::Body {
            // Show validation status for body tab
            if app.validation_error.is_some() {
                format!("{} ✗", label)
            } else if has_content {
                format!("{} ✓", label)
            } else {
                label.to_string()
            }
        } else if has_content {
            format!("{} •", label)
        } else {
            label.to_string()
        };

        spans.push(Span::styled(label_text, style));
    }

    // Right-aligned: response tab toggle
    // Calculate remaining space for right-alignment
    let used: usize = spans.iter().map(|s| s.width()).sum();
    let remaining = (area.width as usize).saturating_sub(used + 20);
    if remaining > 0 {
        spans.push(Span::raw(" ".repeat(remaining)));
    }

    // Response/History toggle
    let resp_active = app.active_response_tab == ResponseTab::Response;
    spans.push(Span::styled(
        "Response",
        if resp_active {
            Style::default().fg(FG_PRIMARY).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(FG_DIM)
        },
    ));
    spans.push(Span::styled(" │ ", Style::default().fg(BORDER_COLOR)));
    spans.push(Span::styled(
        "History",
        if !resp_active {
            Style::default().fg(FG_PRIMARY).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(FG_DIM)
        },
    ));

    let tab_line = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(BG_SURFACE));
    f.render_widget(tab_line, area);
}

fn render_tab_content(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::RequestTabs;
    let border_style = if is_focused {
        Style::default().fg(ACTIVE_BORDER)
    } else {
        Style::default().fg(BORDER_COLOR)
    };

    match app.active_request_tab {
        RequestTab::Params | RequestTab::Headers | RequestTab::Authorization => {
            let title = match app.active_request_tab {
                RequestTab::Params => "Query Params",
                RequestTab::Headers => "Headers",
                RequestTab::Authorization => "Authorization",
                _ => unreachable!(),
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(Span::styled(title, Style::default().fg(FG_PRIMARY)));

            let inner = block.inner(area);
            f.render_widget(block, area);

            let active_entries = match app.active_request_tab {
                RequestTab::Headers => &app.headers,
                RequestTab::Params => &app.params,
                RequestTab::Authorization => &app.authorization,
                _ => unreachable!(),
            };

            let is_editing = app.input_mode == InputMode::Editing && is_focused;

            let kv_widget = KeyValueWidget::new(active_entries)
                .focused(is_focused)
                .editing(is_editing);

            kv_widget.render(f, inner);
        }
        RequestTab::Body => {
            // Body tab — render the TextArea
            let has_error = app.validation_error.is_some();
            let body_border = if has_error {
                Style::default().fg(ACCENT_RED)
            } else {
                border_style
            };

            let body_title = app.get_validation_status();
            let body_block = Block::default()
                .borders(Borders::ALL)
                .border_style(body_border)
                .title(Span::styled(body_title, Style::default().fg(FG_PRIMARY)));

            let mut body_textarea = app.body_input.clone();
            body_textarea.set_block(body_block);

            // Style the cursor based on editing mode
            if app.input_mode == InputMode::Editing && is_focused {
                body_textarea.set_cursor_style(
                    Style::default().bg(ACCENT_BLUE).fg(Color::White),
                );
            }

            f.render_widget(&body_textarea, area);
        }
    }
}

fn render_response_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();
    spans.push(Span::styled(" ", Style::default()));

    if let Some(status) = app.response_status {
        let clr = status_color(status);
        spans.push(Span::styled("● ", Style::default().fg(clr)));
        spans.push(Span::styled(
            format!("{}", status),
            Style::default().fg(clr).add_modifier(Modifier::BOLD),
        ));

        let label = match status {
            200 => " OK",
            201 => " Created",
            204 => " No Content",
            301 => " Moved",
            302 => " Found",
            304 => " Not Modified",
            400 => " Bad Request",
            401 => " Unauthorized",
            403 => " Forbidden",
            404 => " Not Found",
            500 => " Internal Server Error",
            502 => " Bad Gateway",
            503 => " Service Unavailable",
            _ => "",
        };
        spans.push(Span::styled(label, Style::default().fg(clr)));
    }

    if let Some(time) = app.response_time_ms {
        spans.push(Span::styled("  │  ", Style::default().fg(BORDER_COLOR)));
        let time_str = if time > 1000 {
            format!("{:.1}s", time as f64 / 1000.0)
        } else {
            format!("{}ms", time)
        };
        spans.push(Span::styled(time_str, Style::default().fg(FG_DIM)));
    }

    if let Some(size) = app.response_size {
        spans.push(Span::styled("  │  ", Style::default().fg(BORDER_COLOR)));
        let size_str = if size > 1024 * 1024 {
            format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
        } else if size > 1024 {
            format!("{:.1}KB", size as f64 / 1024.0)
        } else {
            format!("{}B", size)
        };
        spans.push(Span::styled(size_str, Style::default().fg(FG_DIM)));
    }

    let bar = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(BG_SURFACE));
    f.render_widget(bar, area);
}

fn render_response_body(f: &mut Frame, app: &App, area: Rect) {
    match app.active_response_tab {
        ResponseTab::Response => {
            let is_focused = app.focused_pane == FocusedPane::Response;
            let border_style = if is_focused {
                Style::default().fg(ACTIVE_BORDER)
            } else {
                Style::default().fg(BORDER_COLOR)
            };

            let response_block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(Span::styled("Response", Style::default().fg(FG_PRIMARY)));

            let content = app.response_text.as_deref().unwrap_or("No response yet. Send a request with Enter.");

            // Syntax highlight JSON responses
            let highlighted_content = if content != "No response yet. Send a request with Enter."
                && content != "Loading..."
            {
                let highlighter = Highlighter::new();
                let lines = highlighter.highlight_json(content);
                Text::from(lines)
            } else {
                Text::styled(content, Style::default().fg(FG_DIM))
            };

            let response_p = Paragraph::new(highlighted_content)
                .block(response_block)
                .wrap(Wrap { trim: false })
                .scroll((app.response_scroll, 0));

            f.render_widget(response_p, area);
        }
        ResponseTab::History => {
            render_history_panel(f, app, area);
        }
    }
}

fn render_history_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Response;
    let border_style = if is_focused {
        Style::default().fg(ACTIVE_BORDER)
    } else {
        Style::default().fg(BORDER_COLOR)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            format!("History ({})", app.history.len()),
            Style::default().fg(FG_PRIMARY),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.history.is_empty() {
        let empty_msg = Paragraph::new(Line::from(Span::styled(
            "  No requests in history. Send a request to see it here.",
            Style::default().fg(FG_DIM),
        )));
        f.render_widget(empty_msg, inner);
        return;
    }

    // Render history entries (most recent first)
    let max_visible = inner.height as usize;
    let mut lines = Vec::new();

    for (idx, entry) in app.history.iter().enumerate().rev() {
        if lines.len() >= max_visible {
            break;
        }

        let is_selected = app.history_index == Some(idx);
        let method_clr = method_color(&entry.method);

        let mut spans = Vec::new();

        // Selection indicator
        if is_selected {
            spans.push(Span::styled(" ▸ ", Style::default().fg(ACCENT_BLUE)));
        } else {
            spans.push(Span::raw("   "));
        }

        // Method badge
        spans.push(Span::styled(
            format!("{:<6}", entry.method),
            Style::default().fg(method_clr).add_modifier(Modifier::BOLD),
        ));

        spans.push(Span::raw(" "));

        // URL (truncated)
        let max_url_len = (inner.width as usize).saturating_sub(30);
        let url_display = if entry.url.len() > max_url_len {
            format!("{}...", &entry.url[..max_url_len.saturating_sub(3)])
        } else {
            entry.url.clone()
        };
        spans.push(Span::styled(
            url_display,
            if is_selected {
                Style::default().fg(FG_PRIMARY)
            } else {
                Style::default().fg(FG_DIM)
            },
        ));

        // Status badge (if response was received)
        if let Some(status) = entry.response_status {
            spans.push(Span::styled(
                format!("  {}", status),
                Style::default().fg(status_color(status)),
            ));
        }

        // Timestamp
        spans.push(Span::styled(
            format!("  {}", entry.formatted_time()),
            Style::default().fg(Color::Rgb(80, 80, 95)),
        ));

        let mut line = Line::from(spans);
        if is_selected {
            line = line.style(Style::default().bg(Color::Rgb(40, 40, 55)));
        }

        lines.push(line);
    }

    let history_p = Paragraph::new(lines).scroll((app.history_scroll as u16, 0));
    f.render_widget(history_p, inner);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    match app.input_mode {
        InputMode::Normal => {
            let bindings = vec![
                ("Tab", "Navigate"),
                ("←→", "Cycle"),
                ("i", "Edit"),
                ("Enter", "Send"),
                ("h", "History"),
                ("q", "Quit"),
            ];

            for (i, (key, action)) in bindings.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled("  ", Style::default()));
                }
                spans.push(Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(60, 60, 75))
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    format!(" {}", action),
                    Style::default().fg(FG_DIM),
                ));
            }
        }
        InputMode::Editing => {
            spans.push(Span::styled(
                " Esc ",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(60, 60, 75))
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                " Stop Editing",
                Style::default().fg(FG_DIM),
            ));

            if app.active_request_tab != RequestTab::Body {
                spans.push(Span::styled("  ", Style::default()));
                spans.push(Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(60, 60, 75))
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    " Next Field",
                    Style::default().fg(FG_DIM),
                ));
                spans.push(Span::styled("  ", Style::default()));
                spans.push(Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(60, 60, 75))
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    " Next Row",
                    Style::default().fg(FG_DIM),
                ));
                spans.push(Span::styled("  ", Style::default()));
                spans.push(Span::styled(
                    " Ctrl+D ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(60, 60, 75))
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    " Delete Row",
                    Style::default().fg(FG_DIM),
                ));
            }
        }
    }

    let footer = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(BG_DARK));
    f.render_widget(footer, area);
}
