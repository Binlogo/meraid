<h1 align="center">meraid</h1>

<p align="center">在终端或 Rust 应用中渲染 Mermaid 图表</p>

<p align="center">
  <img src="docs/demo/meraid-demo.svg" alt="meraid 演示" width="800">
</p>

<p align="center">
  <a href="https://crates.io/crates/meraid">
    <img src="https://img.shields.io/crates/v/meraid?style=flat-square" alt="Crates.io">
  </a>
  <a href="https://github.com/Binlogo/meraid/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/Binlogo/meraid/ci.yml?style=flat-square" alt="CI">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-green.svg?style=flat-square" alt="License: MIT">
  </a>
  <a href="https://rust-lang.org">
    <img src="https://img.shields.io/badge/Rust-2021-orange.svg?style=flat-square" alt="Rust 2021">
  </a>
</p>

[English](README.md) | 中文

## 特性

- **纯 Rust 实现** — 零外部依赖，极速运行，完全可移植
- **AI 友好** — JSON 输出模式，方便 AI 编程工具解析
- **5+ 图表类型** — 流程图、时序图、类图、状态图、饼图
- **6 套主题** — default, terra, neon, mono, amber, phosphor
- **ASCII 回退** — 兼容任何终端
- **管道友好 CLI** — `cat diagram.mmd | meraid` 即刻使用

## 为什么选择 Meraid？

Mermaid 是文档编写的神器，但渲染它通常需要浏览器或外部服务。Meraid 让 Mermaid 渲染直接进入终端 — 非常适合 SSH 会话、CI 日志、TUI 应用或任何有 Rust 的环境。

为 Rust 生态圈而生，提供快速、零依赖的替代方案。

## 安装

### 从 Crates.io 安装

```bash
cargo install meraid
```

### 从源码构建

```bash
git clone https://github.com/Binlogo/meraid.git
cd meraid
cargo build --release
cargo install --path .
```

### 使用 Homebrew（即将支持）

```bash
brew install meraid
```

## 快速开始

### CLI 使用

```bash
# 从文件渲染
meraid diagram.mmd

# 从 stdin 输入
echo "graph LR; A-->B-->C" | meraid

# 使用主题
meraid diagram.mmd --theme neon

# ASCII 纯文本输出
meraid diagram.mmd --ascii

# JSON 输出（AI 友好）
meraid diagram.mmd --format json
```

### Rust 库使用

```rust
use meraid::{render, ThemeType};

fn main() {
    let diagram = render("graph LR\n  A --> B --> C", ThemeType::Default).unwrap();
    println!("{}", diagram);
}
```

## 支持的图表类型

### 流程图

支持所有方向：`LR`、`RL`、`TD`/`TB`、`BT`。

````mermaid
graph TD
    A[开始] --> B{是否有效?}
    B -->|是| C(处理)
    C --> D([完成])
    B -->|否| E[错误]
````

```
┌─────────────┐
│             │
│    开始     │
│             │
└──────┬──────┘
       │
       ▼
┌──────◇──────┐
│             │
│  是否有效?  │
│             │
└──────◇──────┘
       │
       ╰──────────────────╮
    是 │                  │否
       ▼                  ▼
╭─────────────╮    ┌─────────────┐
│             │    │             │
│    处理     │    │    错误    │
│             │    │             │
╰──────┬──────╯    └─────────────┘
       │
       ▼
╭─────────────╮
(             )
(    完成     )
(             )
╰─────────────╯
```

**节点形状：** 矩形 `[文本]`，圆角 `(文本)`，菱形 `{文本}`，体育场 `([文本])`，子程序 `[[文本]]`

**连线样式：** 实线 `-->`，虚线 `-.->`，粗线 `==>`，带标签 `-->|文本|`

### 时序图

````mermaid
sequenceDiagram
    Alice->>Bob: 你好 Bob
    Bob-->>Alice: 你好 Alice
    Alice->>Bob: 最近怎么样?
    Bob-->>Alice: 很好!
````

```
 ┌──────────┐      ┌──────────┐
 │  Alice   │      │   Bob    │
 └──────────┘      └──────────┘
      ┆ 你好 Bob        ┆
      ──────────────────►
      ┆ 你好 Alice      ┆
      ◄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
      ┆ 最近怎么样?     ┆
      ──────────────────►
      ┆ 很好!           ┆
      ◄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
```

**消息类型：** 实线箭头 `->>`，虚线箭头 `-->>`

**参与者：** `participant`、`actor`、别名

### 类图

````mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
    class Dog {
        +String breed
        +fetch()
    }
    Animal <|-- Dog
````

```
  ┌──────────────┐
  │    Animal    │
  ├──────────────┤
  │ +String name │
  │ +int age     │
  ├──────────────┤
  │ +makeSound() │
  └──────────────┘
          △
          │
  ┌───────────────┐
  │      Dog      │
  ├───────────────┤
  │ +String breed │
  ├───────────────┤
  │ +fetch()      │
  └───────────────┘
```

**关系：** 继承 `<|--`，组合 `*--`，聚合 `o--`，关联 `--`

**成员：** 属性和方法，支持可见性（`+` 公有，`-` 私有，`#` 受保护）

### 状态图

````mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: start
    Processing --> Done: complete
    Done --> [*]
````

```
╭───────◯──────╮
│              │
│      ●       │
│              │
╰───────◯──────╯
        │
        ▼
╭──────────────╮
│              │
│     Idle     │
│              │
╰───────┬──────╯
        │
   start│
        ▼
╭──────────────╮
│              │
│  Processing  │
│              │
╰───────┬──────╯
        │
complete│
        ▼
╭──────────────╮
│              │
│     Done     │
│              │
╰───────┬──────╯
        │
        ▼
╭───────◯──────╮
│              │
│      ◉       │
│              │
╰───────◯──────╯
```

**特性：** `[*]` 起始/终止状态，转换标签，复合状态

### 饼图

````mermaid
pie title 领养的宠物
    "狗" : 386
    "猫" : 85
    "鼠" : 15
````

```
  狗┃████████████████████████████████  79.4%
  猫┃▓▓▓▓▓▓▓  17.5%
  鼠┃░   3.1%
```

### ER 图

````mermaid
erDiagram
    CUSTOMER {
        int id PK
        string name
        string email
    }
    ORDER {
        int id PK
        int customer_id FK
        date order_date
    }
    CUSTOMER ||--o{ ORDER : places
````

```
  ┌────────────────────┐
  │      CUSTOMER      │
  ├────────────────────┤
  │PK    : id          │
  │      : name        │
  │      : email       │
  └────────────────────┘
  
  ┌────────────────────┐
  │       ORDER        │
  ├────────────────────┤
  │PK    : id          │
  │   FK : customer_id │
  │      : order_date  │
  └────────────────────┘
  
  CUSTOMER ||--o{ ORDER
```

**基数符号：**
- `||` 恰好一个
- `}|` 一个或多个  
- `o|` 零个或一个
- `o{` 零个或多个

**属性标记：**
- `PK` 主键
- `FK` 外键

## CLI 选项

| 参数 | 描述 |
|------|------|
| `--ascii` | ASCII 纯文本输出（无 Unicode 边框） |
| `--theme 名称` | 颜色主题。可选：default, terra, neon, mono, amber, phosphor |
| `--padding-x N` | 节点内水平边距（默认：4） |
| `--padding-y N` | 节点内垂直边距（默认：2） |
| `--width N` | 最大输出宽度（默认：120） |
| `--sharp-edges` | 连线使用尖角而非圆角 |
| `--format 格式` | 输出格式：text 或 json（AI 友好） |

## 主题

6 套内置主题：

| 主题 | 颜色 | 描述 |
|------|------|------|
| `default` | 青色节点，黄色箭头 | 默认终端颜色 |
| `terra` | 暖色调（棕色、橙色） | 复古风格 |
| `neon` | 洋红节点，绿色箭头 | 赛博朋克风格 |
| `mono` | 灰度单色 | 简洁干净 |
| `amber` | 琥珀色 CRT 风格 | 经典琥珀显示器 |
| `phosphor` | 绿色荧光管风格 | 经典绿色终端 |

## 路线图

- [x] ER 图 ✅
- [ ] 块图
- [ ] Git 提交图
- [ ] 树图
- [ ] 思维导图
- [ ] 更多主题（gruvbox, monokai, dracula, nord, solarized）
- [ ] 自动适应终端宽度
- [ ] 交互式 TUI 查看器

## 贡献

欢迎贡献！请随时提交 Pull Request。

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/amazing-feature`）
3. 提交更改（`git commit -m 'Add amazing feature'`）
4. 推送分支（`git push origin feature/amazing-feature`）
5. 打开 Pull Request

## 致谢

灵感来源：[termaid](https://github.com/fasouto/termaid) by fasouto

## 许可证

MIT 许可证 — 详见 [LICENSE](LICENSE)。

---

<p align="center">用 ❤️ Rust 打造</p>
