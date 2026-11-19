# GitHub Release CI/CD 设计

## 目标

推送版本标签后，自动构建 Nimbo Terminal 的 Windows 与 macOS 安装包，并将全部平台产物放到同一个 GitHub Release。

## 触发与版本

- 工作流由推送 `v*` 标签触发，例如 `v0.1.0`。
- 标签版本应与 `package.json`、`src-tauri/tauri.conf.json` 和 `src-tauri/Cargo.toml` 中的应用版本一致；工作流不自动改版本。
- 工作流有 `contents: write` 权限，用于创建 Release 和上传安装包。

## 构建与发布

- 使用 Tauri 官方 `tauri-action` 在 Windows x64、macOS Intel (`x86_64-apple-darwin`) 与 Apple Silicon (`aarch64-apple-darwin`) 三个 runner 构建。
- 使用仓库锁文件通过 pnpm 安装前端依赖；Tauri 配置负责构建前端并生成平台 bundle。
- 以触发标签为 Release 标签，工作流创建一个正式 GitHub Release，并将三个构建目标的安装包上传到该 Release。
- macOS 构建使用 ad-hoc 签名，不要求仓库 Secret。它不会替代 Apple Developer ID 签名和 notarization；用户首次打开下载的 macOS 应用时，仍可能需要在系统隐私与安全设置中手动允许。

## 验收范围

- YAML 语法与 GitHub Actions 配置有效。
- 前端生产构建成功，Tauri capability / Rust 配置可检查。
- 工作流发布资产包含 Windows 安装包以及 macOS Intel、Apple Silicon 安装包。
- 不在每次普通分支推送时发布正式 Release。
