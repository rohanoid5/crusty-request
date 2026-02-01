use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tui_textarea::TextArea;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestHistoryEntry {
    pub method: HttpMethod,
    pub url: String,
    pub headers: String,
    pub body: String,
    pub timestamp: u64,
}

impl RequestHistoryEntry {
    pub fn new(method: HttpMethod, url: String, headers: String, body: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            method,
            url,
            headers,
            body,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPane {
    Method,
    Url,
    Headers,
    Body,
    Response, // Maybe for scrolling
}

#[derive(Debug, Clone)]
pub struct App {
    pub running: bool,
    pub input_mode: InputMode,
    pub focused_pane: FocusedPane,

    // Request Data
    pub method: HttpMethod,
    pub url_input: String,
    pub headers_input: String,
    pub body_input: TextArea<'static>,

    // Response Data (Placeholder for now)
    pub response_text: Option<String>,
    pub response_status: Option<u16>,
    pub response_scroll: u16,

    // Request History
    pub history: Vec<RequestHistoryEntry>,
    pub history_index: Option<usize>,

    // JSON Validation
    pub validation_error: Option<(usize, usize, String)>, // (line, column, message)
}

impl App {
    pub fn new() -> App {
        App {
            running: true,
            input_mode: InputMode::Normal,
            focused_pane: FocusedPane::Url,
            method: HttpMethod::GET,
            url_input: String::new(),
            headers_input: String::new(),
            body_input: TextArea::default(),
            response_text: None,
            response_status: None,
            response_scroll: 0,
            history: Vec::new(),
            history_index: None,
            validation_error: None,
        }
    }

    pub fn next_method(&mut self) {
        self.method = match self.method {
            HttpMethod::GET => HttpMethod::POST,
            HttpMethod::POST => HttpMethod::PUT,
            HttpMethod::PUT => HttpMethod::DELETE,
            HttpMethod::DELETE => HttpMethod::PATCH,
            HttpMethod::PATCH => HttpMethod::GET,
        };
    }

    pub fn prev_method(&mut self) {
        self.method = match self.method {
            HttpMethod::GET => HttpMethod::PATCH,
            HttpMethod::POST => HttpMethod::GET,
            HttpMethod::PUT => HttpMethod::POST,
            HttpMethod::DELETE => HttpMethod::PUT,
            HttpMethod::PATCH => HttpMethod::DELETE,
        };
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Get body text from TextArea
    pub fn get_body_text(&self) -> String {
        self.body_input.lines().join("\n")
    }

    /// Set body text in TextArea
    pub fn set_body_text(&mut self, text: &str) {
        self.body_input = TextArea::new(text.lines().map(String::from).collect());
    }

    /// Validate the body as JSON and update validation_error field
    pub fn validate_body(&mut self) {
        let body_text = self.get_body_text();

        // Empty body is considered valid (no JSON to validate)
        if body_text.trim().is_empty() {
            self.validation_error = None;
            return;
        }

        match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(_) => {
                self.validation_error = None;
            }
            Err(e) => {
                let line = e.line();
                let column = e.column();
                let message = e.to_string();
                self.validation_error = Some((line, column, message));
            }
        }
    }

    /// Get a formatted validation status message for display
    pub fn get_validation_status(&self) -> String {
        match &self.validation_error {
            None => {
                if self.get_body_text().trim().is_empty() {
                    "Body (JSON)".to_string()
                } else {
                    "Body (JSON) ✓".to_string()
                }
            }
            Some((line, col, _)) => {
                format!("Body (Error at line {}, col {})", line, col)
            }
        }
    }

    /// Save current request to history
    pub fn save_to_history(&mut self) {
        let entry = RequestHistoryEntry::new(
            self.method.clone(),
            self.url_input.clone(),
            self.headers_input.clone(),
            self.get_body_text(),
        );
        self.history.push(entry);
        self.history_index = None; // Reset index after saving
    }

    /// Load a specific history entry by index
    pub fn load_from_history(&mut self, index: usize) {
        if let Some(entry) = self.history.get(index).cloned() {
            self.method = entry.method;
            self.url_input = entry.url;
            self.headers_input = entry.headers;
            self.set_body_text(&entry.body);
            self.history_index = Some(index);
        }
    }

    /// Navigate to previous history entry (older)
    pub fn prev_history(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => self.history.len() - 1, // Start from most recent
            Some(0) => 0,                   // Already at oldest
            Some(idx) => idx - 1,
        };

        self.load_from_history(new_index);
    }

    /// Navigate to next history entry (newer)
    pub fn next_history(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {} // Not browsing history
            Some(idx) if idx >= self.history.len() - 1 => {
                // At newest entry, clear history browsing
                self.history_index = None;
            }
            Some(idx) => {
                self.load_from_history(idx + 1);
            }
        }
    }
}
