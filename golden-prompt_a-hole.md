# Golden Prompt — a-hole / Config Content Manager

## For Your Coding Agent

---

## The One Sentence

Build a Pi-hole for developer attention that mirrors telemetry back to the user, tracks their own config diffs as earned knowledge, and acts as a Content Manager for their terminal stack — paving desire paths instead of pulling up grass.

---

## The Philosophy (Don't Skip This)

Every tool the agent builds must internalize these three principles or it will drift into being another Nix, another chezmoi, another thing that demands you declare everything upfront before it will help you:

**1. Observe first, declare never.**
The tool watches what the user actually touches. It never asks them to describe their system. It learns the system by watching their feet — like a city that seeds the grass and then paves where the students walked.

**2. The diff is the product.**
Not the file. Not the full config. The *change* — scoped, named, reversible. A mod in the Nexus Mods sense. Two mods touching the same key is a conflict to surface, not a reason to refuse.

**3. Mirror, don't block.**
Telemetry already exists and already flows away from the user toward vendor product roadmaps. This tool intercepts a copy of that stream and keeps it locally — not to block it, not to fight it, just to give the user a seat at the table with their own data. The pipe already exists. Point a copy of it the other direction.

---

## What To Build

### Core: The Observer Layer

A lightweight background process — runs on almost nothing, like Pi-hole on a Raspberry Pi — that:

- Watches the config files the user actually touches (inotify / FSEvents)
- Logs every diff with timestamp, tool context, and outcome
- Detects reversions ("changed and changed back within N minutes = signal")
- Detects patterns ("touched this file on every new machine setup = earned knowledge")
- Never touches a file it hasn't been invited into by the user's own behavior

**Key files to watch for the user's specific stack:**
- `~/.config/wezterm/wezterm.lua`
- `~/.config/zellij/` (`.kdl` files)
- `~/.config/nushell/` (`config.nu`, `env.nu`)
- `~/.config/zed/settings.json`
- `~/.dotfiles/` or wherever stow is pointed
- Shell history (read-only, pattern analysis only)

**Output:** A local SQLite log. Not a cloud. Not a service. A file on disk the user owns completely.

---

### Layer 2: The Knowledge Mirror

Reads the observer log and builds a personal knowledge record:

- **Preferences** — settings the user always lands on after experimentation
- **Earned knowledge** — configs they recreate on every machine
- **Danger zones** — changes that historically caused breakage
- **Dead weight** — installed tools and configs that never get touched

This is the telemetry mirror. The same data vendors collect and use to build features — kept locally, readable by the user, queryable in Nushell because everything is structured data.

```nu
a-hole log | where outcome == "reverted" | sort-by timestamp | last 10
a-hole knowledge | where confidence > 0.8 | to md
a-hole patterns | group-by tool | sort-by touch_count
```

---

### Layer 3: The Content Manager Surface

The Nexus Mods / Assetto Corsa Content Manager layer. Takes the knowledge record and makes it actionable:

- **Browse** community config mods by tool, by workflow, by stack similarity
- **Preview** as a diff before applying — never surprise the user
- **Apply** as a named, scoped, reversible delta — not a file replacement
- **Conflict resolution** — two mods touching the same key surfaces the conflict, user decides, choice is logged
- **Revert** any mod with one command, no manual cleanup

A mod is:
```toml
[mod]
name = "wezterm-minimal-tabbar"
tool = "wezterm"
author = "whoever"
safe_keys = ["tab_bar_at_bottom", "hide_tab_bar_if_only_one_tab"]
delta = { tab_bar_at_bottom = true, hide_tab_bar_if_only_one_tab = true }
```

The platform scans submissions and flags any mod that declares keys outside its stated scope. Same as browser extension permission review.

---

### Layer 4: The Sharing Layer (Optional, Build Last)

A mod is just a named diff with a manifest. Sharing is just putting that file somewhere others can find it. The platform:

- Indexes mods by tool + OS + shell
- Shows endorsements and download counts (the Nexus social layer)
- Has a "works with" tag system (e.g. `wezterm + nushell + zellij`)
- Has an "advanced users only / may break things" flag
- Never hosts secrets, never hosts full config files, only deltas

No ownership claimed over any tool. The tool authors don't know this exists. That's the point.

---

## Tech Stack Constraints

The agent should make opinionated choices that match the user's existing stack:

- **Language:** Rust — fits the existing Nustage codebase, fits the terminal-native philosophy, fits the "runs on almost nothing" requirement
- **Config parsing:** Tool-specific adapters — Lua parser for WezTerm, KDL parser for Zellij, TOML/JSON for everything else. No universal format imposed.
- **Storage:** SQLite for the local log. Flat TOML files for mods. No database server. No cloud dependency.
- **Query interface:** Nushell — the observer log should be queryable as structured data natively. This is also the first real-world test of the Nustage/Nushell integration vision.
- **TUI:** Ratatui — consistent with Nustage. Keyboard driven. Lightweight.
- **File watching:** `notify` crate (cross-platform inotify/FSEvents wrapper for Rust)
- **Diff format:** Similar to git patches but scoped to individual keys, not lines. A semantic diff, not a text diff.

---

## Relationship to Nustage

This is not Nustage. But it shares DNA and should be built to eventually share infrastructure:

- The observer log is structured tabular data — Nustage's exact domain
- The Nushell query interface is the first real test of the embed-nu integration
- The mod delta format is spiritually identical to Nustage's `TransformationStep` — a named, scoped, reversible change
- Long term: Nustage manages data pipelines, At-hole manages config pipelines, same step model underneath

Build them as siblings, not as one thing.

---

## What NOT To Build

The agent must resist these temptations or the project becomes everything it's trying not to be:

- **Do not** build a declarative config system. The user declares nothing upfront.
- **Do not** build a dotfile syncer. That's a different (solved) problem.
- **Do not** build a cloud service. The mirror stays local. Always.
- **Do not** abstract away the underlying config formats. WezTerm's Lua stays Lua. Nushell's config stays Nushell. The adapter knows the format, the user still owns it.
- **Do not** require the user to describe their stack to get started. Watch first. Ask never.
- **Do not** pull up the grass. Pave where it gets worn out.

---

## The MVP (What To Actually Ship First)

Don't build all four layers. Build this:

**A Nushell-queryable config diff log for the user's specific stack.**

```
a-hole init        # watches the 5-6 files that matter for this stack
a-hole log         # shows every config change as structured data
a-hole revert <id> # rolls back any logged change
a-hole export      # dumps knowledge record as .md for human review
```

That's it. That's the Pi-hole moment — the thing that runs quietly, proves the concept, and makes you notice how much you needed it the moment it's there.

Everything else — the community layer, the mod marketplace, the sharing platform — comes after the user has lived with their own mirror for a while and started to see the patterns.

---

## Relevant Context & References

- **Nustage repo:** `github.com/CromboJambo/nustage` — sibling project, shared philosophy
- **embed-nu:** `github.com/nustage/embed-nu` — Nushell embedded runtime, target query interface
- **notify crate:** `crates.io/crates/notify` — filesystem watching in Rust
- **Content Manager for AC:** The UX reference. Meets the chaos, doesn't replace it, no ownership claimed.
- **Pi-hole:** The deployment reference. Runs on nothing. Invisible until you need it. Works for you not against you.
- **Nexus Mods:** The community reference. Mod author owns the mod. Platform owns the index. Game owns nothing.
- **Desire path / pave the dirt path:** The philosophical north star. Don't design the correct path. Watch where people walk, then pave it.
- **KDL:** `kdl.dev` — Zellij's config format, worth understanding for the adapter layer
- **Nushell structured data:** The query interface for the log. `a-hole log | where tool == "wezterm" | sort-by timestamp`

---

## The Name

**A-hole** — like Pi-hole, but for attention. Sits quietly in your environment. Filters the noise. Logs what matters. Works for you. Runs forever on almost nothing.

Or something better if the agent has ideas. The concept is more important than the name.

---

*Built from a conversation about Alacritty, WezTerm, Nushell, Assetto Corsa mods, Pi-hole, desire paths, and the honest observation that modding is harder than coding and normal people do it for free and mostly don't want recognition.*
