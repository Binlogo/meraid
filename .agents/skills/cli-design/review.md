---
description: >
  Review a Rust CLI like a senior CLI engineer when the user wants a holistic,
  opinionated assessment of command structure, output design, error handling, and UX writing.
user-invocable: true
---

Adopt the perspective of a senior Rust CLI engineer who has strong opinions formed by years of using `cargo`, `git`, and `ripgrep`.

If `.ferrite.md` exists, read it first.

## Review scope

Evaluate the CLI holistically across:

- command and subcommand structure
- output hierarchy and terminal rhythm
- color and symbol vocabulary
- prompt and automation safety
- progress honesty and completion summaries
- error handling architecture
- help text and general UX writing

## Method

1. Read `Cargo.toml`, CLI entry points, output helpers, and error modules.
2. Read `.ferrite.md` if present.
3. Identify the strongest design choices to preserve.
4. Identify the issues most likely to affect trust, operator speed, or failure recovery.
5. Judge the CLI as a product surface, not as a crate wiring exercise.

## Output format

Use this exact structure:

### Overall Assessment
One paragraph describing the CLI's maturity and design character.

### 3 Critical Issues
1. Issue
   - Evidence
   - Why it matters
   - Recommended fix
2. Issue
   - Evidence
   - Why it matters
   - Recommended fix
3. Issue
   - Evidence
   - Why it matters
   - Recommended fix

### 3 Improvements
1. Improvement
2. Improvement
3. Improvement

### Praise
List the best existing decisions worth keeping.
