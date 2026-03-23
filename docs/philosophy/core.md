# The Philosophy of a-hole

## Diffs Are Immutable

**Diffs are immutable. Everything else is swap-able.**

The base game ships, gets patched, gets replaced, eventually gets abandoned. The mod—the change, the intent, the delta—is the durable thing. If you keep the diff, you can replay it against any version of the base. If you keep the file, you're hostage to whatever the base does next.

This is the core insight of a-hole: **the change itself is the product, not the configuration.**

## The Mod vs Base Game Parallel

Think of flat-to-VR game mods. They don't touch game assets. They touch the renderer—the layer between the game's reality and how it gets presented to you. The game doesn't know it's running in VR. The engine doesn't care. The mod just intercepts at the right abstraction level and says "before you show this to the user, do this first."

In your terminal stack, the shell or terminal emulator is the renderer. A config mod that works at the WezTerm or Nushell level works regardless of what application is running inside it—same way a VR mod works regardless of which game assets are on screen.

**The abstraction level you mod at determines your portability surface.**

## The Subversive Mirror

Telemetry already exists. It's already running. It's already watching every config change, every crash, every preference you set. The data is already being collected—it just flows away from you, toward someone else's product roadmap.

a-hole intercepts a copy of that stream and keeps it locally—not to block it, not to fight it, just to give you a seat at the table with your own data. The pipe already exists. We're just pointing a copy of it in the other direction.

**Mirror, don't block.**

## Resource Constraints

If storage and memory are out of someone's budget can they still utilize resources left?

What you're describing is spare capacity as commons. The same way Pi-hole runs on a $15 Raspberry Pi someone had in a drawer, Folding@home uses idle CPU cycles, SETI@home used overnight compute nobody was buying anyway.

Diffs are immutable and nearly free. The base is someone else's problem. If all you have is a diff and somewhere to apply it, you have enough.

## What It Is

A lightweight background process that:

- **Observes** what you actually touch in your config files
- **Mirrors** telemetry back to you as structured data
- **Tracks** your own config diffs as earned knowledge
- **Acts** as a Content Manager for your terminal stack
- **Paves** desire paths instead of pulling up grass

## What It Is Not

- Do not build a declarative config system
- Do not build a dotfile syncer
- Do not build a cloud service
- Do not abstract away the underlying config formats
- Do not require the user to describe their stack to get started
- Do not pull up the grass. Pave where it gets worn out.

## The Core Principle

You're not fighting the telemetry. You're just insisting on a seat at the table with your own data.

The only thing worth keeping is the intent. You articulate it clearly enough that it could survive being passed around.