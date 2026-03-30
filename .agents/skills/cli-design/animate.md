---
description: >
  Improve a Rust CLI's motion and progress feedback when `indicatif` usage feels bare,
  misleading, or inconsistent across long-running operations.
user-invocable: true
---

Use this command when the CLI already has or clearly needs progress UI. Do not add spinners to work that finishes too quickly to justify them.

If `.ferrite.md` exists, read it first.

## Goals

- Replace bare spinners with descriptive labels.
- Add elapsed-time context after long waits.
- Implement CI-safe fallbacks.
- End long operations with calm completion summaries.

## Rules

- Under 300ms: print nothing.
- For indeterminate work: spinner with concrete label such as `Resolving dependencies`.
- For real totals: progress bar with actual `pos/len`.
- After ~3 seconds: include elapsed time in the completion message or status summary.
- In CI or non-interactive terminals: fall back to plain text, not spinner frames.

## Working method

1. Read the current `indicatif` usage and identify all long-running steps.
2. Replace unlabeled or vague spinner messages.
3. Centralize helpers like `make_spinner`, `make_spinner_or_plain`, `finish_step`, and `make_progress_bar`.
4. Ensure every spinner is cleared or finished cleanly.
5. Design a final completion summary that states outcome first, then timing, then warnings.

## Anti-patterns to remove

- `ProgressBar::new_spinner()` used with no message
- fake percentage bars
- multiple nested spinners for sequential work
- spinner frames left in scrollback after completion
- spinner output while a prompt is waiting for input
