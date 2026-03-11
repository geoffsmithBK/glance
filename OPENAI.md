# Glance Power User Enhancements

Five opinionated enhancements for Glance aimed at power users who want a strong default experience and no setup burden.

## 1. Ambient Mode Switching

**Concept:** Glance should automatically change what it emphasizes based on what the machine is doing right now.

**Behavior:**
- During calm periods, keep the current balanced dashboard layout.
- During sustained CPU, RAM, or disk pressure, promote the System panel and expand process visibility.
- During high network activity, temporarily surface network throughput and top talkers more aggressively.
- During mornings, commutes, and evenings, bias the Weather panel slightly higher in mixed layouts.

**Why it fits Glance:**
- Power users do not want to micromanage views.
- The app should feel aware, not passive.
- The current responsive layout system already provides the right primitives.

**Zero-config stance:** Thresholds and transitions are fixed and curated. Manual layout cycling still works, but auto-behavior remains the default.

## 2. Operator Digest

**Concept:** Add a one-line, always-on summary at the top or bottom that tells the user what actually matters right now.

**Examples:**
- `CPU pressure from rustc and clang. Network quiet. Rain expected after 6 PM.`
- `Memory climbing for Docker Desktop. 3 new headlines from Hacker News. Weather stable.`
- `Disk nearly full on /. TX spike detected. Cloudy morning, clear afternoon.`

**Why it fits Glance:**
- Raw metrics are useful, but power users value interpretation.
- It makes the app feel more editorial and intentional.
- It reinforces the "glanceable" promise instead of requiring scanning across all panels.

**Zero-config stance:** The digest is generated from a fixed ruleset. No custom templates, no user-authored summaries, no severity tuning.

## 3. Focus Scenes

**Concept:** Replace generic panel toggles with a few hardcoded, named operating modes that match real terminal workflows.

**Proposed scenes:**
- `Observe`: normal balanced dashboard
- `Investigate`: expands system metrics, processes, and trends
- `Read`: prioritizes headlines and article reader
- `Travel`: emphasizes weather, local time, and short forecast

**Interaction:**
- One key cycles scenes.
- Glance can also auto-enter a scene when conditions strongly suggest it.

**Why it fits Glance:**
- Power users like opinionated tools with a point of view.
- Named scenes are more memorable than ad hoc toggles.
- This keeps the UI cohesive instead of accumulating one-off switches.

**Zero-config stance:** Scenes are fixed product decisions, not user-defined dashboards.

## 4. Causal Process Lens

**Concept:** When the system panel shows stress, Glance should explain the cause instead of only showing percentages.

**Behavior:**
- Group related processes into readable buckets such as `Rust build`, `Docker`, `Browser`, `Node toolchain`, `Git`.
- Display a short cause label like `CPU dominated by Rust build` or `RAM dominated by container workload`.
- Prefer group-level attribution over noisy per-process lists unless the user drills in.

**Why it fits Glance:**
- Power users often want attribution more than raw numbers.
- It turns Glance from a monitor into an operator aid.
- It preserves the app's low-noise aesthetic better than dumping a mini-`top`.

**Zero-config stance:** Grouping rules are built in. No process classification files, aliases, or custom tags.

## 5. Deadline-Aware News

**Concept:** Make the News panel behave more like a power-user briefing than a passive RSS reader.

**Behavior:**
- Rank headlines by freshness, source quality, and distinctiveness instead of simple feed order.
- Collapse near-duplicate stories from multiple feeds into one entry.
- Apply hardcoded source labels such as `shipping`, `infra`, `security`, `ai`, `markets` when obvious.
- De-emphasize stale items automatically after a short window.

**Why it fits Glance:**
- Power users want fewer, better items.
- Opinionated curation is more valuable than feed exhaust.
- This complements the existing article reader and keeps the panel useful throughout the day.

**Zero-config stance:** Ranking and dedup rules are built into the product. Users supply feeds if they want, but the reading experience stays curated.

## Why These Work Together

- They make Glance more editorial and less mechanical.
- They reduce the amount of scanning and manual mode switching required.
- They preserve the current keyboard-first workflow.
- They add intelligence without expanding the config surface.

## Recommended First Bet

**Operator Digest** is the strongest first enhancement.

It has the best ratio of impact to implementation cost:
- It sharpens the app's identity immediately.
- It can synthesize data Glance already collects.
- It reinforces the zero-config philosophy because the product decides what matters.
- It creates a visible layer of intelligence without forcing a redesign of every panel.
