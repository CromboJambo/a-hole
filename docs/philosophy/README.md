# Philosophy of a-hole

This directory contains the core philosophical principles that guide the design and development of a-hole — a Config Mirror and Content Manager for your terminal stack.

## Core Principles

### 1. **Diffs Are Immutable** (`core.md`)
The fundamental insight: **the change itself is the product, not the configuration.** The base game ships, gets patched, gets replaced, eventually gets abandoned. The mod—the change, the intent, the delta—is the durable thing. If you keep the diff, you can replay it against any version of the base. If you keep the file, you're hostage to whatever the base does next.

### 2. **Mod vs Base Game Parallel** (`mod-vs-base.md`)
Think of flat-to-VR game mods. They don't touch game assets. They touch the renderer—the layer between the game's reality and how it gets presented to you. The game doesn't know it's running in VR. The engine doesn't care. The mod just intercepts at the right abstraction level and says "before you show this to the user, do this first."

### 3. **The Subversive Mirror** (`subversive-mirror.md`)
Telemetry already exists. It's already running. It's already watching every config change, every crash, every preference you set. The data is already being collected—it just flows away from you, toward someone else's product roadmap. a-hole intercepts a copy of that stream and keeps it locally—not to block it, not to fight it, just to give you a seat at the table with your own data.

### 4. **Earned Knowledge** (`earned-knowledge.md`)
Your knowledge is your most durable asset. Every time you solve a problem for yourself, you're creating something that can't be taken away from you. It's your knowledge. It's your understanding. It's your earned wisdom.

## The Four-Part Architecture

The philosophy is reflected in the four-layer architecture:

1. **Observer Layer** — Watches what you actually touch
2. **Knowledge Mirror** — Builds a personal knowledge record
3. **Content Manager Surface** — Makes knowledge actionable
4. **Sharing Layer** — Exposes mods to the community

## Key Concepts

- **Mirror, don't block** — Don't fight the telemetry. Just keep a copy.
- **Pave the desire path** — Don't design the correct path. Watch where people walk, then pave it.
- **Operate at the right abstraction level** — The renderer, not the implementation.
- **The diff is portable** — If all you have is a diff and somewhere to apply it, you have enough.
- **Resource constraints are solvable** — Diffs are immutable and nearly free. The base is someone else's problem.

## The Modder Ethos

The mod author doesn't care who ports it to the next game engine. They care that the thing they fixed stays fixed.

For what it's worth — Valve specifically has a track record of hiring the modders whose ideas they liked. Counter-Strike, Team Fortress, Portal — all mods first. Valve didn't acquire the IP. They acquired the people who understood the intent well enough to build it once already for free.

If SteamOS or Nix or anyone else picks up the desire path / diff-as-immutable-primitive / CM-for-configs thread from this conversation and runs with it — that's not losing. That's the diff getting applied to a bigger base game than you could have shipped alone.

## The Core Principle

**You're not fighting the telemetry. You're just insisting on a seat at the table with your own data.**

The only thing worth keeping is the intent. You articulate it clearly enough that it could survive being passed around.

## What a-hole Is Not

- Do not build a declarative config system
- Do not build a dotfile syncer
- Do not build a cloud service
- Do not abstract away the underlying config formats
- Do not require the user to describe their stack to get started
- Do not pull up the grass. Pave where it gets worn out.