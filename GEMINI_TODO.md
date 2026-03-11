# Glance Enhancements: Top 5 Ideas

1. **Process Monitor ("Top-lite")**
   * **What:** Expand the System panel to show the top 5 CPU and Memory consuming processes in real-time.
   * **Why:** Power users seeing a CPU/RAM spike in the sparklines immediately want to know *what* is causing it. This prevents them from having to exit the dashboard or open another terminal pane to run `htop` or `btm`.

2. **Local Agenda / Todo Integration**
   * **What:** A new panel that reads standard local files (like a `todo.txt`, local `.ics` calendar files, or a Taskwarrior database) to show today's schedule and tasks.
   * **Why:** Completes the "morning briefing" philosophy. Keeping it restricted to local files maintains the "zero API key / zero complex OAuth" ethos of the project.

3. **Mini Port & Network Connection Monitor**
   * **What:** An expansion of the Network stats to show top active network connections or a list of listening ports (e.g., `localhost:8080`).
   * **Why:** Developers constantly lose track of what services are running on which ports. A quick HUD showing active local dev servers is invaluable for the target audience.

4. **"Quick-Action" Command Palette**
   * **What:** Press `:` to bring up a mini command palette at the bottom of the screen, allowing users to run custom local scripts, restart services (`brew services restart postgres`), or force-refresh data.
   * **Why:** Bridges the gap to interactive system management without cluttering the UI with buttons or straying too far into `wtfutil` territory.

5. **TUI Article Preview (Readability Mode)**
   * **What:** Instead of just popping the browser open when pressing `Enter` on a news headline, open a modal window within the TUI that fetches and strips the article to plain text (similar to Firefox's Reader View).
   * **Why:** Keeps the user in the terminal longer and maintains the distraction-free "focus" vibe of the dashboard.
