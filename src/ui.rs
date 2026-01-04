use crate::app::{App, FocusedPane, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
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

    // Headers / Body Area
    let details_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(details_area);

    // Headers
    let headers_block = Block::default()
        .borders(Borders::ALL)
        .title("Headers (Key:Value)")
        .style(if app.focused_pane == FocusedPane::Headers {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let headers_p = Paragraph::new(app.headers_input.as_str()).block(headers_block);
    f.render_widget(headers_p, details_chunks[0]);

    // Body
    let body_block = Block::default()
        .borders(Borders::ALL)
        .title("Body (JSON)")
        .style(if app.focused_pane == FocusedPane::Body {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let body_p = Paragraph::new(app.body_input.as_str()).block(body_block);
    f.render_widget(body_p, details_chunks[1]);

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
    // For MVP, just wrap text. Later we can add scrolling state.
    let response_p = Paragraph::new(content)
        .block(response_block)
        .wrap(Wrap { trim: false });
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
