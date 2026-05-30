# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-05-30

This release focuses on correctness: several diagram types produced wrong output
(not just plain output), and the documentation described features that did not
exist. The renderer is now honest about what it does, and the crate follows
crates.io / open-source packaging conventions.

### Fixed

- **Flowchart**: node-shape syntax (`[ ]`, `( )`, `{ }`, `([ ])`, `[[ ]]`, …)
  and edge labels (`-->|text|`) are now parsed instead of being treated as part
  of the node id — boxes no longer read `A[Start]` or `|Yes| C(Pr`. Per-edge
  styles are tracked, and `;` is accepted as a statement separator.
- **Sequence**: a dashed message such as `Bob-->>Alice` no longer creates a
  phantom `Bob-` participant (arrow tokens are matched longest-first); dashed
  messages now render with a dashed line.
- **Class**: relationships now render both endpoints (e.g. `Animal <|-- Dog`)
  instead of dropping the target; `--|>` and the plain `--` association parse
  correctly; boxes keep declaration order; a divider separates fields from
  methods.
- **State**: `[*]` start/end markers now render as `●` / `◉` (previously dead
  code that always produced blanks).
- **ER**: right-hand cardinalities `|{` and `|o` are no longer dropped, so
  many-to-many relationships render; adjacent relationship lines no longer
  fabricate phantom entity boxes.
- **Pie**: labels containing `:` no longer blank the whole chart; negative and
  overlong bars are clamped; labels are right-aligned.
- `--ascii` is now honored by sequence, class, state, ER, and pie output
  (previously only flowcharts respected it).
- Empty or unrecognized input now returns an error instead of rendering a blank
  canvas; `--format json` reports `success: false` with a suggestion.

### Changed

- README rewritten so every example matches the actual binary output, and the
  dependency, CLI-flag, theme, and install claims are accurate.
- `Cargo.toml`: added `keywords`, `categories`, `readme`, `documentation`,
  `homepage`, and `rust-version`; set a real author; added an `include`
  allowlist so the published crate ships only source, license, and readme
  (previously it bundled unrelated AI-skill directories).
- Removed unused dependencies `crossterm`, `thiserror`, and `regex`.

### Known limitations (planned for 0.3)

- Theme **color** output (ANSI) is not yet wired: `--theme` selects a palette
  but the rendered output is currently monochrome.
- Node-shape **glyphs** (diamond/stadium/rounded/…) are parsed but rendered as
  rectangles.
- `direction` (`LR`/`RL`/`TD`/`TB`/`BT`) is parsed but layout is currently
  left-to-right only.
- `-.->` / `==>` edge styles are parsed but rendered the same as solid edges.

## [0.1.0] - 2026-03-30

- Initial release: flowchart, sequence, class, state, pie, and ER diagram
  rendering; six theme palettes; `--ascii` and `--format json` modes; CJK-aware
  width handling; Claude Code skill.

[Unreleased]: https://github.com/Binlogo/meraid/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Binlogo/meraid/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Binlogo/meraid/releases/tag/v0.1.0
