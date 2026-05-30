# Contributing to meraid

Thanks for your interest in improving meraid! Contributions of all kinds are
welcome — bug reports, feature requests, documentation, and code.

## Getting started

```bash
git clone https://github.com/Binlogo/meraid.git
cd meraid
cargo build
cargo test
```

meraid is both a binary and a library:

- CLI entry point: `src/main.rs`
- Library: `src/lib.rs` (`parse_mermaid`, `render`, plus the `diagram`,
  `parser`, `layout`, `render`, and `theme` modules)

The pipeline is `parse_mermaid` → `Layout::layout` → `Renderer::render`.

## Before opening a pull request

Please make sure the following all pass locally — CI runs the same checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
```

Guidelines:

- Keep the minimum supported Rust version (see `rust-version` in `Cargo.toml`).
- Add a regression test for every bug fix and a test for every new feature.
  Parser tests live in `src/parser.rs`; integration-style tests live in
  `src/lib.rs`.
- When you change rendered output, update the affected examples in `README.md`
  / `README-zh.md` by piping through the built binary so the docs stay honest.
- Add a bullet under `[Unreleased]` in `CHANGELOG.md`.
- Match the surrounding code style; `rustfmt` is the source of truth.

## Reporting bugs

Open an issue using the bug report template and include the exact Mermaid input,
the command you ran, the actual output, and what you expected.

## Roadmap

See the **Roadmap** section in the README for the larger items planned for the
next releases (theme color output, node-shape glyphs, direction-aware layout).

By contributing, you agree that your contributions are licensed under the
project's [MIT License](LICENSE).
