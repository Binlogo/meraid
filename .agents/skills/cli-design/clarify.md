---
description: >
  Rewrite `clap` help text for a Rust CLI when commands, flags, and subcommands read
  like internal labels instead of explaining their real effect and use.
user-invocable: true
---

Use this command when help text is technically present but not actually helpful.

If `.ferrite.md` exists, read it first.

## Rewrite standards

- Every `about` string must explain the effect of running the command.
- Every flag description must answer: what does this do when I pass it?
- Every subcommand description must answer: when would I use this?
- Prefer concrete verbs such as `delete`, `write`, `scan`, `publish`, `compare`, `watch`.
- Remove tautologies like `Run build`, `Manage config`, `Set verbose mode`.

## Working method

1. Read all `clap` parser, subcommand, and args structs.
2. Rewrite root `about`, subcommand `about`, and flag `help` strings.
3. Keep terminology consistent across help text, prompts, and summaries.
4. Re-check examples and defaults for wording drift.

## Quality bar

Good help text should let a user decide which command to run without opening the source.
