---
name: meraid
version: 1.0.0
description: |
  Render Mermaid diagrams in terminal. Use this skill when you need to visualize 
  code architecture, flowcharts, sequence diagrams, or any Mermaid diagrams.
  AI-friendly: supports JSON output for programmatic parsing.
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
---

# Mermaid Diagram Renderer

This skill renders Mermaid diagrams in the terminal using meraid.

## When to Use

- User asks to visualize code architecture
- User wants to see a flowchart, sequence diagram, or class diagram
- User mentions Mermaid or diagram visualization
- You want to render a Mermaid diagram to verify its structure

## Commands

### Render a Mermaid diagram

```bash
meraid - <<'EOF'
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Process]
    B -->|No| D[Skip]
    C --> E[End]
    D --> E
EOF
```

### Render with specific theme

```bash
meraid --theme neon - <<'EOF'
graph LR
    A --> B --> C
EOF
```

### Available themes

- `default` - Cyan nodes, yellow arrows
- `terra` - Warm colors (brown, orange)
- `neon` - Magenta nodes, green arrows (cyberpunk)
- `mono` - Grayscale
- `amber` - Amber CRT style
- `phosphor` - Green phosphor tube style

### JSON output (AI-friendly)

```bash
meraid --format json - <<'EOF'
graph LR
    A --> B
EOF
```

Returns structured JSON with:
- `success`: boolean
- `diagram`: rendered diagram string
- `error`: error info (if failed)
- `metadata`: diagram type, theme, dimensions, node/edge count

### ASCII-only output

```bash
meraid --ascii - <<'EOF'
graph LR
    A --> B
EOF
```

## Installation

If meraid is not installed globally, use the local binary from the repo:

```bash
# Get repo root and use local meraid
cd "$(git rev-parse --show-toplevel)"
./target/release/meraid - <<'EOF'
graph LR
    A --> B
EOF
```

To install globally:

```bash
cargo install meraid
```

Or build from source:

```bash
git clone https://github.com/Binlogo/meraid.git
cd meraid
cargo build --release
cargo install --path .
```

## Examples

### Flowchart

```bash
meraid - <<'EOF'
graph TD
    User -->|Request| API
    API -->|Query| Database
    API -->|Call| Service
    Service -->|Response| API
    API -->|Response| User
EOF
```

### Sequence Diagram

```bash
meraid - <<'EOF'
sequenceDiagram
    participant User
    participant API
    participant DB
    User->>API: GET /items
    API->>DB: SELECT * FROM items
    DB-->>API: results
    API-->>User: JSON data
EOF
```

### Class Diagram

```bash
meraid - <<'EOF'
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
    class Dog {
        +String breed
        +bark()
    }
    Animal <|-- Dog
EOF
```

### State Diagram

```bash
meraid - <<'EOF'
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: start
    Processing --> Done: complete
    Done --> [*]
EOF
```

### Pie Chart

```bash
meraid - <<'EOF'
pie title Market Share
    "Product A" : 45
    "Product B" : 30
    "Product C" : 25
EOF
```
