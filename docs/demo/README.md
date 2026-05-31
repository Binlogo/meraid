# Demo assets

`meraid-demo.svg` is the **editable source**. `meraid-demo.png` is what the
project READMEs actually embed.

## Why the README links the PNG, not the SVG

The demo shows real terminal output — box-drawing characters (`─ │ ┌ ┤ ▶ …`)
laid out on a monospace grid. When an SVG is embedded via `<img>`, GitHub
renders it in an isolated mode that ignores `xml:space="preserve"` and falls
back to a font where box-drawing glyphs don't share the ASCII advance width, so
the columns collapse and the diagram garbles. Rasterizing to PNG sidesteps all
of that: the bitmap displays identically everywhere.

## Regenerating

After editing the SVG, re-render the PNG (2× for crisp display at `width=800`):

```sh
rsvg-convert -w 2400 docs/demo/meraid-demo.svg -o docs/demo/meraid-demo.png
```

The text blocks inside the SVG are copied verbatim from `meraid`'s real output
(`cat ci.mmd | meraid`, the sequence diagram, and `--format json`) — keep them
faithful when editing.
