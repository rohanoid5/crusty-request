mod app;
mod network;
mod ui;

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
use std::{io, time::Duration};
use tokio::sync::mpsc;

use crate::app::{App, FocusedPane, InputMode};
use crate::network::{make_request, ApiResponse};
use crate::ui::ui;

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
    let (tx, mut rx) = mpsc::channel::<Result<ApiResponse, String>>(10);

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
    tx: mpsc::Sender<Result<ApiResponse, String>>,
    rx: &mut mpsc::Receiver<Result<ApiResponse, String>>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // 1. Poll for User Input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Global Quit
                if app.input_mode == InputMode::Normal && key.code == KeyCode::Char('q') {
                    app.quit();
                }

                if app.input_mode == InputMode::Normal {
                    match key.code {
                        KeyCode::Tab => {
                            app.focused_pane = match app.focused_pane {
                                FocusedPane::Method => FocusedPane::Url,
                                FocusedPane::Url => FocusedPane::Headers,
                                FocusedPane::Headers => FocusedPane::Body,
                                FocusedPane::Body => FocusedPane::Response,
                                FocusedPane::Response => FocusedPane::Method,
                            };
                        }
                        KeyCode::Char('i') => {
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Enter => {
                            // Trigger Request!
                            let sender = tx.clone();
                            let method = app.method.clone();
                            let url = app.url_input.clone();
                            let headers = app.headers_input.clone();
                            let body = app.body_input.clone();

                            app.response_text = Some("Loading...".to_string());

                            tokio::spawn(async move {
                                match make_request(method, url, headers, body).await {
                                    Ok(resp) => {
                                        let _ = sender.send(Ok(resp)).await;
                                    }
                                    Err(e) => {
                                        let _ = sender.send(Err(e.to_string())).await;
                                    }
                                }
                            });
                        }
                        // Handle Method Cycling
                        KeyCode::Right | KeyCode::Char(' ') => {
                            if app.focused_pane == FocusedPane::Method {
                                app.next_method();
                            }
                        }
                        KeyCode::Left => {
                            if app.focused_pane == FocusedPane::Method {
                                app.prev_method();
                            }
                        }
                        _ => {}
                    }
                } else if app.input_mode == InputMode::Editing {
                    match key.code {
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => match app.focused_pane {
                            FocusedPane::Url => app.url_input.push(c),
                            FocusedPane::Headers => app.headers_input.push(c),
                            FocusedPane::Body => app.body_input.push(c),
                            _ => {}
                        },
                        KeyCode::Backspace => match app.focused_pane {
                            FocusedPane::Url => {
                                app.url_input.pop();
                            }
                            FocusedPane::Headers => {
                                app.headers_input.pop();
                            }
                            FocusedPane::Body => {
                                app.body_input.pop();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }

        // 2. Poll for Network Responses
        if let Ok(response) = rx.try_recv() {
            match response {
                Ok(resp) => {
                    app.response_status = Some(resp.status);
                    app.response_text = Some(resp.body); // Headers? We can add a tab for that later
                }
                Err(err_msg) => {
                    app.response_status = None;
                    app.response_text = Some(format!("Error: {}", err_msg));
                }
            }
        }

        if !app.running {
            return Ok(());
        }
    }
}
