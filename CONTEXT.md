# meraid

A pure-Rust renderer that turns Mermaid diagram source into terminal output
(Unicode or ASCII, optionally colored).

## Language

**Theme**:
A named mapping from rendering roles to colors (e.g. `neon`, `amber`).
Selecting a theme is how a user asks for a particular colored look.
_Avoid_: Palette (use only for "the set of colors a theme contains").

**Role**:
A semantic slot in a rendered diagram that can be colored independently:
node foreground, node background, edge, edge label, and start/end marker.
A theme assigns one color per role; coloring is always by role, never per node.

**Default theme**:
The neutral theme that inherits the terminal's own colors — it overrides no
foreground or background. Color is opt-in: a user gets a recolored diagram only
by naming a non-default theme.

**Color mode**:
Whether colored ANSI output is emitted for a given run. Resolved at the CLI from
`--color`, stdout TTY detection, `NO_COLOR`, and `TERM`; never emitted for JSON
output.
