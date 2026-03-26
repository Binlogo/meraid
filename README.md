# Meraid 🦈

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
</p>

> 在终端中渲染 Mermaid 图表。一个用 Rust 编写的轻量级实现，受 [termaid](https://github.com/fasouto/termaid) 启发。

## ✨ 特性

- 🚀 **纯 Rust 实现** - 零外部依赖，快速且可移植
- 🎨 **6 套主题** - default, terra, neon, mono, amber, phosphor
- 📝 **支持多种图表** - 流程图、时序图、状态图、类图、饼图
- 🔤 **ASCII 模式** - 兼容任何终端
- ⌨️ **管道友好** - 支持 stdin 输入
- 🌈 **彩色输出** - 完整的 ANSI 颜色支持

## 📦 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/Binlogo/meraid.git
cd mermaID

# 构建
cargo build --release

# 可执行文件位于
./target/release/meraid --help
```

### 安装到本地

```bash
cargo install --path .
```

### 使用 Homebrew

```bash
# 即将支持
brew install mermaID
```

## 🎯 快速开始

```bash
# 从文件渲染
meraid diagram.mmd

# 从 stdin 输入
echo "graph LR; A --> B --> C" | mermaID

# 指定主题
meraid diagram.mmd --theme neon

# 纯 ASCII 输出
meraid diagram.mmd --ascii

# 尖锐边缘（无圆角）
meraid diagram.mmd --sharp-edges
```

## 📖 使用示例

### 流程图

```bash
meraid - <<'EOF'
graph TD
    A[Start] --> B{Is it?}
    B -->|Yes| C[OK]
    B -->|No| D[Stop]
    C --> D
EOF
```

### 时序图

```bash
meraid - <<'EOF'
sequenceDiagram
    participant Alice
    participant Bob
    Alice->>Bob: Hello Bob
    Bob-->>Alice: Hi Alice
EOF
```

### 状态图

```bash
meraid - <<'EOF'
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: start
    Processing --> Done: complete
    Done --> [*]
EOF
```

### 饼图

```bash
meraid - <<'EOF'
pie title Pets
    "Dogs" : 386
    "Cats" : 85
    "Rats" : 15
EOF
```

## 🎨 主题预览

| 主题 | 描述 | 适用场景 |
|------|------|----------|
| `default` | 青色节点，黄色箭头 | 默认，适合大多数终端 |
| `terra` | 暖色调（棕、橙） | 复古风格 |
| `neon` | 洋红节点，绿色箭头 | 赛博朋克风格 |
| `mono` | 灰度单色 | 简单终端 |
| `amber` | 琥珀色 CRT 风格 | 老式显示器 |
| `phosphor` | 绿色荧光管风格 | 经典终端 |

## ⚙️ 配置

### 命令行选项

```
USAGE:
    mermaID [OPTIONS] [INPUT]

ARGS:
    <INPUT>    Mermaid 文件路径，或使用 - 表示 stdin

OPTIONS:
    -t, --theme <THEME>    选择主题 [default: default]
    -a, --ascii           使用 ASCII 模式（无彩色）
    -s, --sharp-edges     使用尖锐边缘
    -w, --width <WIDTH>    最大输出宽度 [default: 120]
    -h, --help            显示帮助信息
    -V, --version         显示版本信息
```

### 环境变量

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `MERAID_THEME` | 默认主题 | `default` |
| `MERAID_WIDTH` | 默认宽度 | `120` |

## 🏗️ 项目架构

```
src/
├── lib.rs        # 库入口，暴露主要接口
├── diagram.rs    # 图表数据结构定义
├── parser.rs     # Mermaid 语法解析器
├── layout.rs     # 基于网格的布局引擎
├── render.rs     # 终端渲染器
├── theme.rs      # 主题定义
└── main.rs       # CLI 入口
```

### 核心模块

- **parser** - 将 Mermaid 语法转换为中间表示
- **diagram** - 图表的通用数据结构
- **layout** - 计算节点和边的位置
- **render** - 将布局渲染为终端输出
- **theme** - 颜色和样式定义

## 🛣️ 路线图

- [x] 改进流程图边路由（曼哈顿风格）
- [x] 添加类图成员渲染
- [ ] 添加 ER 图支持
- [ ] Git 提交图渲染
- [ ] 块图支持
- [ ] 树图可视化
- [ ] 添加测试和性能基准

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing`)
5. 创建 Pull Request

## 📄 许可证

MIT License - 查看 [LICENSE](LICENSE) 获取详情。

## 🙏 致谢

- [termaid](https://github.com/fasouto/termaid) - 灵感来源
- [Mermaid](https://mermaid.js.org/) - 语法规范
- [crossterm](https://github.com/crossterm-rs/crossterm) - 终端操作库

---

<p align="center">用 ❤️ 构建 via Rust</p>
