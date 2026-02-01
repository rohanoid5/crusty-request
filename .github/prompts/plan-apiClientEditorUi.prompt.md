## Plan: Enhanced CLI API Client with Editor UI ✅ COMPLETE

A Rust TUI API client using ratatui with a clean modular architecture. The MVP supports GET/POST/PUT/DELETE/PATCH methods with panels for URL, headers, body, and response. This plan adds a proper text editor for the body, request history, syntax highlighting, and real-time JSON validation.

### Steps

1. ✅ **Add dependencies** in [Cargo.toml](Cargo.toml):

   - `tui-textarea = "0.7"` for multi-line editor with cursor movement
   - `syntect = { version = "5.3", default-features = false, features = ["default-fancy"] }` for JSON syntax highlighting

2. ✅ **Create `RequestHistoryEntry` struct and history storage** in [app.rs](src/app.rs):

   - Add `RequestHistoryEntry { method, url, headers, body, timestamp }` struct
   - Add `history: Vec<RequestHistoryEntry>` and `history_index: Option<usize>` to `App`
   - Add methods: `save_to_history()`, `load_from_history(index)`, `next_history()`, `prev_history()`

3. ✅ **Replace `body_input: String` with `TextArea`** in [app.rs](src/app.rs):

   - Change `body_input` field to `tui_textarea::TextArea<'static>`
   - Initialize with `TextArea::default()` in `App::new()`
   - Add `validation_error: Option<(usize, usize, String)>` for JSON error tracking

4. ✅ **Add syntax highlighting module** - create new file `src/highlight.rs`:

   - Load `SyntaxSet` and `ThemeSet` from syntect defaults
   - Implement `highlight_json(text: &str) -> Vec<Line>` to convert JSON to styled ratatui Lines
   - Implement `convert_syntect_style()` to map syntect styles to ratatui `Style`

5. ✅ **Add real-time JSON validation** in [app.rs](src/app.rs):

   - Add `validate_body()` method that parses body with `serde_json::from_str`
   - On error, extract `e.line()`, `e.column()`, `e.to_string()` for error position
   - Store in `validation_error` field, call on every body text change
   - Update textarea block title to show error location or "Valid JSON ✓"

6. ✅ **Update body panel rendering** in [ui.rs](src/ui.rs):

   - Render `TextArea::widget()` instead of `Paragraph` for body pane
   - Apply red border style when `validation_error.is_some()`
   - Show error message in block title: `"Body (Error at line 5, col 12)"`

7. ✅ **Add highlighted response rendering** in [ui.rs](src/ui.rs):

   - Use `highlight_json()` for response body when content-type is JSON
   - Render with `Paragraph::new(highlighted_lines)` instead of plain text

8. ✅ **Refactor input handling** in [main.rs](src/main.rs):

   - When `focused_pane == Body` and `input_mode == Editing`:
     - Route `KeyEvent` to `app.body_textarea.input(event)` for standard keybindings
     - Call `app.validate_body()` after each input
   - Keep existing character-by-character handling for URL/Headers panes

9. ✅ **Add history navigation keybindings** in [main.rs](src/main.rs):

   - `Ctrl+P` / `Up` (in Normal mode on URL pane): Load previous history entry
   - `Ctrl+N` / `Down` (in Normal mode on URL pane): Load next history entry
   - Auto-save to history before sending each request

10. ✅ **Add response scrolling** in [ui.rs](src/ui.rs) and [app.rs](src/app.rs):
    - Add `response_scroll: u16` to `App` struct
    - Handle `Up`/`Down` keys when `focused_pane == Response`
    - Apply `.scroll((app.response_scroll, 0))` to response Paragraph

### Decisions Made

- **Standard keybindings** for tui-textarea (arrows, Home/End, Ctrl+A/E) - no vim mode
- **syntect with `default-fancy`** feature for pure Rust (no C dependencies)
- **Real-time JSON validation** included in first iteration with error line/column display
- **Request history** stored in memory (file persistence can be added later)

### File Changes Summary

| File                       | Changes                                                  |
| -------------------------- | -------------------------------------------------------- |
| [Cargo.toml](Cargo.toml)   | Add `tui-textarea`, `syntect`                            |
| [src/app.rs](src/app.rs)   | Add `TextArea`, `RequestHistoryEntry`, validation fields |
| [src/ui.rs](src/ui.rs)     | TextArea rendering, syntax highlighting, error styling   |
| [src/main.rs](src/main.rs) | Route events to TextArea, history navigation             |
| `src/highlight.rs`         | New file for syntect integration                         |
