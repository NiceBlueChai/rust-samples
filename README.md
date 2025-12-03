# Rust Samples

一个 Rust 学习和实践的代码示例仓库，包含各种 Rust 示例代码和独立项目。

## 📁 仓库结构

```
rust-samples/
├── crates/              # Cargo 工作空间成员
│   └── examples/        # Rust 示例代码集合
│       └── examples/    # 可执行示例
└── standalone/          # 独立项目（不在工作空间中）
    └── tauri-app/       # Tauri + Vue 3 禅道Bug管理系统
```

## 📚 示例列表

### 基础示例 (crates/examples)

| 示例名称 | 说明 | 相关技术 |
|---------|------|---------|
| `hello_world` | 经典的 Hello World 程序 | Rust 基础 |
| `async_basic` | 异步编程基础示例 | Tokio 异步运行时 |
| `ascii_generator` | 图片转 ASCII 字符画生成器 | 图像处理、命令行参数解析 |

### 独立项目 (standalone/)

- **tauri-app**: 一个功能完整的禅道Bug管理桌面应用
  - 技术栈：Tauri + Vue 3 + TypeScript + Element Plus
  - 详细说明请参考 [standalone/tauri-app/README.md](./standalone/tauri-app/README.md)

## 🚀 快速开始

### 环境要求

- Rust 1.91+ (2024 edition)
- Cargo

### 安装 Rust

如果还没有安装 Rust，请访问 [https://rustup.rs/](https://rustup.rs/) 安装。

### 运行示例

```bash
# 克隆仓库
git clone https://github.com/NiceBlueChai/rust-samples.git
cd rust-samples

# 运行 Hello World 示例
cargo run --example hello_world

# 运行异步示例
cargo run --example async_basic

# 运行 ASCII 生成器（需要提供图片）
cargo run --example ascii_generator -- --input /path/to/image.jpg --width 100
```

### 构建所有示例

```bash
# 构建工作空间中的所有 crate
cargo build

# 构建并运行测试
cargo test
```

## 🛠️ 工作空间依赖

本项目使用 Cargo 工作空间管理依赖，常用依赖在根 `Cargo.toml` 中统一定义：

- `serde` & `serde_json` - 序列化/反序列化
- `tokio` - 异步运行时
- `anyhow` & `thiserror` - 错误处理
- `tracing` & `tracing-subscriber` - 日志和追踪
- `clap` - 命令行参数解析
- `image` - 图像处理

成员 crate 通过 `.workspace = true` 引用这些依赖。

## 📖 学习资源

- [The Rust Programming Language](https://doc.rust-lang.org/book/) - Rust 官方书籍
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - 通过示例学习 Rust
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Tokio 异步编程教程

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。
