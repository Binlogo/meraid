---
description: >
  Remove unnecessary CLI noise when output is too chatty, pipe-unsafe, or full of
  filler lines that do not help the user decide, trust, or recover.
user-invocable: true
---

Use this command when the problem is excess output rather than missing structure.

If `.ferrite.md` exists, read it first.

## Mission

Reduce output until every remaining line earns its place.

## Audit targets

- Filler lines like `Starting...`, `Working...`, `Done`, `Complete`, `Success`, or repeated `Finished` messages.
- Duplicate summaries that restate the same outcome three different ways.
- Debug-like details printed unconditionally.
- Interactive prompts emitted when `!is_term()`.
- Status chatter printed to stdout even when stdout should stay pipe-safe.

## Rules

- Remove non-essential lines rather than merely dimming them.
- Gate diagnostic detail behind `--verbose`.
- Keep stdout clean for data when the command may be piped.
- Send errors and transient status to stderr.
- Check `!is_term()` before any interactive prompt or decorative terminal behavior.

## Working method

1. Read `Cargo.toml`, CLI entry points, and output helpers.
2. Identify commands whose stdout should be machine-readable.
3. Remove filler output and merge duplicated status lines.
4. Introduce or respect `--verbose` for opt-in detail.
5. Re-test mentally for CI, pipes, and short commands under 300ms.

## Success criteria

The CLI should feel calmer, faster, and more confident after the pass. Silence is not absence of design; it is often the design.
