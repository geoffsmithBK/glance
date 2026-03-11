# Operator Digest Implementation Plan

## Goal

Add a single-line, non-scrolling Operator Digest to Glance that summarizes what matters right now without adding configuration surface area.

The digest should live in the existing bottom status bar and replace the current static key-hint line during normal operation. It should read like an operator status line, not a cable-news crawl.

## Product Shape

### Placement

- Use the existing bottom status bar row in [`src/ui.rs`](/Users/gsmith/work/glance/src/ui.rs).
- Keep overlays (`Help`, `LocationSearch`, article reader/loading) on their current dedicated status text.
- In normal `Running` mode, split the row into two regions:
  - Left: digest text
  - Right: compact hint suffix

### Behavior

- The digest is static between refreshes. No marquee, no continuous scrolling.
- Recompute on a low-frequency cadence or when materially relevant state changes.
- Prefer one sentence with up to three clauses.
- Content priority order:
  - System issue or operator-relevant cause
  - Weather event if notable
  - News freshness/signal if space remains

### Hint Suffix

- Preserve top-line key discoverability with a short suffix such as `?: keys`.
- Do not use literal `k: keys` unless keybindings are changed, because `k` already means scroll-up throughout the app.
- If the user wants a single-keystroke keys overlay beyond `?`, add a new binding deliberately rather than overloading `k`.

## UX Rules

### Writing Style

- Short, declarative, editorial tone.
- Prefer attribution over raw numbers.
- Avoid filler like `All systems normal` unless there is genuinely nothing meaningful to say.
- Examples:
  - `CPU pressure from rustc. Rain after 6 PM. 3 fresh headlines.`
  - `Memory climbing for Docker Desktop. Network quiet. Weather steady.`
  - `Disk pressure on /. Showers this evening. Hacker News active.`

### Truncation

- The digest should truncate cleanly to fit available width.
- Never horizontally scroll the digest by default.
- If space is tight, drop lower-priority clauses before truncating the highest-priority clause.

### State-Specific Rendering

- `Running`: digest + hint suffix
- `Help`: existing close-help message
- `LocationSearch`: existing location controls
- `LoadingArticle` / `ReadingArticle`: retain context-specific hints rather than digest

## Implementation Plan

### 1. Add a Digest Model

Create a small module, likely `src/digest.rs`, to keep summary generation out of `ui.rs`.

Suggested types:

```rust
pub struct DigestState {
    pub text: String,
}

pub struct DigestInputs<'a> {
    pub app: &'a App,
}
```

Responsibilities:
- Generate the current one-line summary from app state.
- Encapsulate prioritization rules.
- Return already-composed display text rather than UI spans.

Reasoning:
- `ui.rs` should render, not decide product language.
- This keeps the summarization logic testable.

### 2. Extend App State Ownership

Add a digest field to [`src/app.rs`](/Users/gsmith/work/glance/src/app.rs), for example:

```rust
pub digest_text: String,
pub last_digest_refresh: Instant,
```

Behavior:
- Initialize to an empty string or a conservative startup message.
- Refresh digest after:
  - `load_data()`
  - `update_metrics()` on a throttled cadence
  - location confirmation
  - process toggle if it changes the most relevant story

Reasoning:
- Avoid generating prose every frame.
- Keep the digest coherent and stable rather than flickering with every 100ms tick.

### 3. Define Fixed Heuristics

Use simple hardcoded rules. No configuration.

#### System Clause

Primary candidates:
- CPU pressure
- memory pressure
- disk pressure
- network spike

Attribution sources:
- [`src/system.rs`](/Users/gsmith/work/glance/src/system.rs) already has CPU, RAM, disk, throughput, and top processes.
- Reuse `top_processes` for causal phrasing.

Example rules:
- If CPU > 75% and top process exists: `CPU pressure from rustc`
- If memory > 85% and top process exists: `Memory climbing for Docker Desktop`
- If root or first disk > 90%: `Disk pressure on /`
- If RX/TX exceeds an opinionated threshold and CPU is calm: `Network spike detected`

Only emit one system clause, picking the highest-severity condition.

#### Weather Clause

Use weather when something is actionable or time-bound:
- precipitation-like condition today
- large temperature swing
- sunrise/sunset approaching within a fixed window
- severe-ish wording already derivable from weather code

Examples:
- `Rain after 6 PM`
- `Cloudy morning, clear afternoon`
- `Sunset at 7:12 PM`

If weather is unremarkable, emit nothing.

#### News Clause

Use news as tertiary context:
- `3 fresh headlines`
- `Hacker News active`
- `No fresh headlines` should usually be omitted

Use only if there are items and space remains.

### 4. Add Helper Methods for Digest Inputs

Add a few narrowly scoped helpers instead of putting all logic in one function.

Possible additions:
- [`src/system.rs`](/Users/gsmith/work/glance/src/system.rs)
  - `primary_process_label() -> Option<String>`
  - `highest_severity_signal() -> Option<SystemSignal>`
- [`src/weather.rs`](/Users/gsmith/work/glance/src/weather.rs)
  - helper for deriving a short actionable summary from existing weather data
- [`src/news.rs`](/Users/gsmith/work/glance/src/news.rs)
  - helper to count fresh headlines from published timestamps

Reasoning:
- The digest should consume meaningful domain signals, not raw widget text.
- This avoids duplicating threshold logic later if other features reuse it.

### 5. Render the Status Bar as Two Regions

Update [`src/ui.rs`](/Users/gsmith/work/glance/src/ui.rs) `render_status_bar()`:

- In normal mode, split `area` horizontally into:
  - flexible left chunk for digest
  - fixed right chunk for hint suffix
- Render both chunks with the existing status bar background color.
- Keep the suffix concise, for example:
  - `?: keys`
  - `?: help`

Layout behavior:
- If terminal width is too narrow, hide the suffix first.
- If width is extremely narrow, render only the highest-priority digest clause.

Reasoning:
- This preserves glanceability.
- It also preserves a path to key discovery without reverting to the current full hint sentence.

### 6. Keep Existing Help Overlay

Do not replace the current help overlay.

Plan:
- Keep `?` as the existing overlay trigger.
- Use the suffix as a reminder, not a new interaction model.
- Optionally revise overlay copy later so it aligns with the new digest-driven status bar.

Reasoning:
- The digest should summarize status, not become an instruction manual.
- Existing behavior is already coherent and lightweight.

### 7. Add Tests Around the Rules

Add unit tests near the digest module and any helper modules.

Coverage:
- CPU-heavy state yields CPU clause with process attribution
- memory-heavy state wins over news/weather when more severe
- quiet system + notable weather yields weather-led digest
- news clause is dropped first when width is constrained
- empty or unavailable data does not generate awkward filler text

Because [`src/system.rs`](/Users/gsmith/work/glance/src/system.rs) relies on live system state, keep most tests at the pure-string-builder layer using synthetic inputs.

## Suggested Rollout Order

1. Create `digest.rs` with pure rule evaluation and string composition.
2. Add digest storage and throttled refresh hooks in [`src/app.rs`](/Users/gsmith/work/glance/src/app.rs).
3. Update [`src/ui.rs`](/Users/gsmith/work/glance/src/ui.rs) status bar rendering to show left digest and right suffix.
4. Add tests for prioritization and truncation.
5. Tune clause thresholds against real usage once the first version is visible.

## Non-Goals

- No animated crawl
- No user-configurable severity thresholds
- No custom digest templates
- No LLM-generated summaries
- No expansion beyond one line in the first version

## Recommendation

Implement the first version with these constraints:

- Bottom status bar only
- `Running` mode only
- Fixed suffix of `?: keys`
- One system clause, optional one weather clause, optional one news clause
- Digest refresh no more than once per second

That version is small enough to ship quickly and strong enough to validate whether Glance should become more editorial over time.
