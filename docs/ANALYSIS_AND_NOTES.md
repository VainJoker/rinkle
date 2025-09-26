# Rinkle 项目理解、实现步骤与见解（2025-09-01）

本文档基于当前仓库状态与 `docs/SPECIFICATION.md`、`docs/IMPLEMENTATION_PLAN.md` 的内容，对项目目的、范围、实现路径进行梳理，并提出可执行的落地建议、设计取舍、潜在风险与问题清单。

## 项目目的与定位

- 核心："symlink farm manager"，面向 dotfiles 等配置，以声明式 `rinkle.toml` 为中心，将分散在多个目录的配置“汇聚”为单一目录树中的符号链接，提供 link/remove/status 等命令式体验。
- 用户画像：
  - 新手：希望快速初始化与引导（`rinkle init`）。
  - 进阶用户：需要 profile、版本选择（VSC）、OS 限定、冲突策略等高级能力。
- 非目标：模板渲染、机密管理。

## 现状快照（仓库状态）

- Cargo 元信息完整（edition = 2024，lints 配置齐全），但 `dependencies` 为空。
- `src/main.rs` 仅占位打印，未实现 CLI。
- `docs/` 内已有详细的规范与实施计划（中英混合），清晰可依。
- `config/rinkle.toml` 为空（样例/占位）。
- 测试仅有一个 `healthy_test.rs`（恒真）。

## 架构与模块化建议（Contract）

输入：
- 配置文件 `rinkle.toml`（路径可由 CLI 覆盖）。
- 可选状态文件 `~/.config/rinkle/state.toml`。

输出：
- 文件系统层面的符号链接行为与状态报告。
- 标准输出上的诊断与结果；非 0 退出码代表失败。

错误模式：
- 配置解析失败、路径不存在/权限不足、冲突策略需要交互但处于非交互模式、Windows/WSL 的符号链接权限差异等。

成功判据：
- 按 profile/OS/tag/VSC 解析得到的包集合被正确 link/remove；status 能稳定识别 4 类状态（正常/损坏/非链接/缺失）；所有命令在 dry-run 下无副作用。

## 分阶段落地步骤（结合现有实施文档微调）

里程碑 1：配置与状态
- 引入依赖：clap、serde、serde_with、toml、thiserror、tracing（或 log+env_logger）、dirs（定位配置目录）、regex、anyhow/eyre（二选一）
- 定义配置模型：Global、Vsc、Profiles、Package、Config；State。
- Loader：load_config(path) 与 load_state()/save_state()；合并优先级（CLI > state > file > defaults）。
- 单元测试：针对最小/错误/完整样例。

里程碑 2：核心链接
- Link 引擎：路径展开（~、env）、源/目标计算、冲突策略（skip/overwrite/backup/prompt）、原子性（先写临时名再替换）、幂等性与 dry-run。
- Status 扫描：遍历包目标路径，分类输出；为 TTY/非 TTY 保持简洁输出。
- remove：仅删除我们创建的链接（校验链接指向）。

里程碑 3：Profiles 与 VSC
- Profile 解析：根据 active_profile（state）或 profiles.default，筛选包。
- VSC：根据 vsc.template 提取版本目录，支持 package@version 覆盖，default_version 与 state.pinned_versions 融合。
- 命令 `vsc <pkg> <ver>`：更新 pinned_versions。

里程碑 4：体验
- init：交互式创建基础 `rinkle.toml`，或基于 git 仓库。
- interactive：MVP REPL（可择后）。
- UI：使用 indicatif 打点与进度（可延后）。

里程碑 5：监控
- notify 监听 source_dir，节流与批处理，守护进程模式（start/stop）。

里程碑 6：收尾
- 完善 README、CHANGELOG、rustdoc，增加集成/端到端测试；准备发布。

## 数据结构草案（最小可用）

- Config
  - global: { source_dir: PathBuf, target_dir: PathBuf, conflict_strategy: enum, ignore: Vec<String>, dry_run: bool? }
  - vsc: { template: String, default_version: String }
  - profiles: { default: Vec<String>, [name]: Vec<String> }
  - packages: HashMap<String, Package>
- Package
  - source: Option<String>
  - target: Option<String>
  - os: Option<Vec<String>>
  - tags: Vec<String>
  - default_version: Option<String>
- State
  - active_profile: Option<String>
  - pinned_versions: HashMap<String, String>

说明：source/target 可相对/绝对；最终解析阶段统一归一化。

## 关键技术点与取舍

- 平台支持：当前仅支持 Linux/macOS；暂不支持 Windows（不提供 fallback）。
- 路径展开：支持 ~、$ENV、相对路径；统一用 shellexpand + dunce/canonicalize。
- 冲突策略：
  - backup 命名：<name>.bak.<timestamp>，避免覆盖；提供回滚信息；不提供一键清理命令。
  - prompt 需交互：在非 TTY 下自动降级为 skip 或报错（可通过 flag 指定）。
- 幂等性：目标是已存在且正确指向同一源 -> 直接成功；否则根据策略处理。
- 版本解析：基于 regex 的命名模板；引入版本目录缓存以提速（内存/可选磁盘）。
- 过滤：tags/profiles/os。OS 值：linux/macos/windows，运行时判定当前 OS。
- Dry-run：对文件操作打印计划，不落盘。
- 日志：tracing + RUST_LOG；用户友好输出与 debug 日志分离；state 读写需加文件锁避免并发损坏。

## CLI 草图（clap）

- `rinkle link [pkg@ver]... [--all] [--profile <name>] [--dry-run] [--config <path>]`
- `rinkle remove [pkg]... [--all]`
- `rinkle status [--profile <name>] [--json]`
- `rinkle vsc <pkg> <ver>`
- `rinkle use-profile <name>`
- `rinkle init [git_repo] [--dest <path>]`
- `rinkle start|stop`

## 测试建议

- 单元：配置解析（含默认值）、路径展开、冲突策略（在临时目录内）、VSC 解析。
- 集成：在 `tests/` 下用 tempdir 构造多包与 profile，实际创建与校验链接。
- 端到端（可选）：通过 assert_cmd 调起二进制，校验输出与副作用。

## 最小依赖建议

- clap = "^4"
- serde = { version = "^1", features = ["derive"] }
- toml = "^0.8"
- thiserror = "^1"
- tracing = { version = "^0.1", features = ["env-filter"] }
- dirs = "^5"
- regex = "^1"
- shellexpand = "^3"
- fs_err = "^2"
- anyhow 或 color-eyre（二选一，看团队偏好）
- indicatif（可选，后期）
- notify（监控阶段再加）

## 风险与未决问题清单

1. （已决）Windows 暂不支持。
2. （已决）不提供 fallback（仅支持 symlink）。
3. （已决）不提供一键清理命令。
4. ignore 规则：glob 语义如何定义（gitignore 风格/标准 glob）？
5. prompt 的 UX：在脚本环境如何避免阻塞？
6. 多 profile/tag 的选择关系：`profiles` 中是 tags 还是包名？当前文档示例为 tags，需确认解析逻辑。
7. （已决）VSC 目录扫描需要缓存以提速。
8. `status --json` 是否需要稳定 schema 以便集成到其他工具？
9. （已决）state 文件需要文件锁。
10. 性能目标与指标：可后续基准测试；暂不考虑与其他工具的继承/导入。

## 下一步可执行工作（建议）

- 引入依赖并实现 `src/cli.rs`、`src/config/{mod.rs,loader.rs,entity.rs}`、`src/state.rs`、`src/linker.rs` 的骨架；
- 用最小 happy-path 流程打通：读取配置 -> 解析 1 个包 -> 计算目标 -> dry-run 打印计划；
- 在 `tests/` 新增 1-2 个集成测试覆盖 dry-run 输出与路径展开；
- 在 README 增加示例配置片段与快速开始；
- 待功能稳定后，再推进 VSC 与 profiles。

--
文档作者：自动化分析助手（基于当前仓库状态生成）
