# Implementation Plan: Process Monitor ("Top-lite")

**Feature:** Integrate a real-time Top Processes view into the existing System panel, identifying exactly what is driving the CPU and RAM sparklines.

**Target Demographic:** The Power User / Developer. When they see the CPU sparkline spike to 100%, they immediately need to know if it's `rust-analyzer`, a Docker container, or a runaway Node script.

## 1. Backend: Data Collection (`src/system.rs`)
Since `sysinfo` is already heavily utilized in the project and specified in `Cargo.toml`, we can leverage its `refresh_processes()` capability with a very minimal footprint.

*   **Update `SystemState` Struct:** Add a `top_processes` field to store the parsed data.
    ```rust
    pub struct ProcessInfo {
        pub pid: u32,
        pub name: String,
        pub cpu_usage: f32,
        pub mem_mb: f64,
    }
    // Add to SystemState: pub top_processes: Vec<ProcessInfo>,
    ```
*   **Modify `update()` logic:**
    *   Call `sys.refresh_processes()`. *Note: For CPU usage to be accurate, this needs to be called iteratively, which fits perfectly into your existing async tick loop.*
    *   Iterate through `sys.processes()`, sort them by CPU usage (primary) and Memory (secondary).
    *   Extract the top 5 to 7 processes, map them into the `ProcessInfo` struct, and store them in the state.
*   **Performance Consideration:** Process polling can be slightly heavier than basic CPU/RAM polling. We should gate `refresh_processes()` to run every 2 seconds instead of every single tick, using an elapsed time check.

## 2. Frontend: UI Rendering (`src/ui.rs` & `src/layout.rs`)
We want to keep the UI clean and avoid cluttering the visual hierarchy. We can adapt the layout depending on the user's preference or screen size.

*   **Widget Choice:** Use Ratatui's `Table` widget.
    *   Columns: `PID` (right-aligned, dim), `Name` (truncated), `CPU%` (styled), `MEM` (styled).
*   **Integration into the System Panel:**
    *   **Option A (Split View):** Split the existing System block vertically. The top 60% shows the current gauges and sparklines; the bottom 40% shows the Top Processes table. This is ideal for the `Tall` and `Wide` layout presets.
    *   **Option B (Interactive Toggle):** Add a subtle footer to the System block: `[p] Processes`. When the user presses `p`, the sparklines/gauges are hidden and the process list takes over that block. This is ideal for the `Compact` layout.
    *   *Recommendation:* Implement both, using Option A when vertical constraints permit, and Option B as a fallback.

## 3. Configuration & Controls (`src/app.rs` & `src/config.rs`)
*   **App State:** Add `show_processes: bool` to the main application state (for the toggle approach).
*   **Event Handling:** Map the `p` key in the main event loop (likely `app.rs` or `main.rs`) to toggle this boolean.
*   **Config:** Add a `show_processes_by_default = true` boolean to the `config.toml` parser so users can dictate their preferred startup view.

## 4. Execution Steps
1.  **Data Fetching:** Add the `ProcessInfo` struct and sorting logic to `system.rs`. Log the output to verify CPU % calculations match OS expectations (e.g., deciding whether to divide by core count, as `sysinfo` returns 0-100% per core by default).
2.  **UI Layout:** Adjust the layout constraints in `ui.rs` to allocate space for the new `Table` widget.
3.  **Styling:** Apply the current Theme colors (`theme.rs`) to the table headers and create a threshold formatter (green -> yellow -> red) for high CPU/RAM values.
4.  **Testing:** Run a heavy workload in another terminal (e.g., `cargo build`) and verify the dashboard accurately reflects the `rustc` processes bubbling to the top of the list in real-time.

---

# Implementation Plan: TUI Article Preview (Readability Mode)

**Feature:** When a user presses `Enter` on a news headline, open a modal window within the TUI that displays the plain-text article, stripped of ads and tracking scripts, formatted in clean Markdown.

**Target Demographic:** The focused terminal user who wants to read an article without being context-switched out of their workflow into a visually noisy web browser.

## 1. Backend: Markdown Extraction (`src/news.rs` or new `article.rs`)
To maintain the project's "zero-API-key" ethos and avoid dragging heavy local HTML parsers/DOM-tree dependencies into the binary, we will leverage the free Jina Reader API (`https://r.jina.ai/`). By prefixing any URL with `https://r.jina.ai/`, it returns a beautifully formatted, ad-free Markdown version of the page content.
*   **Reqwest Fetch:** Create an async function `fetch_article_markdown(url: &str)` that calls the Jina endpoint and returns the raw text string.
*   **State Update:** Create a new `AppState` variant to handle the lifecycle:
    *   `AppState::LoadingArticle`
    *   `AppState::ReadingArticle { title: String, content: String, scroll: u16 }`

## 2. Input Handling (`src/main.rs` & `src/app.rs`)
We need to adjust how the News panel responds to keys.
*   **Enter (Return):** If the News panel is focused, pressing `Enter` kicks off the async fetch and shifts the app state to `LoadingArticle`.
*   **'o' Key:** Re-map opening the article in the *external browser* to the `o` key, preserving the original functionality for users who want to see images or interact with the site.
*   **Modal Navigation:** When in `AppState::ReadingArticle`, hijack the event loop so that:
    *   `j` / `Down` arrow: Increment `scroll` by 1.
    *   `k` / `Up` arrow: Decrement `scroll` by 1.
    *   `Esc` / `q`: Return to `AppState::Running`.

## 3. Frontend: Modal Rendering (`src/ui.rs`)
Ratatui has an excellent `Clear` widget that allows us to draw floating modal windows over existing UI elements.
*   **Draw Logic:** At the end of `ui::render()`, check if `app.state` is `ReadingArticle` (or `LoadingArticle`). If so, execute a dedicated `render_article_modal()` function.
*   **Modal Layout:**
    *   Use `Layout::default()` to create a centered rect (e.g., 80% width, 80% height of the screen).
    *   Render a `Clear` widget over this rect to erase the background.
    *   Render a `Block` with `Borders::ALL` and a stylized title (the article headline).
*   **Content Display:**
    *   Use the Ratatui `Paragraph` widget.
    *   Enable `.wrap(Wrap { trim: false })` to keep the markdown readable.
    *   Apply `.scroll((app.article_scroll, 0))` to allow vertical navigation through the document.

## 4. Execution Steps
1.  **State Management:** Expand `AppState` enum and add the `article_scroll` tracking logic to `app.rs`.
2.  **Key Bindings:** Update the `main.rs` event loop to handle `Enter` vs `o` in the News panel, and add the modal escape/scroll bindings.
3.  **Data Fetcher:** Write the async `fetch_article` helper using `reqwest` and hook it into the main loop via a `tokio::spawn` or similar async channel pattern (similar to how weather/news feeds are fetched).
4.  **UI Construction:** Build the `Clear` + `Paragraph` modal in `ui.rs`. Test it with dummy text to dial in the borders and wrap behavior, then hook it up to the real fetched markdown.
