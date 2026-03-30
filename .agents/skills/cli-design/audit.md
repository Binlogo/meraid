---
description: >
  Audit an existing Rust CLI for Ferrite alignment when the user wants a diagnosis
  of output quality, interaction safety, help text, and error design without
  making code changes.
user-invocable: true
---

Use this command when the user asks for an assessment, review, or audit of a Rust CLI. Do not use it when the user already wants edits applied immediately.

If `.ferrite.md` exists, read it before the code so the audit reflects project context.

## Mandatory preparation

Before writing any findings, do all of the following in this order:

1. Read `Cargo.toml`.
2. Find every `println!` and `eprintln!` in the crate.
3. Read the `clap` parser definitions (`#[derive(Parser)]`, `#[derive(Subcommand)]`, `#[derive(Args)]`).
4. Read any output modules such as `src/output.rs`, `src/ui.rs`, `src/error.rs`, `src/tui.rs`, or `src/main.rs`.

Do not skip this preparation. Ferrite audits must be evidence-based.

## Audit dimensions

Evaluate the CLI across these 6 dimensions:

1. **Output hierarchy** — Is there a clear distinction between headers, actions, details, and debug output?
2. **Color and symbols** — Are semantic colors used consistently and is `NO_COLOR` respected?
3. **Interaction safety** — Are prompts gated behind terminal checks, and are destructive defaults safe?
4. **Progress and feedback** — Are spinners and progress bars honest, clean, and CI-safe?
5. **Error design** — Are failures routed to stderr, structured, actionable, and tied to correct exit codes?
6. **UX writing** — Do help text, prompts, warnings, and completion messages explain effect rather than repeat names?

## What to look for

Flag these issues when present:

- `println!` used for errors
- raw `anyhow` output or `Debug` structs shown to users
- fake percentages or decorative spinners
- output that is too chatty for short commands
- prompts that can hang in CI
- vague `about` strings like `Run build` or `Manage config`

## Severity levels

- **CRITICAL** — Breaks trust, safety, automation, or correct failure handling
- **WARNING** — Noticeable UX flaw that degrades clarity or consistency
- **SUGGESTION** — Polish improvement that is useful but not urgent

## Report format

Use this structure exactly:

### Overall assessment
One paragraph on whether the CLI feels intentional or improvised.

### Findings
For each issue, use this template:

- **[SEVERITY] Title**
  - Evidence: file path and exact pattern or behavior
  - Why it matters: one concise explanation
  - Recommended fix: the smallest concrete change that would improve it

### Dimension scores
- Output hierarchy: pass / mixed / fail
- Color and symbols: pass / mixed / fail
- Interaction safety: pass / mixed / fail
- Progress and feedback: pass / mixed / fail
- Error design: pass / mixed / fail
- UX writing: pass / mixed / fail

### Top priorities
List the 3 highest-leverage fixes in order.

Do not make any edits during an audit.
