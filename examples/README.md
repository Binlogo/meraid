# meraid examples — a guided tour

Every file here is a real, runnable Mermaid diagram. The output blocks below are
**copied verbatim from the binary** — what you see is exactly what `meraid`
prints today (v0.2). Nothing here is aspirational.

## Run them

```sh
# build once
cargo build --release

# render a single example
./target/release/meraid examples/01-flowchart.mmd

# render every example in one go (passes extra flags through)
examples/run-all.sh
examples/run-all.sh --ascii
examples/run-all.sh --theme neon
```

If you've installed meraid (`cargo install meraid`), drop the `./target/release/`
prefix and just run `meraid examples/01-flowchart.mmd`.

You can also pipe from stdin — handy for AI agents and shell glue:

```sh
printf 'graph LR\n  A --> B --> C' | meraid -
```

---

## 1. Flowchart — linear (`01-flowchart.mmd`)

```
graph LR
    A[Input] --> B(Parse) --> C{Valid?} --> D([Output])
```

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Input   │ ──▶│  Parse   │ ──▶│  Valid?  │ ──▶│  Output  │
│          │    │          │    │          │    │          │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

Chained edges (`A --> B --> C`), `LR`/`RL`/`TB`/`TD`/`BT` directions, and the
shape brackets `[]` `()` `{}` `([])` all parse. **Shapes currently render as the
same rectangle** — distinct glyphs per shape are on the 0.3 roadmap.

## 2. Flowchart — branching (`02-flowchart-branch.mmd`)

```
graph TD
    A[Start] --> B{OK?}
    B -->|yes| C[Save]
    B -->|no| D[Stop]
```

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│  Start   │ ──▶│   OK?    │ ye▶│   Save   │
│          │    │          ││   │          │
└──────────┘    └──────────┘│   └──────────┘
                            │
                            │   ┌──────────┐
                             no▶│   Stop   │
                                │          │
                                └──────────┘
```

Branches and edge labels (`|yes|`, `|no|`) parse and route correctly. Note that
edge-label placement on branching layouts is still rough — long labels get
clipped (`yes` → `ye▶`). This is honestly a 0.3 layout item. For clean output
today, prefer linear chains or keep branch labels short.

## 3. Sequence diagram (`03-sequence.mmd`)

```
sequenceDiagram
    participant Client
    participant Server
    Client->>Server: GET /status
    Server-->>Client: 200 OK
    Client->>Server: POST /jobs
    Server-->>Client: 202 Accepted
```

```
   Client            Server
      │                 │

      ├─────────────────▶ GET /status
      ◀┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤ 200 OK
      ├─────────────────▶ POST /jobs
      ◀┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤ 202 Accepted
```

Solid arrows (`->>`) render as solid lines; dashed/reply arrows (`-->>`) render
as dotted lines (`┄`). This is one of meraid's most polished outputs.

## 4. Class diagram (`04-class.mmd`)

```
classDiagram
    class Shape {
        +String name
        +area() float
    }
    class Circle {
        +float radius
        +area() float
    }
    Shape <|-- Circle
```
<sub>(truncated — see the file for all three classes)</sub>

```
┌────────────────┐
│     Shape      │
├────────────────┤
│+String name    │
├────────────────┤
│+area()         │
└────────────────┘

...

Shape <|-- Circle
Shape <|-- Square
```

Fields and methods are split into their own compartments. Relationships
(`<|--`, `-->`, `*--`, `o--`, `..>`, `..|>`) are parsed and echoed below the
boxes as a relationship list. Drawing the relationship arrows *between* the
boxes graphically is a 0.3 item.

## 5. State diagram (`05-state.mmd`)

```
stateDiagram-v2
    [*] --> Idle
    Idle --> Running: start
    Running --> Idle: pause
    Running --> Done: finish
    Done --> [*]
```

```
● ──▶ Idle
Idle ──▶ Running : start
Running ──▶ Idle : pause
Running ──▶ Done : finish
Done ──▶ ◉
```

`[*]` becomes a start (`●`) or end (`◉`) marker. Transition labels after `:`
are preserved. Rendered today as a transition list rather than positioned state
boxes — boxed layout is 0.3.

## 6. Pie chart (`06-pie.mmd`)

```
pie title Language usage
    "Rust" : 58
    "TypeScript" : 27
    "Shell" : 15
```

```
      Rust┃███████████████████████ 58.0%
TypeScript┃███████████ 27.0%
     Shell┃██████ 15.0%
```

Values are normalized to percentages and drawn as right-aligned horizontal bars.
Works well as-is.

## 7. Entity-relationship diagram (`07-er.mmd`)

```
erDiagram
    USER {
        int id PK
        string email
    }
    POST {
        int id PK
        int user_id FK
        string title
    }
    USER ||--o{ POST : writes
```

```
┌────────────────────┐
│        USER        │
├────────────────────┤
│PK    : id          │
│      : email       │
└────────────────────┘

┌────────────────────┐
│        POST        │
├────────────────────┤
│PK    : id          │
│   FK : user_id     │
│      : title       │
└────────────────────┘

USER ||--o{ POST
```

Entity attribute blocks render with `PK`/`FK` key markers. The cardinality
relationship (`||--o{`) is parsed and echoed below.

## CJK / wide-character support

meraid measures display width with `unicode-width`, so Chinese, Japanese, and
Korean text aligns correctly. See `cjk-flowchart.mmd`, `cjk-sequence.mmd`, and
`cjk-class.mmd`:

```sh
meraid examples/cjk-sequence.mmd
```

```
   客户端           API网关           用户服务           数据库
      │                 │                 │                 │

      ├─────────────────▶ POST /login
                        ├─────────────────▶ 验证凭证
                                          ├─────────────────▶ 查询用户信息
                                          ◀┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤ 返回用户记录
                        ◀┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤ 签发 JWT token
      ◀┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤ 200 OK
```

---

## Flags worth knowing

| Flag | What it does |
| --- | --- |
| `--ascii` / `-a` | Use only ASCII box-drawing (`+ - \| -->`) — great for logs, CI, and terminals without Unicode box glyphs. |
| `--theme <name>` | `default`, `terra`, `neon`, `mono`, `amber`, `phosphor`. Affects ANSI colors when writing to a TTY. |
| `--format json` | Emit a structured JSON envelope (`success`, `diagram`, `error`, `metadata`) instead of plain text — designed for AI agents and tooling. |
| `--padding-x` / `--padding-y` | Tune the whitespace inside boxes. |

### `--ascii` example

```sh
meraid examples/01-flowchart.mmd --ascii
```

```
+----------+    +----------+    +----------+    +----------+
|  Input   | -->|  Parse   | -->|  Valid?  | -->|  Output  |
|          |    |          |    |          |    |          |
+----------+    +----------+    +----------+    +----------+
```

### `--format json` example

```sh
meraid examples/07-er.mmd --format json
```

```json
{
  "success": true,
  "diagram": "...rendered text...",
  "metadata": {
    "diagram_type": "er",
    "theme": "default",
    "width": 22,
    "height": 16,
    "nodes": 2,
    "edges": 1
  }
}
```

On error, `success` is `false` and `error` carries `message` plus, when
available, `line`/`column`/`suggestion`. The process exits non-zero on any
parse/render failure, so you can use it directly in scripts:

```sh
meraid diagram.mmd || echo "failed to render"
```

---

## What's supported in 0.2 vs. coming in 0.3

| Diagram type | Parses | Renders today | Polished output |
| --- | :---: | --- | :---: |
| Flowchart (linear) | ✅ | boxes + arrows | ✅ |
| Flowchart (branching) | ✅ | boxes + arrows, rough label placement | ⚠️ 0.3 |
| Sequence | ✅ | lifelines, solid/dashed arrows | ✅ |
| Class | ✅ | boxes + field/method compartments, relationship list | partial |
| State | ✅ | transition list with start/end markers | partial → boxed in 0.3 |
| Pie | ✅ | horizontal percentage bars | ✅ |
| ER | ✅ | entity boxes with PK/FK, relationship list | partial |

**Deferred to 0.3** (tracked, not yet implemented): per-shape glyphs in
flowcharts, branch-aware edge-label placement, drawn relationship arrows for
class/ER, and boxed state-machine layout.

**Not supported** (clean error + non-zero exit): `gitGraph`, `block`,
`treemap`, and other diagram types not listed above. meraid tells you exactly
which type is unsupported rather than guessing.
