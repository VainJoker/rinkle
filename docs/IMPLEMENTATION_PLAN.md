# Rinkle - 项目实施路线图 (V1.0)

本文档将 `rinkle` 的开发过程分解为一系列清晰、可执行的里程碑和步骤。

#### **里程碑 1: 项目基础与配置解析 (The Foundation)**

*目标：搭建项目，并让程序能够完全理解 `rinkle.toml` 配置文件。这是所有功能的地基。*

1.  **步骤 1.1: 初始化项目与依赖**
    *   [ ] 使用 `cargo new rinkle` 创建项目。
    *   [ ] 在 `Cargo.toml` 中添加核心依赖：
        *   `clap`: 用于解析命令行参数。
        *   `serde`: 用于序列化/反序列化数据。
        *   `toml`: 用于解析 `.toml` 文件。
    *   `tracing` + `tracing-subscriber`: 用于分层日志记录（替代 log）。
        *   `thiserror`: 用于优雅的错误处理。
    *   `fs4` 或 `fd-lock`: 用于文件锁（state.toml 访问）。

2.  **步骤 1.2: 定义配置数据结构**
    *   [ ] 在 `src/config/mod.rs` 和 `src/config/entity.rs` 中，创建与 `rinkle.toml` 结构完全对应的 Rust `struct`s。
    *   [ ] 为所有 `struct` 派生 `serde::Deserialize`，以便从 TOML 文件中加载。
    *   [ ] 详细定义 `Global`, `Vsc`, `Profiles`, `Package` 等每一个结构体。

3.  **步骤 1.3: 实现配置加载器**
    *   [ ] 创建一个模块 `src/config/loader.rs`。
    *   [ ] 实现一个函数 `load_config(path: &Path) -> Result<Config, Error>`，它负责：
        *   读取指定路径的 `rinkle.toml` 文件。
        *   使用 `toml::from_str` 将文件内容解析到我们定义的 `Config` 结构体中。
        *   处理文件读取和解析可能发生的错误。

4.  **步骤 1.4: 实现状态加载器（带文件锁）**
    *   [ ] 类似地，定义 `state.toml` 对应的 `State` 结构体。
    *   [ ] 实现 `load_state()` 和 `save_state()` 函数，用于读取和写入 `~/.config/rinkle/state.toml`，且在读/写时获取文件锁，避免并发损坏。

#### **里程碑 2: 核心链接逻辑 (The Core Logic)**

*目标：实现最核心的链接、移除和状态检查功能。*

1.  **步骤 2.1: 链接引擎 (Linker Engine)**
    *   [ ] 创建 `src/rinkle.rs` 模块。
    *   [ ] 实现一个核心函数 `link_package(package: &Package, config: &Config)`。
    *   [ ] 在此函数中，实现 `conflict_strategy`（`skip`, `overwrite`, `backup`, `prompt`）的逻辑；不提供自动 fallback（复制/硬链）——出现问题让用户手动处理。
    *   [ ] 处理文件和目录的符号链接创建。

2.  **步骤 2.2: 实现 `status` 命令**
    *   [ ] 添加 `status` 子命令到 `src/cli.rs`。
    *   [ ] 实现逻辑：遍历所有已配置的包，检查其目标路径的链接状态（正常链接、链接损坏、非链接文件、不存在）。
    *   [ ] 设计状态信息的美观输出格式。

3.  **步骤 2.3: 实现 `link` 和 `remove` 命令**
    *   [ ] 添加 `link` 和 `remove` 子命令到 `src/cli.rs`。
    *   [ ] 将 `link` 命令连接到链接引擎。
    *   [ ] 实现 `remove` 命令的逻辑，即安全地删除符号链接。暂不提供一键清除备份或残留的命令。

#### **里程碑 3: 高级功能 (Advanced Features)**

*目标：实现 Profiles 和版本选择，让工具变得“强大”。*

1.  **步骤 3.1: Profile 解析与应用**
    *   [ ] 实现逻辑：根据 `state.toml` 中记录的 `active_profile`，从 `rinkle.toml` 中筛选出当前激活的所有包。
    *   [ ] 如果没有 `active_profile`，则使用 `profiles.default`。
    *   [ ] 将 `link`, `status` 等命令的逻辑更新为只对当前 profile 的包生效。

2.  **步骤 3.2: 实现 `use-profile` 命令**
    *   [ ] 添加 `use-profile` 子命令。
    *   [ ] 实现其逻辑：更新 `state.toml` 中的 `active_profile` 字段，并询问用户是否立即链接新的 Profile。

3.  **步骤 3.3: 版本选择 (VSC) 逻辑（含缓存）**
    *   [ ] 实现逻辑：在解析包时，根据 `vsc.template` 正则表达式识别版本化包。
    *   [ ] 根据 `state.toml` 中固定的版本或 `vsc.default_version` 来解析出正确的源文件路径。
    *   [ ] 缓存（内存/磁盘）已发现的版本目录，减少文件系统扫描开销，并提供过期策略或基于 mtime 的快速校验。
    *   [ ] 在 `link` 命令中支持 `package@version` 语法，以覆盖默认版本。

4.  **步骤 3.4: 实现 `vsc` 命令**
    *   [ ] 添加 `vsc` 子命令。
    *   [ ] 实现其逻辑：将用户为某个包选择的特定版本写入 `state.toml` 的 `[pinned_versions]` 表中。

#### **里程碑 4: 用户体验优化 (User Experience)**

*目标：实现新用户引导和交互模式，让工具变得“有趣”。*

1.  **步骤 4.1: 实现 `init` 命令**
    *   [ ] 添加 `init` 子命令。
    *   [ ] 实现克隆 Git 仓库的逻辑。
    *   [ ] 实现交互式问答，引导用户生成 `rinkle.toml`。

2.  **步骤 4.2: 实现 `interactive` 命令 (MVP)**
    *   [ ] 添加 `interactive` 子命令。
    *   [ ] 创建一个简单的 REPL (Read-Eval-Print Loop)，允许用户输入 `link`, `status` 等命令。

3.  **步骤 4.3: 美化输出**
    *   [ ] 全面应用 `log` 和 `global.ui` 中的设置。
    *   [ ] 使用 `colored` 或类似库为输出添加颜色。
    *   [ ] 为耗时操作（如链接多个包）添加 `indicatif` 风格的进度条。
    *   [ ] 在错误情况下不进行自动修复或回退，输出明确提示让用户手动处理。

#### **里程碑 5: 自动化 (Automation)**

*目标：实现后台监控功能。*

1.  **步骤 5.1: 文件监控逻辑**
    *   [ ] 引入 `notify` 库来监控文件系统事件。
    *   [ ] 实现一个循环，监听 `source_dir` 中的所有文件变动。
    *   [ ] 根据 `global.monitor_interval` 设置延迟执行，以避免短时间内大量重复操作。
    *   [ ] 该阶段仍不考虑与其他工具的互操作或导入路径。

2.  **步骤 5.2: 后台服务 (Daemon)**
    *   [ ] 实现 `start` 和 `stop` 命令。
    *   [ ] `start` 命令需要将监控进程作为后台守护进程运行。
    *   [ ] `stop` 命令需要能安全地终止后台进程。

#### **里程碑 6: 收尾工作 (Finalization)**

*目标：准备首次发布。*

1.  **步骤 6.1: 完善文档**
    *   [ ] 撰写 `README.md`，包含安装和使用指南；明确当前仅支持 Linux/macOS，不支持 Windows；无 `clean` 命令；无自动 fallback；
    *   [ ] 为所有公开的函数和结构体编写 `rustdoc` 注释。
    *   [ ] 创建 `CHANGELOG.md`。

2.  **步骤 6.2: 编写测试**
    *   [ ] 为核心逻辑（特别是配置解析和链接引擎）编写单元测试和集成测试。

3.  **步骤 6.3: 发布准备**
    *   [ ] 在 `Cargo.toml` 中完善元数据（`description`, `license`, `repository` 等）。
    *   [ ] 在 Crates.io 上发布。
