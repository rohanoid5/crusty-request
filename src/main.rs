mod app;
mod collection;
mod highlight;
mod key_value;
mod network;
mod storage;
mod text_input;
mod ui;
mod variables;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::error::Error;
use std::time::Instant;
use std::{io, time::Duration};
use tokio::sync::mpsc;

use crate::app::{App, DialogMode, FocusedPane, InputMode, RequestTab};
use crate::network::{make_request, ApiResponse};
use crate::ui::ui;

/// Extended response with timing info
struct TimedResponse {
    result: Result<ApiResponse, String>,
    elapsed_ms: u128,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App and Channel
    let mut app = App::new();
    // Load persisted data
    app.history = storage::load_history();
    app.collections = storage::load_collections();
    let (tx, mut rx) = mpsc::channel::<TimedResponse>(10);

    // Run the main loop
    let res = run_app(&mut terminal, &mut app, tx, &mut rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tx: mpsc::Sender<TimedResponse>,
    rx: &mut mpsc::Receiver<TimedResponse>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // 1. Poll for User Input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Dialog mode takes priority
                if app.dialog_active != DialogMode::None {
                    handle_dialog_mode(app, key);
                    continue;
                }

                // Global Quit
                if app.input_mode == InputMode::Normal && key.code == KeyCode::Char('q') {
                    app.quit();
                }

                if app.input_mode == InputMode::Normal {
                    handle_normal_mode(app, key, &tx);
                } else if app.input_mode == InputMode::Editing {
                    handle_editing_mode(app, key);
                }
            }
        }

        // 2. Poll for Network Responses
        if let Ok(timed) = rx.try_recv() {
            match timed.result {
                Ok(resp) => {
                    let body_size = resp.body.len();
                    app.response_status = Some(resp.status);
                    app.response_text = Some(resp.body.clone());
                    app.response_time_ms = Some(timed.elapsed_ms);
                    app.response_size = Some(body_size);

                    // Update last history entry with response data
                    app.update_last_history_response(
                        resp.status,
                        &resp.body,
                        timed.elapsed_ms,
                        body_size,
                    );

                    // Persist history to disk
                    let _ = storage::save_history(&app.history);
                }
                Err(err_msg) => {
                    app.response_status = None;
                    app.response_text = Some(format!("Error: {}", err_msg));
                    app.response_time_ms = Some(timed.elapsed_ms);
                    app.response_size = None;
                }
            }
        }

        if !app.running {
            return Ok(());
        }
    }
}

fn handle_normal_mode(
    app: &mut App,
    key: crossterm::event::KeyEvent,
    tx: &mpsc::Sender<TimedResponse>,
) {
    match key.code {
        // Navigation
        KeyCode::Tab => {
            app.focused_pane = match app.focused_pane {
                FocusedPane::Method => FocusedPane::Url,
                FocusedPane::Url => FocusedPane::RequestTabs,
                FocusedPane::RequestTabs => FocusedPane::Response,
                FocusedPane::Response => FocusedPane::Method,
            };
        }
        KeyCode::BackTab => {
            app.focused_pane = match app.focused_pane {
                FocusedPane::Method => FocusedPane::Response,
                FocusedPane::Url => FocusedPane::Method,
                FocusedPane::RequestTabs => FocusedPane::Url,
                FocusedPane::Response => FocusedPane::RequestTabs,
            };
        }

        // Enter edit mode
        KeyCode::Char('i') => {
            app.input_mode = InputMode::Editing;
        }

        // Send request
        KeyCode::Enter => {
            // Save to history before sending
            app.save_to_history();

            let sender = tx.clone();
            let method = app.method.clone();
            let mut url = app.url_input.text.clone();
            let mut headers = app.headers.clone();
            let mut params = app.params.clone();
            let mut auth = app.authorization.clone();
            let mut body = app.get_body_text();

            // Apply variable interpolation if there's an active environment
            if let Some(env) = app.get_active_environment() {
                let (i_url, i_headers, i_params, i_auth, i_body) =
                    variables::interpolate_all(&url, &headers, &params, &auth, &body, env);
                url = i_url;
                headers = i_headers;
                params = i_params;
                auth = i_auth;
                body = i_body;
            }

            app.response_text = Some("Loading...".to_string());
            app.response_status = None;
            app.response_time_ms = None;
            app.response_size = None;

            tokio::spawn(async move {
                let start = Instant::now();
                let result = match make_request(method, url, &headers, &params, &auth, body).await {
                    Ok(resp) => Ok(resp),
                    Err(e) => Err(e.to_string()),
                };
                let elapsed_ms = start.elapsed().as_millis();
                let _ = sender
                    .send(TimedResponse {
                        result,
                        elapsed_ms,
                    })
                    .await;
            });
        }

        // Method cycling
        KeyCode::Right | KeyCode::Char(' ') => {
            if app.focused_pane == FocusedPane::Method {
                app.next_method();
            } else if app.focused_pane == FocusedPane::RequestTabs {
                app.next_request_tab();
            }
        }
        KeyCode::Left => {
            if app.focused_pane == FocusedPane::Method {
                app.prev_method();
            } else if app.focused_pane == FocusedPane::RequestTabs {
                app.prev_request_tab();
            }
        }

        // Toggle Response/History
        KeyCode::Char('h') => {
            app.toggle_response_tab();
        }

        // Toggle sidebar (Ctrl+B)
        KeyCode::Char('b')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            app.toggle_sidebar();
        }

        // New collection (Ctrl+N)
        KeyCode::Char('n')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            app.dialog_active = DialogMode::NewCollection;
            app.dialog_input.clear();
        }

        // Save to collection (Ctrl+S)
        KeyCode::Char('s')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            if app.active_collection.is_some() {
                app.dialog_active = DialogMode::SaveRequest;
                app.dialog_input.clear();
            }
        }

        // Vertical navigation
        KeyCode::Up => {
            if app.focused_pane == FocusedPane::Response {
                if app.active_response_tab == crate::app::ResponseTab::Response {
                    app.response_scroll = app.response_scroll.saturating_sub(1);
                } else {
                    // History navigation
                    app.prev_history();
                }
            } else if app.focused_pane == FocusedPane::RequestTabs && app.is_kv_tab() {
                if let Some(entries) = app.get_active_tab_mut() {
                    if entries.focused_index > 0 {
                        entries.focused_index -= 1;
                    }
                }
            }
        }
        KeyCode::Down => {
            if app.focused_pane == FocusedPane::Response {
                if app.active_response_tab == crate::app::ResponseTab::Response {
                    app.response_scroll = app.response_scroll.saturating_add(1);
                } else {
                    app.next_history();
                }
            } else if app.focused_pane == FocusedPane::RequestTabs && app.is_kv_tab() {
                if let Some(entries) = app.get_active_tab_mut() {
                    if entries.focused_index <= entries.entries.len() {
                        entries.focused_index += 1;
                    }
                }
            }
        }

        _ => {}
    }
}

fn handle_editing_mode(app: &mut App, key: crossterm::event::KeyEvent) {
    // Body tab — route all keys to TextArea
    if app.active_request_tab == RequestTab::Body && app.focused_pane == FocusedPane::RequestTabs {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                app.body_input.insert_newline();
                let (row, _) = app.body_input.cursor();
                if row > 0 {
                    let lines = app.body_input.lines();
                    if let Some(prev_line) = lines.get(row - 1) {
                        let mut indent: String = prev_line.chars().take_while(|c| c.is_whitespace()).collect();
                        let trimmed = prev_line.trim_end();
                        if trimmed.ends_with('{') || trimmed.ends_with('[') {
                            indent.push_str("  "); // 2 space indent
                        }
                        app.body_input.insert_str(indent);
                    }
                }
                app.validate_body();
            }
            _ => {
                app.body_input.input(key);
                app.validate_body();
            }
        }
        return;
    }

    // Key-value tab editing
    if app.focused_pane == FocusedPane::RequestTabs && app.is_kv_tab() {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                // Cycle through Key -> Value -> Description fields
                if let Some(entries) = app.get_active_tab_mut() {
                    entries.next_field();
                }
            }
            KeyCode::Enter => {
                // Move to next row, create new if at end
                if let Some(entries) = app.get_active_tab_mut() {
                    if entries.focused_index >= entries.entries.len() {
                        entries.add_entry(String::new(), String::new());
                    }
                    entries.focused_index += 1;
                    if entries.focused_index > entries.entries.len() {
                        entries.focused_index = entries.entries.len();
                    }
                    entries.focused_field = crate::key_value::KeyValueField::Key;
                }
            }
            KeyCode::Delete | KeyCode::Char('d')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                if let Some(entries) = app.get_active_tab_mut() {
                    let idx = entries.focused_index;
                    if idx < entries.entries.len() {
                        entries.remove_entry(idx);
                        if entries.focused_index >= entries.entries.len()
                            && entries.focused_index > 0
                        {
                            entries.focused_index -= 1;
                        }
                    }
                }
            }
            KeyCode::Char('e')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Toggle enabled/disabled
                if let Some(entries) = app.get_active_tab_mut() {
                    let idx = entries.focused_index;
                    entries.toggle_enabled(idx);
                }
            }
            KeyCode::Left => {
                if let Some(entries) = app.get_active_tab_mut() {
                    let focused_field = entries.focused_field.clone();
                    if let Some(entry) = entries.get_selected_mut(entries.focused_index) {
                        match focused_field {
                            crate::key_value::KeyValueField::Key => entry.key.move_left(),
                            crate::key_value::KeyValueField::Value => entry.value.move_left(),
                            crate::key_value::KeyValueField::Description => entry.description.move_left(),
                        }
                    }
                }
            }
            KeyCode::Right => {
                if let Some(entries) = app.get_active_tab_mut() {
                    let focused_field = entries.focused_field.clone();
                    if let Some(entry) = entries.get_selected_mut(entries.focused_index) {
                        match focused_field {
                            crate::key_value::KeyValueField::Key => entry.key.move_right(),
                            crate::key_value::KeyValueField::Value => entry.value.move_right(),
                            crate::key_value::KeyValueField::Description => entry.description.move_right(),
                        }
                    }
                }
            }
            KeyCode::Char(c) => {
                if let Some(entries) = app.get_active_tab_mut() {
                    let focused_field = entries.focused_field.clone();
                    let focused_index = entries.focused_index;

                    if let Some(entry) = entries.get_selected_mut(focused_index) {
                        match focused_field {
                            crate::key_value::KeyValueField::Key => entry.key.insert_char(c),
                            crate::key_value::KeyValueField::Value => entry.value.insert_char(c),
                            crate::key_value::KeyValueField::Description => {
                                entry.description.insert_char(c)
                            }
                        }
                    } else if focused_index >= entries.entries.len() {
                        entries.add_entry(String::new(), String::new());
                        if let Some(entry) = entries.get_selected_mut(focused_index) {
                            match focused_field {
                                crate::key_value::KeyValueField::Key => entry.key.insert_char(c),
                                crate::key_value::KeyValueField::Value => entry.value.insert_char(c),
                                crate::key_value::KeyValueField::Description => {
                                    entry.description.insert_char(c)
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(entries) = app.get_active_tab_mut() {
                    let focused_field = entries.focused_field.clone();
                    if let Some(entry) = entries.get_selected_mut(entries.focused_index) {
                        match focused_field {
                            crate::key_value::KeyValueField::Key => entry.key.delete_back(),
                            crate::key_value::KeyValueField::Value => entry.value.delete_back(),
                            crate::key_value::KeyValueField::Description => entry.description.delete_back(),
                        }
                    }
                }
            }
            KeyCode::Delete => {
                if let Some(entries) = app.get_active_tab_mut() {
                    let focused_field = entries.focused_field.clone();
                    if let Some(entry) = entries.get_selected_mut(entries.focused_index) {
                        match focused_field {
                            crate::key_value::KeyValueField::Key => entry.key.delete_forward(),
                            crate::key_value::KeyValueField::Value => entry.value.delete_forward(),
                            crate::key_value::KeyValueField::Description => entry.description.delete_forward(),
                        }
                    }
                }
            }
            _ => {}
        }
        return;
    }

    // URL editing
    if app.focused_pane == FocusedPane::Url {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Left => {
                app.url_input.move_left();
            }
            KeyCode::Right => {
                app.url_input.move_right();
            }
            KeyCode::Char(c) => {
                app.url_input.insert_char(c);
            }
            KeyCode::Backspace => {
                app.url_input.delete_back();
            }
            KeyCode::Delete => {
                app.url_input.delete_forward();
            }
            _ => {}
        }
        return;
    }

    // History search editing
    if app.focused_pane == FocusedPane::Response && app.active_response_tab == crate::app::ResponseTab::History {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Left => app.history_search.move_left(),
            KeyCode::Right => app.history_search.move_right(),
            KeyCode::Char(c) => app.history_search.insert_char(c),
            KeyCode::Backspace => app.history_search.delete_back(),
            KeyCode::Delete => app.history_search.delete_forward(),
            _ => {}
        }
        return;
    }

    // Any other pane — just handle Esc
    if key.code == KeyCode::Esc {
        app.input_mode = InputMode::Normal;
    }
}

fn handle_dialog_mode(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.dialog_active = DialogMode::None;
            app.dialog_input.clear();
        }
        KeyCode::Enter => {
            let input = app.dialog_input.text.clone();
            if !input.is_empty() {
                match app.dialog_active {
                    DialogMode::NewCollection => {
                        app.create_collection(input);
                        let _ = storage::save_collections(&app.collections);
                    }
                    DialogMode::SaveRequest => {
                        app.save_request_to_collection(input);
                        let _ = storage::save_collections(&app.collections);
                    }
                    DialogMode::NewEnvironment => {
                        app.create_environment(input);
                        let _ = storage::save_collections(&app.collections);
                    }
                    DialogMode::AddVariable => {
                        // Parse "key=value" format
                        if let Some((key, value)) = input.split_once('=') {
                            app.add_variable_to_active_env(
                                key.trim().to_string(),
                                value.trim().to_string(),
                            );
                            let _ = storage::save_collections(&app.collections);
                        }
                    }
                    DialogMode::None => {}
                }
            }
            app.dialog_active = DialogMode::None;
            app.dialog_input.clear();
        }
        KeyCode::Left => {
            app.dialog_input.move_left();
        }
        KeyCode::Right => {
            app.dialog_input.move_right();
        }
        KeyCode::Char(c) => {
            app.dialog_input.insert_char(c);
        }
        KeyCode::Backspace => {
            app.dialog_input.delete_back();
        }
        KeyCode::Delete => {
            app.dialog_input.delete_forward();
        }
        _ => {}
    }
}
