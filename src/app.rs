use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tui_textarea::TextArea;

use crate::collection::{Collection, Environment, SavedRequest};
use crate::key_value::KeyValueEntries;
use crate::text_input::TextInput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestHistoryEntry {
    pub method: HttpMethod,
    pub url: String,
    pub headers: KeyValueEntries,
    pub params: KeyValueEntries,
    pub auth: KeyValueEntries,
    pub body: String,
    pub timestamp: u64,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub response_time_ms: Option<u128>,
    pub response_size: Option<usize>,
}

impl RequestHistoryEntry {
    pub fn new(
        method: HttpMethod,
        url: String,
        headers: KeyValueEntries,
        params: KeyValueEntries,
        auth: KeyValueEntries,
        body: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            method,
            url,
            headers,
            params,
            auth,
            body,
            timestamp,
            response_status: None,
            response_body: None,
            response_time_ms: None,
            response_size: None,
        }
    }

    /// Format the timestamp as a human-readable relative or absolute string
    pub fn formatted_time(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let diff = now.saturating_sub(self.timestamp);
        if diff < 60 {
            "just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
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
    RequestTabs,
    Response,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestTab {
    Params,
    Headers,
    Authorization,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseTab {
    Response,
    History,
}

#[derive(Debug, Clone)]
pub struct App {
    pub running: bool,
    pub input_mode: InputMode,
    pub focused_pane: FocusedPane,

    // Request Data
    pub method: HttpMethod,
    pub url_input: TextInput,
    pub active_request_tab: RequestTab,
    pub headers: KeyValueEntries,
    pub params: KeyValueEntries,
    pub authorization: KeyValueEntries,
    pub body_input: TextArea<'static>,

    // Response Data
    pub response_text: Option<String>,
    pub response_status: Option<u16>,
    pub response_scroll: u16,
    pub response_time_ms: Option<u128>,
    pub response_size: Option<usize>,

    // Response/History toggle
    pub active_response_tab: ResponseTab,

    // Request History
    pub history: Vec<RequestHistoryEntry>,
    pub history_index: Option<usize>,
    pub history_scroll: usize,
    pub history_search: TextInput,

    // JSON Validation
    pub validation_error: Option<(usize, usize, String)>,

    // Collections
    pub collections: Vec<Collection>,
    pub active_collection: Option<usize>,
    pub active_saved_request: Option<usize>,
    pub active_environment: Option<usize>,
    pub show_sidebar: bool,
    pub sidebar_scroll: usize,

    // Text input for dialogs (naming collections, requests, etc.)
    pub dialog_input: TextInput,
    pub dialog_active: DialogMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogMode {
    None,
    NewCollection,
    SaveRequest,
    NewEnvironment,
    AddVariable,
}

impl App {
    pub fn new() -> App {
        App {
            running: true,
            input_mode: InputMode::Normal,
            focused_pane: FocusedPane::Url,
            method: HttpMethod::GET,
            url_input: TextInput::new(),
            active_request_tab: RequestTab::Params,
            headers: KeyValueEntries::new(),
            params: KeyValueEntries::new(),
            authorization: KeyValueEntries::new(),
            body_input: TextArea::default(),
            response_text: None,
            response_status: None,
            response_scroll: 0,
            response_time_ms: None,
            response_size: None,
            active_response_tab: ResponseTab::Response,
            history: Vec::new(),
            history_index: None,
            history_scroll: 0,
            history_search: TextInput::new(),
            validation_error: None,
            collections: Vec::new(),
            active_collection: None,
            active_saved_request: None,
            active_environment: None,
            show_sidebar: false,
            sidebar_scroll: 0,
            dialog_input: TextInput::new(),
            dialog_active: DialogMode::None,
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

    /// Cycle to next request tab
    pub fn next_request_tab(&mut self) {
        self.active_request_tab = match self.active_request_tab {
            RequestTab::Params => RequestTab::Authorization,
            RequestTab::Authorization => RequestTab::Headers,
            RequestTab::Headers => RequestTab::Body,
            RequestTab::Body => RequestTab::Params,
        };
    }

    /// Cycle to previous request tab
    pub fn prev_request_tab(&mut self) {
        self.active_request_tab = match self.active_request_tab {
            RequestTab::Params => RequestTab::Body,
            RequestTab::Headers => RequestTab::Params,
            RequestTab::Authorization => RequestTab::Headers,
            RequestTab::Body => RequestTab::Authorization,
        };
    }

    /// Toggle response tab between Response and History
    pub fn toggle_response_tab(&mut self) {
        self.active_response_tab = match self.active_response_tab {
            ResponseTab::Response => ResponseTab::History,
            ResponseTab::History => ResponseTab::Response,
        };
    }

    /// Get mutable reference to the active tab's key-value entries
    pub fn get_active_tab_mut(&mut self) -> Option<&mut KeyValueEntries> {
        match self.active_request_tab {
            RequestTab::Headers => Some(&mut self.headers),
            RequestTab::Params => Some(&mut self.params),
            RequestTab::Authorization => Some(&mut self.authorization),
            RequestTab::Body => None, // Body uses TextArea, not KeyValueEntries
        }
    }

    /// Check if the current request tab is a key-value tab (not body)
    pub fn is_kv_tab(&self) -> bool {
        self.active_request_tab != RequestTab::Body
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
                    "Body ✓".to_string()
                }
            }
            Some((line, col, _)) => {
                format!("Body (Error L{}:C{})", line, col)
            }
        }
    }

    /// Check if a tab has content (for dot indicators)
    pub fn tab_has_content(&self, tab: &RequestTab) -> bool {
        match tab {
            RequestTab::Params => !self.params.entries.is_empty(),
            RequestTab::Headers => !self.headers.entries.is_empty(),
            RequestTab::Authorization => !self.authorization.entries.is_empty(),
            RequestTab::Body => !self.get_body_text().trim().is_empty(),
        }
    }

    /// Format response metadata for status bar
    pub fn response_status_text(&self) -> String {
        let mut parts = Vec::new();
        if let Some(status) = self.response_status {
            let status_label = match status {
                200..=299 => format!("{} OK", status),
                300..=399 => format!("{} Redirect", status),
                400..=499 => format!("{} Client Error", status),
                500..=599 => format!("{} Server Error", status),
                _ => format!("{}", status),
            };
            parts.push(status_label);
        }
        if let Some(time) = self.response_time_ms {
            if time > 1000 {
                parts.push(format!("{:.1}s", time as f64 / 1000.0));
            } else {
                parts.push(format!("{}ms", time));
            }
        }
        if let Some(size) = self.response_size {
            if size > 1024 * 1024 {
                parts.push(format!("{:.1}MB", size as f64 / (1024.0 * 1024.0)));
            } else if size > 1024 {
                parts.push(format!("{:.1}KB", size as f64 / 1024.0));
            } else {
                parts.push(format!("{}B", size));
            }
        }
        if parts.is_empty() {
            String::new()
        } else {
            parts.join(" │ ")
        }
    }

    /// Save current request to history
    pub fn save_to_history(&mut self) {
        let entry = RequestHistoryEntry::new(
            self.method.clone(),
            self.url_input.text.clone(),
            self.headers.clone(),
            self.params.clone(),
            self.authorization.clone(),
            self.get_body_text(),
        );
        self.history.push(entry);
        self.history_index = None;
    }

    /// Update the last history entry with response data
    pub fn update_last_history_response(
        &mut self,
        status: u16,
        body: &str,
        time_ms: u128,
        size: usize,
    ) {
        if let Some(entry) = self.history.last_mut() {
            entry.response_status = Some(status);
            entry.response_body = Some(body.to_string());
            entry.response_time_ms = Some(time_ms);
            entry.response_size = Some(size);
        }
    }

    /// Load a specific history entry by index
    pub fn load_from_history(&mut self, index: usize) {
        if let Some(entry) = self.history.get(index).cloned() {
            self.method = entry.method;
            self.url_input = TextInput::from(entry.url);
            self.headers = entry.headers;
            self.params = entry.params;
            self.authorization = entry.auth;
            self.set_body_text(&entry.body);
            self.history_index = Some(index);

            // Also load response if available
            if let Some(status) = entry.response_status {
                self.response_status = Some(status);
            }
            if let Some(body) = entry.response_body {
                self.response_text = Some(body);
            }
            self.response_time_ms = entry.response_time_ms;
            self.response_size = entry.response_size;
        }
    }

    /// Get filtered history based on search query, returning original indices
    pub fn get_filtered_history(&self) -> Vec<(usize, &RequestHistoryEntry)> {
        let query = self.history_search.text.to_lowercase();
        self.history
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                if query.is_empty() {
                    return true;
                }
                entry.url.to_lowercase().contains(&query)
                    || entry.method.to_string().to_lowercase().contains(&query)
                    || entry.response_status.map(|s| s.to_string() == query).unwrap_or(false)
            })
            .collect()
    }

    /// Navigate to previous history entry (older)
    pub fn prev_history(&mut self) {
        let filtered = self.get_filtered_history();
        if filtered.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => filtered.last().map(|(i, _)| *i).unwrap_or(0), // Start from most recent
            Some(current_idx) => {
                if let Some(pos) = filtered.iter().position(|(i, _)| *i == current_idx) {
                    if pos == 0 {
                        filtered[0].0
                    } else {
                        filtered[pos - 1].0
                    }
                } else {
                    filtered.last().map(|(i, _)| *i).unwrap_or(0)
                }
            }
        };

        self.load_from_history(new_index);
    }

    /// Navigate to next history entry (newer)
    pub fn next_history(&mut self) {
        let filtered = self.get_filtered_history();
        if filtered.is_empty() {
            return;
        }

        match self.history_index {
            None => {} // Not browsing history
            Some(current_idx) => {
                if let Some(pos) = filtered.iter().position(|(i, _)| *i == current_idx) {
                    if pos >= filtered.len() - 1 {
                        // At newest entry, clear history browsing
                        self.history_index = None;
                    } else {
                        self.load_from_history(filtered[pos + 1].0);
                    }
                }
            }
        }
    }

    // ─── Collection Methods ───────────────────────────────

    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
    }

    /// Create a new collection with the given name
    pub fn create_collection(&mut self, name: String) {
        let collection = Collection::new(name);
        self.collections.push(collection);
        self.active_collection = Some(self.collections.len() - 1);
    }

    /// Delete the active collection
    pub fn delete_active_collection(&mut self) {
        if let Some(idx) = self.active_collection {
            if idx < self.collections.len() {
                self.collections.remove(idx);
                if self.collections.is_empty() {
                    self.active_collection = None;
                } else if idx >= self.collections.len() {
                    self.active_collection = Some(self.collections.len() - 1);
                }
                self.active_saved_request = None;
                self.active_environment = None;
            }
        }
    }

    /// Save current request to the active collection
    pub fn save_request_to_collection(&mut self, name: String) {
        if let Some(col_idx) = self.active_collection {
            let request = SavedRequest::new(
                name,
                self.method.clone(),
                self.url_input.text.clone(),
                self.headers.clone(),
                self.params.clone(),
                self.authorization.clone(),
                self.get_body_text(),
            );
            if let Some(col) = self.collections.get_mut(col_idx) {
                col.add_request(request);
                self.active_saved_request = Some(col.requests.len() - 1);
            }
        }
    }

    /// Load a saved request from the active collection
    pub fn load_saved_request(&mut self, req_index: usize) {
        let req = self
            .active_collection
            .and_then(|col_idx| self.collections.get(col_idx))
            .and_then(|col| col.requests.get(req_index))
            .cloned();

        if let Some(req) = req {
            self.method = req.method;
            self.url_input = TextInput::from(req.url.clone());
            self.headers = req.headers;
            self.params = req.params;
            self.authorization = req.auth;
            self.set_body_text(&req.body);
            self.active_saved_request = Some(req_index);
        }
    }

    /// Delete a saved request from the active collection
    pub fn delete_saved_request(&mut self, req_index: usize) {
        if let Some(col_idx) = self.active_collection {
            if let Some(col) = self.collections.get_mut(col_idx) {
                col.remove_request(req_index);
                if col.requests.is_empty() {
                    self.active_saved_request = None;
                } else if let Some(active) = self.active_saved_request {
                    if active >= col.requests.len() {
                        self.active_saved_request = Some(col.requests.len() - 1);
                    }
                }
            }
        }
    }

    /// Create a new environment in the active collection
    pub fn create_environment(&mut self, name: String) {
        if let Some(col_idx) = self.active_collection {
            let env = Environment::new(name);
            if let Some(col) = self.collections.get_mut(col_idx) {
                col.add_environment(env);
            }
        }
    }

    /// Set the active environment for the active collection
    pub fn set_active_environment(&mut self, env_index: usize) {
        if let Some(col_idx) = self.active_collection {
            if let Some(col) = self.collections.get_mut(col_idx) {
                col.set_active_environment(env_index);
                self.active_environment = Some(env_index);
            }
        }
    }

    /// Get the active environment (if any)
    pub fn get_active_environment(&self) -> Option<&Environment> {
        self.active_collection
            .and_then(|col_idx| self.collections.get(col_idx))
            .and_then(|col| col.active_environment())
    }

    /// Add a variable to the active environment
    pub fn add_variable_to_active_env(&mut self, key: String, value: String) {
        if let Some(col_idx) = self.active_collection {
            if let Some(col) = self.collections.get_mut(col_idx) {
                if let Some(env_idx) = self.active_environment {
                    if let Some(env) = col.environments.get_mut(env_idx) {
                        env.add_variable(key, value);
                    }
                }
            }
        }
    }
}
