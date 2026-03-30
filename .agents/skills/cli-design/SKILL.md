---
name: cli-design
description: >
  Create distinctive, production-grade CLI and TUI interfaces with high design
  quality. Use this skill when building command-line tools, terminal UIs,
  interactive prompt flows, or when improving the output aesthetics of any
  CLI application. Default implementation language is Rust.
license: Apache-2.0
---

When Claude works on a CLI, default to designing the terminal experience with the same care normally reserved for a web UI. Ferrite is opinionated on purpose: terminal output is product surface area, not debug exhaust.

## Default Rust Toolchain

| Concern | Crate | Why this default exists |
| --- | --- | --- |
| Argument parsing | `clap` v4 with `derive` | Mature help output, ergonomic derive macros, strong ecosystem fit |
| Progress and spinners | `indicatif` | Solid terminal primitives, good finish/clear behavior, multi-progress support |
| Interactive prompts | `dialoguer` + `ColorfulTheme` | Consistent prompt styling, validation hooks, reliable selection flows |
| Terminal styling | `console` | Styling that can respect terminal capabilities and `NO_COLOR` |
| Structured domain errors | `thiserror` | Clear typed errors with user-facing messages |
| App-level error propagation | `anyhow` | Practical context layering at boundaries |
| TUI | `ratatui` + `crossterm` | Production-ready terminal UI stack |
| Ctrl+C handling | `ctrlc` | Lets the app restore terminal state and exit intentionally |

## Design Thinking

Before editing code, answer these questions from the repository and the user's request:

- **Audience** — Who reads this output: first-time users, power users, CI logs, operators under stress?
- **Environment** — Is this interactive terminal use, CI, redirected stdout, log aggregation, or a TUI session?
- **Interaction Model** — Is the tool fire-and-forget, long-running, confirm-heavy, or exploratory?
- **Tone** — Should the tool feel surgical like `ripgrep`, reassuring like `cargo`, or conversational only where prompts require it?
- **Differentiation** — What makes this CLI memorable besides color? Better defaults, sharper wording, cleaner summaries, calmer failures?

Silence is a valid and powerful design choice. If output does not help the user decide, trust, recover, or verify, remove it.

## Context Gathering Protocol

1. Read `Cargo.toml` first. Confirm the actual crate stack before recommending changes.
2. Read the CLI entry points and output paths next: `src/main.rs`, `src/bin/*.rs`, `src/lib.rs`, and any `output.rs`, `ui.rs`, or `tui.rs` modules.
3. Ask at most 3 targeted questions, and only when the code cannot answer them. Prefer questions about audience, runtime environment, and whether the command must be pipe-safe or CI-safe.

## CLI Aesthetics Guidelines

### Output Hierarchy & Typography → `reference/output-hierarchy.md`

**DO**
- Use exactly four output levels: section header, primary action, secondary detail, and debug.
- Indent nested detail by 2 spaces, never tabs.
- Align key-value output when scanning speed matters, especially summaries and diagnostics.
- Use blank lines as punctuation between conceptual groups.

**DON'T**
- Dump five unrelated facts into one line because it “saves space.”
- Print every event with equal emphasis.
- Use ad-hoc table borders built from repeated `-` when Unicode box drawing is available.

### Color & Symbols → `reference/color-and-symbols.md`

**DO**
- Keep color semantic and stable: success is green, warnings yellow, errors red, info cyan, muted detail dim.
- Offer Unicode symbols by default and ASCII fallback when the environment demands it.
- Respect `NO_COLOR` and terminal capability checks before styling output.
- Let symbols support scanning, not carry the whole message.

**DON'T**
- Color every line.
- Encode meaning only in color with no text fallback.
- Color paths, timestamps, or already-noisy text that users may copy-paste.
- Use blinking text (`\e[5m`) for urgency.

### Interaction Patterns → `reference/interaction-patterns.md`

**DO**
- Use `dialoguer` with `ColorfulTheme` for all prompts so the interaction surface feels consistent.
- Fail fast outside interactive terminals unless a non-interactive flag like `--yes` or `--output` is provided.
- Make destructive confirmation defaults `false`.
- Restore cursor state and exit 130 on Ctrl+C.

**DON'T**
- Hang forever waiting for stdin in CI.
- Prompt users for information already available in flags or config.
- Hide destructive consequences behind cheerful wording.

### Progress & Feedback → `reference/progress-and-feedback.md`

**DO**
- Show nothing for operations under 300ms.
- Use spinners only for indeterminate work and clear them when done.
- Use progress bars only when position and total are real.
- End long operations with outcome-first summaries.

**DON'T**
- Leave spinner artifacts in the scroll buffer.
- Fake percentages.
- Keep a spinner spinning during blocking user input.
- Print “Done” with no statement of what finished.

### Error Design → `reference/error-design.md`

**DO**
- Structure errors as WHAT happened, WHY it likely happened, and HOW to recover.
- Route user-facing errors to stderr.
- Use typed `thiserror` variants for anticipated failures and `anyhow` for context propagation.
- Gate raw chains and stack-like detail behind `--debug`.

**DON'T**
- Surface raw `anyhow` chains or Rust `Debug` dumps to end users by default.
- Print errors with `println!`.
- Exit with code 0 on failure.
- Say “Something went wrong,” “Invalid input,” or “Error occurred.”

### UX Writing for CLIs → `reference/ux-writing-cli.md`

**DO**
- Make every `about` string explain effect, not restate the subcommand name.
- Prefer concrete nouns and verbs: “Remove generated cache files” beats “Run cleanup.”
- Use one term per concept throughout flags, prompts, and summaries.
- Write prompts and errors for stressed readers, not for maintainers.

**DON'T**
- Repeat the command name as help text.
- Use vague filler like “process,” “handle,” or “perform operation” when a more exact verb exists.
- Switch between “workspace,” “project,” and “repo” for the same thing.

## The AI CLI Slop Test

Reject the implementation if any item below fails:

1. Would a user see a raw `anyhow` chain, `Debug` struct, or panic-style dump during normal failure paths?
2. Does any user-facing error still go through `println!` instead of `eprintln!`?
3. Does any spinner or progress element leave artifacts behind instead of clearing or finishing cleanly?
4. Does any progress bar imply fake precision, invented percentages, or made-up totals?
5. Can any failure path still exit with `0` instead of a non-zero code?
6. Are any messages still vague enough to read like “Something went wrong” or “Invalid input” without telling the user what to do next?
7. Does the CLI ignore `NO_COLOR`, over-color every line, or use decorative output such as banners or blinking text?
8. Can the tool block on stdin in CI or non-interactive mode because terminal checks are missing?

## Implementation Principles

- Match the amount of interface design to the interaction model. A one-shot formatter needs crisp output, not a theatrical spinner system.
- Prefer a small output vocabulary repeated consistently over many one-off styles.
- Centralize symbols, styles, and output helpers once a project has more than two commands.
- Treat stdout as data surface and stderr as status surface. Be explicit when a command must be pipe-safe.
- If a TUI is warranted, design the command-line path first so automation still has a clean interface.
- Good defaults beat optional complexity. Add flags only when they unlock real user control.

## Explicit Anti-Patterns

Call these out when present and remove them when asked to improve a CLI:

- Raw `anyhow` chain or Rust `Debug` output surfaced to end users
- `println!` for errors instead of `eprintln!`
- Spinners that leave artifacts in scroll buffer because they are not cleared on completion
- Progress bars with fake or estimated percentages
- `exit(0)` on failure
- Vague errors such as `Something went wrong`, `Invalid input`, or `Error occurred`
- Blinking text via `\e[5m`
- ASCII art banners or figlet headers in production tools
- Every line colored, causing color fatigue so nothing stands out
- `NO_COLOR` not respected
- Tool hangs waiting for stdin in CI because `!is_term()` was not checked

## How to Use the Reference Pack

Read only the references needed for the task:

- Read `reference/output-hierarchy.md` when touching summary output, tables, or layout.
- Read `reference/color-and-symbols.md` when changing symbols, styles, or fallback behavior.
- Read `reference/interaction-patterns.md` when adding prompts, confirmations, or selection flows.
- Read `reference/progress-and-feedback.md` when the command runs longer than a blink.
- Read `reference/error-design.md` when editing failures, exit codes, or debug behavior.
- Read `reference/ux-writing-cli.md` when editing `clap` help, prompt copy, or success messaging.
