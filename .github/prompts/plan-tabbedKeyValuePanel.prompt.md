## Plan: Tabbed Headers/Params/Authorization Panel with Key-Value Editor

Replace the current single-line Headers panel with a tabbed interface supporting Headers, Params, and Authorization sections, each with a two-column key-value editor.

### Steps

1. **Create `KeyValueEntry` and `KeyValueList` structs** in [src/app.rs](src/app.rs):

   - Add `KeyValueEntry { key: String, value: String, enabled: bool }` struct
   - Add `KeyValueList { entries: Vec<KeyValueEntry>, selected_index: usize, focused_field: KeyValueField }` where `KeyValueField` is `Key` or `Value`
   - Add methods: `add_entry()`, `remove_entry()`, `get_selected_mut()`, `to_header_string()`

2. **Create `RequestTab` enum and update `App` state** in [src/app.rs](src/app.rs):

   - Add `RequestTab { Headers, Params, Authorization }` enum
   - Replace `headers_input: String` with `headers: KeyValueList`, `params: KeyValueList`, `auth: KeyValueList`
   - Add `active_request_tab: RequestTab` field to track current tab
   - Add methods: `next_tab()`, `prev_tab()`, `get_active_tab_mut()`

3. **Update `FocusedPane` for granular focus** in [src/app.rs](src/app.rs):

   - Change `FocusedPane::Headers` to `FocusedPane::RequestDetails` (covers the tabbed panel)
   - Add helper `is_in_request_details()` method
   - Tab cycling within RequestDetails pane handled separately

4. **Create `key_value.rs` module** for reusable key-value widget:

   - Create `KeyValueWidget` that renders two-column layout (Key | Value)
   - Style active row with highlight background
   - Style active field (Key or Value) with cursor indicator
   - Show enabled/disabled checkbox per row

5. **Update tabbed panel rendering** in [src/ui.rs](src/ui.rs):

   - Add horizontal tab bar at top: `[Headers] [Params] [Auth]` with active tab highlighted
   - Split remaining space for key-value list using `KeyValueWidget`
   - Layout: `Length(1)` for tab bar, `Min(0)` for content

6. **Add key-value list rendering** in [src/ui.rs](src/ui.rs):

   - Two-column table layout: `Constraint::Percentage(50)` for each
   - Render column headers: "Key" | "Value"
   - Render each `KeyValueEntry` as a row with selection highlight
   - Show empty row placeholder for adding new entries

7. **Update input handling for tab switching** in [src/main.rs](src/main.rs):
   - When `focused_pane == RequestDetails` and `input_mode == Normal`:
     - `Left`/`Right` or `Shift+Tab`/`Tab`: Cycle between Headers/Params/Auth tabs
     - `Up`/`Down`: Navigate between key-value rows
     - `Enter` or `i`: Enter editing mode on selected field
8. **Add key-value field editing** in [src/main.rs](src/main.rs):

   - When `focused_pane == RequestDetails` and `input_mode == Editing`:
     - Character input goes to current field (key or value)
     - `Tab`: Switch between Key and Value fields in same row
     - `Enter`: Move to next row (create new if at end)
     - `Ctrl+D` or `Delete`: Remove current row
     - `Esc`: Exit editing mode

9. **Update `make_request` to use new data structures** in [src/network.rs](src/network.rs):

   - Accept `KeyValueList` for headers instead of `String`
   - Build query params from `params: KeyValueList`
   - Apply authorization from `auth: KeyValueList` (Basic auth: username/password keys)

10. **Update history serialization** in [src/app.rs](src/app.rs):
    - Change `RequestHistoryEntry.headers: String` to `headers: Vec<(String, String)>`
    - Add `params` and `auth` fields
    - Update `save_to_history()` and `load_from_history()` methods

### Decisions Made

- **Authorization types**: Start with simple key-value (supports Bearer token, API key). Dropdown for Basic/Bearer/API Key presets can be added later.
- **Enable/disable checkbox**: Include toggle per row (Space key) to temporarily disable entries without deleting
- **TextArea for values**: Keep single-line for MVP. Multi-line value support with `tui-textarea` can be added later.

### File Changes Summary

| File                             | Changes                                                                |
| -------------------------------- | ---------------------------------------------------------------------- |
| [src/app.rs](src/app.rs)         | Add `KeyValueEntry`, `KeyValueList`, `RequestTab`, update `App` struct |
| [src/ui.rs](src/ui.rs)           | Tab bar rendering, key-value table layout                              |
| [src/main.rs](src/main.rs)       | Tab switching, row navigation, field editing                           |
| [src/network.rs](src/network.rs) | Accept new data structures for headers/params/auth                     |
| `src/key_value.rs`               | New module for reusable key-value widget                               |
