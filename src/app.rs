use serde::{Deserialize, Serialize};

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
    pub body_input: String,

    // Response Data (Placeholder for now)
    pub response_text: Option<String>,
    pub response_status: Option<u16>,
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
            body_input: String::new(),
            response_text: None,
            response_status: None,
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
}
