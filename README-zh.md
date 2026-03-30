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
- **5+ 图表类型** — 流程图、时序图、类图、状态图、饼图、ER 图
- **6 套主题** — default, terra, neon, mono, amber, phosphor
- **CJK 对齐** — 基于 `unicode-width` 精确计算显示宽度，中日韩字符边框对齐无偏差
- **ASCII 回退** — 兼容任何终端
- **管道友好 CLI** — `cat diagram.mmd | meraid` 即刻使用

## 为什么选择 Meraid？

Mermaid 是文档编写的神器，但渲染它通常需要浏览器或外部服务。Meraid 让 Mermaid 渲染直接进入终端 — 非常适合 SSH 会话、CI 日志、TUI 应用或任何有 Rust 的环境。

为 Rust 生态圈而生，提供快速、零依赖的替代方案。

## 安装

### 从 Git 安装（推荐 - 最新版本）

```bash
cargo install --git https://github.com/Binlogo/meraid.git
```

### 从源码构建

```bash
git clone https://github.com/Binlogo/meraid.git
cd meraid
cargo build --release
cargo install --path .
```

### 从 Crates.io 安装（即将支持）

```bash
cargo install meraid
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

**节点形状：** 矩形 `[文本]`，圆角 `(文本)`，菱形 `{文本}`，体育场 `([文本])`，子程序 `[[文本]]`

**连线样式：** 实线 `-->`，虚线 `-.->`，粗线 `==>`，带标签 `-->|文本|`

### 时序图

````mermaid
sequenceDiagram
    participant 用户A
    participant API服务
    用户A->>API服务: 查询 user-详情
    API服务-->>用户A: 返回 成功OK
````

```
用户A              API服务
  │                   │
  ├─────────────────▶ 查询 user-详情
  ◀─────────────────────────────────┤ 返回 成功OK
```

**消息类型：** 实线箭头 `->>`，虚线箭头 `-->>`

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
│+登录()         │
│+注销()         │
└────────────────┘

┌─────────────────┐
│     管理员      │
├─────────────────┤
│+String 权限级别 │
│+管理用户()      │
└─────────────────┘

用户 ◄───
```

**关系：** 继承 `<|--`，组合 `*--`，聚合 `o--`，关联 `--`

**成员：** 属性和方法，支持可见性（`+` 公有，`-` 私有，`#` 受保护）

### 状态图

````mermaid
stateDiagram-v2
    [*] --> 待处理
    待处理 --> 处理中: 开始 job-1
    处理中 --> 已完成: 完成 OK
    已完成 --> [*]
````

```
● ──▶ 待处理 :
待处理 ──▶ 处理中 : 开始 job-1
处理中 ──▶ 已完成 : 完成 OK
已完成 ──▶ ◉ :
```

### 饼图

````mermaid
pie title 领养的宠物
    "狗" : 386
    "猫" : 85
    "鼠" : 15
````

```
狗┃████████████████████████████████  79.4%
猫┃███████  17.5%
鼠┃█  3.1%
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

**基数符号：** `||` 恰好一个，`}|` 一个或多个，`o|` 零个或一个，`o{` 零个或多个

## CJK 与 Unicode 对齐

meraid 使用 [`unicode-width`](https://crates.io/crates/unicode-width)（遵循 Unicode 标准附录 #11）计算每个字符的**显示宽度**，而非字节数或代码点数。这保证了中日韩文字在终端中的边框对齐精确无误。

### 对齐原理

| 字符类型 | 示例 | 显示宽度 |
|----------|------|----------|
| ASCII 字母 | `A` | 1 列 |
| CJK 全角字符 | `中` `文` `字` | 2 列 |
| 半角片假名 | `ｱ` | 1 列 |
| 全角标点 | `，` `。` | 2 列 |

所有渲染路径（节点边框、成员列表、居中对齐）均通过显示宽度计算填充量，而非 Rust 的字符计数，因此混排中英文时边框不会偏移。

### CJK 示例

示例文件位于 [`examples/`](examples/) 目录：

```bash
# 中文流程图
meraid examples/cjk-flowchart.mmd

# 中文时序图
meraid examples/cjk-sequence.mmd

# 中文类图（边框对齐效果最佳）
meraid examples/cjk-class.mmd
```

中文类图渲染效果：

```
┌────────────────┐
│      用户      │
├────────────────┤
│+String 姓名    │
│+String 邮箱    │
│+int 年龄       │
│+登录()         │
│+注销()         │
└────────────────┘
```

注意每行末尾的空格填充均基于**显示宽度**计算：`+String 姓名` 显示宽 12，`+登录()` 显示宽 7，补充至 box_width 后右侧边框 `│` 精确对齐。

## CLI 选项

```
meraid [OPTIONS] [INPUT]
```

| 参数 | 默认值 | 描述 |
|------|--------|------|
| `[INPUT]` | stdin | 输入文件路径，`-` 表示从 stdin 读取 |
| `--theme <主题>` | `default` | 颜色主题：`default` `terra` `neon` `mono` `amber` `phosphor` |
| `--ascii` / `-a` | 关 | ASCII 纯文本边框（无 Unicode 线框字符） |
| `--padding-x <N>` | `4` | 节点内水平边距 |
| `--padding-y <N>` | `2` | 节点内垂直边距 |
| `--format <格式>` | `text` | 输出格式：`text` 或 `json`（JSON 为 AI 友好模式） |

`--help` 会列出所有主题和格式的可选值，无需查阅文档。

## 主题

6 套内置主题：

| 主题 | 描述 |
|------|------|
| `default` | 默认终端颜色 |
| `terra` | 暖色复古风（棕色、橙色） |
| `neon` | 赛博朋克风（洋红、翠绿） |
| `mono` | 灰度单色 |
| `amber` | 琥珀色 CRT 风格 |
| `phosphor` | 经典绿色荧光管终端 |

## 路线图

- [x] 流程图 ✅
- [x] 时序图 ✅
- [x] 类图 ✅
- [x] 状态图 ✅
- [x] 饼图 ✅
- [x] ER 图 ✅
- [x] CJK 字符精确对齐 ✅
- [ ] 块图
- [ ] Git 提交图
- [ ] 思维导图
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
