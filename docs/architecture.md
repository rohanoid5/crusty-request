# API Client Architecture

This document provides a high-level overview of the architecture and implementation details for the terminal-based API client.

## Tech Stack
- **Language**: Rust
- **Terminal UI**: `ratatui` for UI rendering, `crossterm` for terminal backend and event handling.
- **Async Runtime**: `tokio` to handle asynchronous execution (specifically networking).
- **Networking**: `reqwest` for executing HTTP requests.
- **Serialization**: `serde` and `serde_json` for representing logic states, parsing responses, and saving config to disk.
- **Text Editing**: `tui-textarea` for the multi-line JSON request body editor.
- **Highlighting**: `syntect` for syntax highlighting the JSON results.
- **File System**: `dirs` crate to resolve persistent storage paths.

## Application Structure

The application follows an Elm-like architecture (Model-View-Update pattern) adapted for Rust terminal apps. 

### 1. `main.rs` (Event Loop & Core Logic)
The entry point of the application. It initializes the terminal, creates the application state (`App`), and spawns the main `tokio` event loop.
- **Event Polling**: Listens for terminal events (keypresses) via `crossterm::event::poll`.
- **Keyboard Navigation**: Interprets key commands to navigate between panes, toggle input modes, or submit actions.
- **Async Handling**: Communicates with the background `reqwest` tasks via `tokio::sync::mpsc` channels so that the terminal doesn't block while waiting for a network response.

### 2. `app.rs` (State Management / Model)
Contains the central `App` struct representing the entire state of the program.
- **Navigation State**: Tracks the currently active pane (`Method`, `Url`, `RequestTabs`, `Response`) and the current input mode (`Normal` or `Editing`).
- **Data State**: Holds input values for the Request (HTTP method, URL, Headers, Params, Auth, Body) and the resulting Response (Status, Time, Body size, payload).
- **Request History**: Maintains a `Vec<RequestHistoryEntry>` of all sent requests with metadata (method, URL, headers, body, timestamp, response status/body/time/size). Exposes `save_to_history()`, `update_last_history_response()`, and `load_from_history()` methods.
- **History Search**: Holds a `history_search: TextInput` buffer. The `get_filtered_history()` method filters entries by URL substring, HTTP method, or response status code, returning a list of `(original_index, &entry)` tuples so keyboard navigation and selection remain consistent across filtered views.
- **Dialogs**: Manages the state and text buffer for modal popup interactions (e.g., adding variables, renaming collections).
- **State Logic**: Contains methods for state transitions (e.g., cycling through HTTP methods, loading historical requests).

### 3. `ui.rs` (Rendering / View)
Responsible strictly for drawing the current frame based entirely on the `App` state.
- Builds layouts utilizing `ratatui`'s `Layout` constraint solver.
- Modifies colors based on active and focused panes to provide visual feedback.
- Conditionally formats JSON response texts using `syntect` highlighters before they hit the terminal frame.

### 4. `network.rs` (HTTP Interaction)
Handles the translation of our App state into a physical HTTP request.
- Translates `KeyValueEntries` directly into `reqwest::header::HeaderMap`.
- Encodes query parameters utilizing `urlencoding`.
- Executes requests asynchronously using `reqwest::Client`.

### 5. `key_value.rs` (Custom List UI Widget)
Provides both a data structure and a rendering widget for key-value tables (Params, Headers, Auth).
- Supports rows mapping an enabled state (Checkbox) to a Key, Value, and Description.
- Provides specialized editing cycles to quickly input complex header/query pairs.

### 6. `text_input.rs` (Single-line Input Buffer)
Manages the memory of a single-line input with precise cursor tracking.
- Retains string state via a local cursor index.
- Encapsulates exact UTF-8 boundaries insertion and deletion (`Left`, `Right`, `Backspace`, `Delete`).
- Automatically computes active terminal rendering displays with an editing block indicator (`▏`).

### 7. `collection.rs` (Collections Data Model)
Structs and representations for user-defined groups of requests.
- Handles grouping logic mapping a list of configurations corresponding to `SavedRequest` objects.
- Defines `Environment` scope models holding overriding key-value maps.

### 8. `variables.rs` (Templating and Interpolation)
Handles the string interpolation system allowing variables to be injected into URL/header parameters.
- Exposes `interpolate(&str)` that dynamically parses `{{variable_name}}` strings against the actively loaded `Environment`.

### 9. `storage.rs` (Persistence layer)
Defines how application states write to and reload from disk.
- Saves request run histories to `~/.api-client/history.json` on **every successful or failed request** via `save_history()` automatically called from the response handler in `main.rs`.
- Loads history on startup in `main()` by calling `load_history()` and assigning directly to `app.history`.
- Persists collections and environments to `~/.api-client/collections.json` via `save_collections()` and `load_collections()`.
- Uses `dirs::home_dir()` to resolve cross-platform storage paths; gracefully falls back to an empty state if the file is absent or malformed.

## Common Workflows

### 1. Navigating Input fields
When hitting `Left`/`Right` or inputting characters into an active single-line input element (like the `Url` field or `KeyValue` fields), the specific keypress translates down through `handle_editing_mode` and executes mutations natively against the `TextInput` cursor logic.

### 2. Sending a Request
When hitting `Enter` in the main view:
1. `main.rs` intercepts the call, copies the current `App` input payload, and saves it to `.history()`.
2. It processes text fields against `variables::interpolate_all()` using the currently selected environment.
3. The resulting request strings are sent to `network::make_request` on a separate `tokio::spawn` asynchronous thread.
4. The event loop listens repeatedly for the `rx.try_recv()` channel. When the request resolves, the new response object replaces the loading state inside `app.response_text`.

### 3. JSON Auto Formatting
When focused inside the `tui_textarea` body block:
- Hitting `Enter` captures the previously typed line's preceding white space.
- If the line terminates in a logical indentifier (`[` or `{`), the editor mechanically appends 2 spaces to the captured indent block, maintaining a visually pleasing JSON structure without needing dedicated code servers.

### 4. Saving & Loading History
After every request send:
1. `app.save_to_history()` is called before dispatching the network task, recording method, URL, headers, params, auth, body, and timestamp.
2. When the `tokio::spawn` task resolves, `app.update_last_history_response()` patches the last entry with status, body, elapsed time, and size.
3. `storage::save_history(&app.history)` immediately serialises the full history slice to `~/.api-client/history.json`.
4. On the next launch, `storage::load_history()` deserialises the file and restores the history vector before the first frame is rendered.

### 5. Searching History
With the History tab active (toggled with `h`):
1. Press `i` to enter Editing mode — focus routes to the `history_search` input at the top of the panel.
2. As characters are typed, `app.get_filtered_history()` recomputes a filtered list matching against URL, method name, or response status.
3. `Up`/`Down` keys navigate within the **filtered** list; the underlying original index is preserved so that `load_from_history(idx)` loads the correct entry.
4. Pressing `Esc` or `Enter` exits the search input back to Normal navigation mode.
