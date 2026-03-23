A-hole MVP Technical Spec v0.0.3**

**Purpose**
A-hole is a local config-observer for terminal-centric developers. Its first job is not to manage config, synchronize dotfiles, or infer an ideal setup. Its first job is to watch a small set of user-owned config files, record real changes as they happen, and make those changes inspectable and reversible from a truthful CLI.

This MVP is the “Pi-hole moment” for the project: a small local utility that quietly proves the concept by giving the user a reliable mirror of their own config behavior.

**Product Vision**
The long-term vision is a local telemetry mirror for developer configuration: a system that observes what users actually change, keeps that record locally, and eventually derives patterns from it. The project’s philosophy is important, but the MVP exists to validate one narrow claim:

If a user edits a watched config file, a-hole can reliably record what changed, when it changed, and where it changed.

Everything else is downstream of that.

**Core Principles**
The implementation must preserve these principles:

- Observe first, declare never.
- The change is the product, not the full config file.
- Mirror locally, do not block or replace upstream tooling.
- Tell the truth about what the software is doing.
- Prefer narrow, working behavior over broad, implied capability.

These principles are design constraints, not marketing copy.

**Target User**
The MVP is for a technically capable local-first user with a terminal-centric setup. The first supported stack is explicitly narrow: known local config files in a Linux/macOS-style home directory, with emphasis on terminal and editor config.

This is not a general enterprise telemetry system, not a cloud product, and not a workflow tool for non-technical users.

**MVP Scope**
Version 0.1 includes only the following capabilities:

- Initialize a local SQLite database for change tracking.
- Watch a small default list of config files and optionally user-specified paths.
- Detect file save/change events for watched files.
- Capture file content snapshots before and after changes when possible.
- Compute a basic diff summary for each recorded change.
- Store changes as structured local records.
- Expose recent changes through a CLI.
- Revert a recorded change when the system has enough stored data to do so safely.
- Export change history in a human-readable format.

Version 0.1 does not include inferred preferences, community mods, marketplace behavior, stack similarity, endorsements, cloud sync, background service installation, or broad semantic parsing across all config formats.

**Non-Goals**
The following are explicitly out of scope for v0.1:

- Declarative config management
- Dotfile synchronization
- Automatic config merging across machines
- Universal config abstraction
- Cross-user mod sharing
- Background service integration with `systemd`, `launchd`, or Windows services
- Shell history mining
- Community ecosystem features
- Full “knowledge mirror” inference beyond basic change logging
- Rich TUI or UI surface
- Embedded Nushell runtime as a hard dependency

If a proposed task does not directly improve observing, storing, diffing, querying, or reverting watched file changes, it should be deferred.

**Primary User Stories**
- As a user, I can run `a-hole init` and create a local store for config observations.
- As a user, I can start a watcher process that monitors a small set of known config files.
- As a user, when I save a watched file, a-hole records that a change occurred.
- As a user, I can run `a-hole log` and see recent recorded changes.
- As a user, I can inspect what file changed, when it changed, and a summary of the diff.
- As a user, I can revert a prior change if a-hole has enough information to do so safely.
- As a user, I can export my recent change history into a readable report.

**Supported Inputs for v0.1**
Default watch candidates may include:

- `~/.config/wezterm/wezterm.lua`
- `~/.config/zellij/config.kdl`
- `~/.config/nushell/config.nu`
- `~/.config/nushell/env.nu`
- `~/.config/zed/settings.json`

The implementation may ship with this default list, but the user must also be able to provide explicit paths.

For v0.1, path discovery should be simple and explicit. The software does not need to auto-discover a full stack. It only needs to support known defaults and direct path registration.

**Operational Model**
Version 0.1 runs as a foreground process.

Recommended command shape:

- `a-hole init`
- `a-hole watch`
- `a-hole log`
- `a-hole show <id>`
- `a-hole revert <id>`
- `a-hole export`

The system should not claim to run “in background” unless a real background mode exists. If background behavior is not implemented, `start` and `stop` commands should not exist in v0.1.

**Command Contract**

`a-hole init`
- Creates the SQLite database if missing.
- Creates required tables and indexes.
- Registers a default watch list unless explicit files are passed.
- Prints what was initialized.
- Does not claim watching has started.

`a-hole watch`
- Starts the foreground observer loop.
- Expands and normalizes watch paths.
- Verifies accessible watched files.
- Subscribes to file events for valid paths.
- On each eligible change event, captures a new snapshot and stores a change record.
- Exits non-zero if watcher startup fails.

`a-hole log`
- Shows recent change records from the database.
- Supports `--limit`.
- Optional filters may include `--file` and `--tool` if implemented.
- Must only display real persisted data.

`a-hole show <id>`
- Displays the full recorded details for a single change.
- Includes file path, timestamp, tool, summary, and stored diff/snapshot metadata.

`a-hole revert <id>`
- Restores prior file content only if the recorded change is safely reversible.
- Must fail with a clear message if the current file has diverged beyond the recorded state or if no prior snapshot exists.

`a-hole export`
- Exports recent changes to Markdown or JSON.
- For v0.1 this is a reporting feature, not a knowledge-inference feature.

**Truthfulness Rule**
No command may print success for work it did not actually perform. Any incomplete behavior must surface as one of:

- unsupported
- not configured
- not yet implemented
- unsafe to perform

This rule is mandatory because the product’s credibility depends on being an honest mirror.

**Data Model**
The minimum durable model should include these entities.

`watched_files`
- `id`
- `path`
- `normalized_path`
- `tool`
- `file_type`
- `status`
- `created_at`
- `updated_at`

`file_snapshots`
- `id`
- `watched_file_id`
- `content`
- `content_hash`
- `captured_at`

`snapshot content` may be stored inline in SQLite for v0.1. Optimization can come later.

`config_changes`
- `id`
- `watched_file_id`
- `previous_snapshot_id`
- `current_snapshot_id`
- `timestamp`
- `change_kind`
- `diff_format`
- `summary_json`
- `metadata_json`

`change_kind` may initially just be `updated`. More refined change types can be added later.

Optional `tools` may be inferred from the watched file registration rather than separately normalized.

**Storage Strategy**
For v0.1, store both snapshots and derived diff summaries.

Reason:
- Revert requires a trustworthy source of prior state.
- Semantic diffing is not mature yet.
- Snapshots are cheap at MVP scale.
- Raw snapshots preserve user trust even when diff adapters are imperfect.

The system should treat derived diffs as helpful metadata, not as the sole source of truth.

**Diff Strategy**
Version 0.1 should use a phased diff model.

Baseline behavior:
- Compute textual diffs or simple line-based summaries for all watched files.
- Derive summary fields such as lines added, lines removed, and whether content changed materially.

Optional enhanced behavior:
- JSON and TOML semantic summaries may be added first because they are easier to parse reliably.
- Lua, KDL, and Nushell semantic parsing should be deferred unless the team can do it cheaply and safely.

The architectural rule is:
Every watched file must be recordable even if semantic parsing is unavailable.

Valid parser states:
- observed, text-diff only
- observed, semantic summary available
- observed, semantic parse failed but raw snapshots preserved

**Revert Strategy**
Revert in v0.1 is snapshot-based, not intent-based.

When reverting a change:
- Confirm the target change has a prior snapshot.
- Confirm the current file still matches the expected post-change state, or define a safe override mode explicitly.
- Write back the prior snapshot content.
- Record the revert as a new change event.

If the file has diverged since the target change and safe restoration cannot be guaranteed, revert must fail clearly rather than guessing.

This keeps revert honest and predictable.

**Architecture**
Recommended module boundaries:

- `cli`
  - command parsing
  - user-facing formatting
  - no business logic beyond dispatch

- `watch`
  - path expansion and normalization
  - watcher lifecycle
  - debounce policy
  - event intake

- `capture`
  - file reads
  - hashing
  - snapshot creation
  - previous-state lookup

- `diff`
  - textual diffing
  - summary generation
  - optional semantic adapters

- `store`
  - SQLite connection management
  - schema creation
  - CRUD for watched files, snapshots, changes

- `domain`
  - shared types for `WatchedFile`, `Snapshot`, `ConfigChange`, `DiffSummary`

Dependency direction should be one-way:
CLI -> use cases -> store/watch/capture/diff

Adapters must not leak into CLI formatting. The DB layer must not know about watcher internals.

**Path Handling**
The watcher must normalize real paths before registration.

Requirements:
- Expand `~` to home directory.
- Resolve relative paths to absolute paths when passed explicitly.
- Handle missing files as a valid but non-watchable state.
- Record watch status for each file.
- Skip invalid paths with explicit feedback instead of crashing silently.

This is required for credibility on day one.

**Event Handling**
The watcher loop should treat file events conservatively.

Requirements:
- Debounce rapid save bursts.
- Ignore duplicate unchanged saves by comparing content hash.
- Record a change only when content actually differs.
- Persist events in order.

The system does not need to infer user intent from editor behavior in v0.1. It only needs to reliably distinguish real content changes from no-op events.

**Output Format**
CLI output should be human-readable by default and structured when requested.

Recommended:
- human-readable table/summary by default
- `--json` for logs and show output if feasible

Nushell compatibility is desirable, but embedding Nushell is not required for v0.1. Structured JSON is sufficient.

**Error Handling Policy**
Errors must be explicit and categorized. Minimum categories:

- database initialization failure
- invalid path
- path inaccessible
- watcher startup failure
- file read failure
- snapshot write failure
- diff generation failure
- revert safety failure
- missing change id

The process must fail loudly on startup issues and continue gracefully where possible on per-file issues.

**Security and Privacy**
Version 0.1 is local-only.

Rules:
- No network behavior.
- No cloud sync.
- No external telemetry.
- User data remains on local disk.
- Full file content may be stored locally because local ownership is a product assumption, but this should be documented clearly.

If later versions need selective redaction, that is future work.

**Acceptance Criteria for v0.1**
The MVP is done when all of the following are true:

- The crate builds successfully with `cargo check`.
- The test suite runs successfully with `cargo test`.
- `a-hole init` creates and verifies a working DB.
- `a-hole watch` successfully watches at least one supported file end-to-end.
- Editing a watched file creates a persisted change record.
- `a-hole log` shows that record.
- `a-hole show <id>` shows the associated metadata.
- `a-hole revert <id>` safely restores prior content when possible.
- Commands do not claim unsupported behavior.
- At least one end-to-end test validates observe -> store -> inspect -> revert.

**Suggested Milestones**

1. Buildable Core
- Implement `Database`
- Define domain types
- Make CLI truthful
- Remove fake commands
- Green compile and tests

2. Observer Vertical Slice
- Path normalization
- Foreground watcher
- Snapshot capture
- Change persistence
- `log` and `show`

3. Safe Revert
- Snapshot-based revert
- Divergence checks
- Revert recording
- Error cases

4. Basic Reporting
- Markdown/JSON export
- Diff summaries
- Light metadata improvements

Anything beyond that should be treated as post-MVP.

**Deferred Work**
These are explicitly deferred so the team does not accidentally build them into the MVP:

- semantic config parsing for all formats
- “earned knowledge” scoring
- reversion pattern detection
- danger-zone inference
- background daemon/service installation
- TUI
- mod manifest/application system
- community sharing/indexing
- stack similarity and recommendation logic

**Head Dev Guidance**
The senior dev should scope this as a narrow systems product, not a philosophy engine.

The correct bar for v0.1 is:
- trustworthy
- local
- boring in the right places
- explicit about unsupported states
- easy to extend later

The wrong bar is:
- visionary
- comprehensive
- semantic everywhere
- daemonized from day one
- marketplace-ready

The team should be rewarded for reducing ambiguity, deleting fake capability, and shipping one real loop that works.

**Definition of Success**
If a user can say, truthfully, “I changed my config, and a-hole kept a local record I can inspect and revert,” then v0.1 succeeded.

If the repo still mostly says what it wants to mean, but does not yet perform that loop reliably, then it has not succeeded.

If you want, I can also turn this into a tighter one-page internal brief and a separate engineering task breakdown for your senior dev.
