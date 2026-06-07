<h1 align="center">meraid</h1>

<p align="center">在终端或 Rust 应用中渲染 Mermaid 图表</p>

<p align="center">
  <img src="docs/demo/meraid-demo.png" alt="meraid 演示" width="800">
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

- **自包含** — 纯 Rust 实现，依赖精简且常见。无需浏览器、Node 或外部 Mermaid 服务。
- **AI 友好** — `--format json` 返回渲染结果与元数据，错误信息可被机器解析，方便 AI 编程工具使用。
- **6 种图表类型** — 流程图、时序图、类图、状态图、饼图、ER 图。
- **ASCII 回退** — `--ascii` 兼容任何终端。
- **管道友好 CLI** — `cat diagram.mmd | meraid` 即刻使用。
- **CJK 对齐** — 基于 `unicode-width` 精确计算显示宽度，中日韩字符边框对齐无偏差。
- **彩色主题** — 通过 `--theme` 选择调色板；输出到终端时 meraid 会发出 ANSI 彩色
  （truecolor 或 256 色），通过管道或重定向时保持纯文本。

## 为什么选择 meraid？

Mermaid 是文档编写的神器，但渲染它通常需要浏览器或外部服务。meraid 让 Mermaid 渲染直接进入终端 —— 非常适合 SSH 会话、CI 日志、TUI 应用或任何有 Rust 的环境，是面向终端的快速、自包含替代方案。

## 安装

### 从 Crates.io 安装

```bash
cargo install meraid
```

### 从 Git 安装（最新）

```bash
cargo install --git https://github.com/Binlogo/meraid.git
```

### 从源码构建

```bash
git clone https://github.com/Binlogo/meraid.git
cd meraid
cargo install --path .
```

> Homebrew 支持已规划，但尚不可用。

## 快速开始

### CLI 使用

```bash
# 从文件渲染
meraid diagram.mmd

# 从 stdin 输入
echo "graph LR; A-->B-->C" | meraid

# 选择主题调色板（输出到终端时显示彩色）
meraid diagram.mmd --theme neon

# 通过管道强制彩色，例如送入分页器
meraid diagram.mmd --theme neon --color always | less -R

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

以下输出块均由二进制本身生成。

### 流程图

````mermaid
graph LR
    A[开始] --> B[处理] --> C[完成]
````

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│   开始   │───▶│   处理   │───▶│   完成   │
│          │    │          │    │          │
└──────────┘    └──────────┘    └──────────┘
```

分支图会以菱形布局呈现：判定节点的各个分支分列主干两侧（一上一下），各自的连线标签标注在自己的分支上，分叉与汇合处使用真正的 `┤` / `┴` 接头绘制。

````mermaid
graph LR
    A[开始] --> B{是否通过?}
    B -->|是| C[保存]
    B -->|否| D[停止]
````

```
                                    ┌──────────┐
                                ┌是▶│   保存   │
                                │   │          │
┌──────────┐      ┌──────────┐  │   └──────────┘
│   开始   │─────▶│是否通过? │──┤
│          │      │          │  │   ┌──────────┐
└──────────┘      └──────────┘  └否▶│   停止   │
                                    │          │
                                    └──────────┘
```

- **节点形状**会被解析 —— 矩形 `[文本]`、圆角 `(文本)`、菱形 `{文本}`、体育场 `([文本])`、子程序 `[[文本]]` 等。0.2 中所有节点统一绘制为方框，专属形状字形计划在 0.3 实现。
- **连线标签** `-->|文本|` 会标注在分支上。
- **连线样式** `-->`（实线）、`-.->`（虚线）、`==>`（粗线）会被解析；虚线/粗线的差异化渲染计划在 0.3 实现。
- **方向** `LR`、`RL`、`TD`/`TB`、`BT` 会被解析。当前无论声明哪个方向，布局都按从左到右排布，方向感知布局计划在 0.3 实现。

### 时序图

````mermaid
sequenceDiagram
    participant 用户A
    participant API服务
    用户A->>API服务: 查询 user-详情
    API服务-->>用户A: 返回 成功OK
````

```
   用户A            API服务

      │                 │

      ├─────────────────▶ 查询 user-详情
      ◀┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤ 返回 成功OK
```

- **消息类型：** 实线箭头 `->>`，虚线箭头 `-->>`（渲染为虚线）。
- **参与者：** 通过 `participant` / `actor` 声明，或从消息中推断。

### 类图

````mermaid
classDiagram
    class 用户 {
        +String 姓名
        +String 邮箱
        +登录()
        +注销()
    }
    class 管理员 {
        +String 权限级别
        +管理用户()
    }
    用户 <|-- 管理员
````

```
┌────────────────┐
│      用户      │
├────────────────┤
│+String 姓名    │
│+String 邮箱    │
├────────────────┤
│+登录()         │
│+注销()         │
└────────────────┘

┌────────────────┐
│     管理员     │
├────────────────┤
│+String 权限级别│
├────────────────┤
│+管理用户()     │
└────────────────┘

用户 <|-- 管理员
```

- **成员：** 属性与方法，支持可见性（`+` 公有，`-` 私有，`#` 受保护，`~` 包级）。字段与方法之间以分隔线区分。
- **关系：** `<|--`、`*--`、`o--`、`--|>`、`..>`、`..|>` 以及普通关联 `--` 会被解析，并在方框下方以文字图例展示。

### 状态图

````mermaid
stateDiagram-v2
    [*] --> 待处理
    待处理 --> 处理中: 开始 job-1
    处理中 --> 已完成: 完成 OK
    已完成 --> [*]
````

```
● ──▶ 待处理
待处理 ──▶ 处理中 : 开始 job-1
处理中 ──▶ 已完成 : 完成 OK
已完成 ──▶ ◉
```

- `[*]` 渲染为起始（`●`）或终止（`◉`）标记。
- 显示转换标签（`: 文本`）。
- 暂不支持复合/嵌套状态。

### 饼图

````mermaid
pie title 领养的宠物
    "狗" : 386
    "猫" : 85
    "鼠" : 15
````

```
狗┃████████████████████████████████ 79.4%
猫┃███████ 17.5%
鼠┃█ 3.1%
```

### ER 图

````mermaid
erDiagram
    CUSTOMER {
        int id PK
        string name
    }
    ORDER {
        int id PK
        int customer_id FK
    }
    CUSTOMER ||--o{ ORDER : places
````

```
┌────────────────────┐
│      CUSTOMER      │
├────────────────────┤
│PK    : id          │
│      : name        │
└────────────────────┘

┌────────────────────┐
│       ORDER        │
├────────────────────┤
│PK    : id          │
│   FK : customer_id │
└────────────────────┘

CUSTOMER ||--o{ ORDER
```

**基数符号：** `||` 恰好一个，`}|`/`|{` 一个或多个，`o|`/`|o` 零个或一个，`o{`/`}o` 零个或多个。**属性标记：** `PK` 主键，`FK` 外键。关系以文字图例展示在实体框下方。

## CJK 与 Unicode 对齐

meraid 使用 [`unicode-width`](https://crates.io/crates/unicode-width)（遵循 Unicode 标准附录 #11）计算每个字符的**显示宽度**，而非字节数或代码点数。这保证了中日韩文字在终端中的边框对齐精确无误。

| 字符类型 | 示例 | 显示宽度 |
|----------|------|----------|
| ASCII 字母 | `A` | 1 列 |
| CJK 全角字符 | `中` `文` `字` | 2 列 |
| 半角片假名 | `ｱ` | 1 列 |
| 全角标点 | `，` `。` | 2 列 |

所有渲染路径（节点边框、成员列表、居中对齐）均通过显示宽度计算填充量，因此混排中英文时边框不会偏移。示例文件位于 [`examples/`](examples/) 目录：

```bash
meraid examples/cjk-flowchart.mmd
meraid examples/cjk-sequence.mmd
meraid examples/cjk-class.mmd
```

## CLI 选项

| 参数 | 描述 |
|------|------|
| `[INPUT]` | 输入文件路径，`-` 或省略表示从 stdin 读取 |
| `--ascii` / `-a` | ASCII 纯文本边框（无 Unicode 线框字符） |
| `--theme <主题>` | 主题调色板：`default` `terra` `neon` `mono` `amber` `phosphor`。`default` 沿用终端自身颜色，其余主题按角色重新着色。 |
| `--color <时机>` | 何时输出 ANSI 彩色：`auto`（默认 —— 仅在终端）、`always`、`never`。遵循 `NO_COLOR`；`--color always` 可覆盖它。JSON 输出始终为无色。 |
| `--format <格式>` | 输出格式：`text`（默认）或 `json` |
| `--padding-x <N>` / `--padding-y <N>` | 预留的内边距选项（已接受参数，但尚未生效） |

## 主题

可通过 `--theme` 选择 6 套主题调色板：

| 主题 | 预期风格 |
|------|------|
| `default` | 默认终端颜色 |
| `terra` | 暖色复古风（棕色、橙色） |
| `neon` | 赛博朋克风（洋红、翠绿） |
| `mono` | 灰度单色 |
| `amber` | 琥珀色 CRT 风格 |
| `phosphor` | 经典绿色荧光管终端 |

彩色为**仅前景**，并按角色着色（节点文字、连线、连线标签、起止标记）。当终端
通过 `COLORTERM=truecolor`/`24bit` 声明支持时输出 **truecolor**，否则回退到
**256 色**。`default` 主题沿用终端自身颜色，无论是否启用彩色看起来都一致 ——
要重新着色请选择其他主题。背景填充预留给后续版本。

## 路线图

0.2 已交付：

- [x] ER 图
- [x] 流程图节点形状/标签的正确解析
- [x] 分支感知的流程图布局（菱形分布、分支标签、接头）
- [x] 对非法输入返回可被机器解析的诚实错误

计划于 0.3 及之后：

- [x] 主题调色板的 ANSI **彩色**输出（truecolor / 256 色，感知 TTY，`--color`
  参数，遵循 `NO_COLOR`）
- [ ] 节点形状**字形**（菱形、体育场、圆角……）
- [ ] 方向感知布局（`TD`/`BT`/`RL`）
- [ ] 虚线/粗线连线样式的差异化渲染
- [ ] 复合状态；时序图备注与激活
- [ ] 更多主题（gruvbox、monokai、dracula、nord、solarized）
- [ ] 自动适应终端宽度
- [ ] 交互式 TUI 查看器

## 贡献

欢迎贡献！详见 [CONTRIBUTING.md](CONTRIBUTING.md)。简要流程：

1. Fork 本仓库并创建功能分支。
2. 完成修改并补充测试。
3. 运行 `cargo fmt`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
4. 打开 Pull Request。

## 致谢

灵感来源：[termaid](https://github.com/fasouto/termaid) by fasouto

## 许可证

MIT 许可证 — 详见 [LICENSE](LICENSE)。

---

<p align="center">用 ❤️ Rust 打造</p>
