---
description: >
  Improve prompt-driven Rust CLI flows when confirmation, selection, or text input
  needs better defaults, validation, and non-interactive safety.
user-invocable: true
---

Use this command when a CLI uses `dialoguer` or should use it, and the interaction model needs refinement.

If `.ferrite.md` exists, read it first.

## Goals

- Standardize on `dialoguer` with `ColorfulTheme`.
- Add inline validation.
- Add sensible defaults to selects and confirmations.
- Implement `--yes` / `-y` bypasses where automation needs them.
- Restore terminal state and exit 130 on Ctrl+C.

## Working method

1. Read prompt flows, CLI flags, and any CI or pipe handling.
2. Add `fn is_interactive() -> bool` and `fn require_interactive_or_flag()` if missing.
3. Wrap prompts with `ColorfulTheme::default()`.
4. Add `.validate_with(...)` for text input that has format constraints.
5. Use `.default(0)` for `Select`, `.defaults(&[...])` for `MultiSelect`, and `.default(false)` for destructive `Confirm` prompts.
6. Add `args.yes || Confirm::...` patterns where users need a non-interactive bypass.
7. Install `ctrlc::set_handler` for interactive commands.

## Constraints

- Do not prompt in CI.
- Do not use destructive default `true`.
- Do not ask the user for information already available in args, config, or environment.
- Do not let Ctrl+C leave the cursor hidden or the terminal in an inconsistent state.
