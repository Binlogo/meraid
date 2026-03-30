---
description: >
  Apply a final Ferrite quality pass when a Rust CLI already works and now needs
  sharper copy, cleaner output, and more consistent terminal presentation.
user-invocable: true
---

Use this command when the implementation is basically correct and the remaining work is refinement. Do not use it as the first pass on a broken CLI.

If `.ferrite.md` exists, read it first.

## Checklist

### 1. Copy & Messaging
- Rewrite vague success lines so they state the concrete outcome.
- Rewrite vague warnings so they explain what was skipped, degraded, or assumed.
- Rewrite vague errors so they tell the user what failed and what to do next.
- Make `clap` `about` strings explain effect, not command names.

### 2. Visual Consistency
- Normalize section headers, symbols, indentation, and blank-line rhythm.
- Ensure secondary details are muted instead of competing with primary actions.
- Remove decorative color or one-off formatting flourishes.

### 3. Robustness
- Confirm prompts are gated behind interactive checks.
- Confirm destructive confirmation defaults are `false`.
- Confirm Ctrl+C restores terminal state for interactive commands.

### 4. Output Hygiene
- Remove filler text like `Starting...`, `Done`, `Operation complete`, or `Successfully executed`.
- Move machine-readable output to stdout and status output to stderr when pipe safety matters.
- Gate non-essential trace output behind `--verbose` or `--debug`.
- Clear spinners cleanly and end long work with outcome-first summaries.

### 5. Cargo.toml verification
- Read `Cargo.toml` and confirm the implementation uses the actual crates in the project.
- If the crate stack differs from Ferrite defaults, adapt recommendations to the real dependencies instead of forcing a rewrite.

## Working method

1. Read `Cargo.toml`, the CLI entry point, and any shared output/error modules.
2. Read `.ferrite.md` if present.
3. Apply the smallest changes that noticeably improve the user experience.
4. Re-scan for `println!`, `eprintln!`, `ProgressBar`, `Confirm`, `Input`, and `Select` usage.
5. Present the diff in terms of user-facing improvements, not framework trivia.

Run the AI CLI Slop Test from SKILL.md.
