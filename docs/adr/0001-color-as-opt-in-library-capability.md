# Color is an opt-in, role-based library capability; the CLI owns detection

## Status

accepted

## Context & Decision

For 0.3 we wire up the long-defined themes to produce real ANSI color. `meraid`
is published as both a library and a CLI, so where color lives is a semver-level
decision. We decided color is a **library capability that is opt-in**: the
library is a pure function `(source, theme, ColorMode) -> bytes` where
`ColorMode` is one of `None | Ansi256 | TrueColor`, and `None` is the default
(byte-for-byte identical to today's monochrome output). All environment policy —
`--color`, `NO_COLOR`, stdout TTY detection, `COLORTERM` — is resolved **in the
CLI** down to a single `ColorMode` that is passed in. Coloring is applied **by
role** (node fg, edge, edge label, start/end marker), foreground-only for 0.3,
with the `default` theme emitting no overrides so it inherits the terminal's own
colors.

## Considered Options

- **Default-on color in the library** — rejected: a breaking change for existing
  library consumers and it pollutes any captured string (JSON, logs, tests).
- **CLI-only post-hoc colorization** — rejected: a finished `─` carries no hint
  whether it is a node border or an edge wire, so color-by-role is impossible
  once the diagram is flattened; role is only knowable at draw time.
- **Library auto-detects from env** — rejected: makes `render()` non-
  deterministic, breaks exact-escape snapshot tests, and is hostile to TUI
  embedders that already own the terminal.

## Consequences

- `render()` keeps its behavior and output; color is reached via an explicit
  `Renderer::color_mode(...)`, so the common path is unchanged. (`Theme`'s color
  fields did change from `Color` to `Option<Color>` to express "inherit" — a
  breaking change only for code that constructs `Theme` directly; users of
  `Theme::get` are unaffected.)
- The library is deterministic and testable with exact-escape assertions.
- `--format json` always passes `ColorMode::None`, keeping the `diagram` field
  clean and its width/height metadata correct.
- Color is invisible out-of-the-box: a user sees it only by naming a non-default
  theme. This is the deliberate cost of respecting arbitrary terminal palettes.
