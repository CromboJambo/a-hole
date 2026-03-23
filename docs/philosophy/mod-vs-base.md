# The Mod vs Base Game Parallel

## The Core Insight

**Diffs are immutable. Everything else is swap-able.**

The base game ships, gets patched, gets replaced, eventually gets abandoned. The mod—the change, the intent, the delta—is the durable thing. If you keep the diff, you can replay it against any version of the base. If you keep the file, you're hostage to whatever the base does next.

This is the core insight of a-hole: **the change itself is the product, not the configuration.**

## Flat-to-VR as a Model

Think of flat-to-VR game mods. They don't touch game assets. They touch the renderer—the layer between the game's reality and how it gets presented to you.

The game doesn't know it's running in VR. The engine doesn't care. The mod just intercepts at the right abstraction level and says "before you show this to the user, do this first."

This is the key: **the mod operates at the abstraction layer, not the implementation layer.**

## Applied to Terminal Config

In your terminal stack, the shell or terminal emulator is the renderer. A config mod that works at the WezTerm or Nushell level works regardless of what application is running inside it—same way a VR mod works regardless of which game assets are on screen.

**The abstraction level you mod at determines your portability surface.**

If you mod at the config file level (editing `~/.config/wezterm/wezterm.lua`), you're tied to WezTerm version X. If you mod at the renderer level (injecting into the WezTerm process itself), you can work across WezTerm versions and even other terminal emulators that use similar rendering logic.

## The Portable Mod

A good config mod:

1. **Is scoped** — only touches what it needs to touch
2. **Is reversible** — can be removed without leaving traces
3. **Is version-agnostic** — works against the base it's designed for
4. **Is self-documenting** — the diff itself explains what it does

## The Base Game Problem

The base game (your OS, your toolchain, your vendor's software) evolves. It breaks. It changes. It gets abandoned.

Your mods—the changes you make to make things work for you—should be the durable thing. If you keep the file, you're hostage to whatever the base does next.

## Earned Knowledge as Mods

Every time you:
- Find a config setting that solves a problem
- Discover a pattern that works across multiple machines
- Create a workaround for a vendor's design flaw
- Customize a tool to match your workflow

You're creating a mod. The base game doesn't know about it. The vendor doesn't know about it. But it's your knowledge, and it's durable.

## The Mod Author's Mindset

The mod author doesn't care who ports it to the next game engine. They care that the thing they fixed stays fixed.

This is the modder ethos exactly. The mod author doesn't care who ports it to the next game engine. They care that the thing they fixed stays fixed.

For what it's worth — Valve specifically has a track record of hiring the modders whose ideas they liked. Counter-Strike, Team Fortress, Portal — all mods first. Valve didn't acquire the IP. They acquired the people who understood the intent well enough to build it once already for free.

## The Portable Diff

A diff is:
- Tiny (usually)
- Self-contained
- Reversible
- Self-documenting
- Portable

If all you have is a diff and somewhere to apply it, you have enough.

## The Desire Path

Don't design the correct path. Watch where people walk, then pave it.

The mod is the paved path. The base game is the terrain. The terrain will change. The path you've worn out is what matters.

## Summary

The mod vs base game parallel teaches us:

1. **Operate at the right abstraction level** — the renderer, not the implementation
2. **Keep the durable thing** — the diff, not the file
3. **Be version-agnostic** — work against interfaces, not versions
4. **Be portable** — your knowledge should travel with you, not get tied to one version of one tool
5. **Be self-documenting** — the mod itself explains what it does
6. **Be reversible** — nothing should be permanent except the intent

The only thing worth keeping is the intent. You articulate it clearly enough that it could survive being passed around.