# GitHub Release CI/CD 设计

## 目标

推送代码到任意分支时自动构建 Nimbo Terminal 的 Windows 与 macOS 安装包；推送版本标签后，将全部平台产物放到同一个 GitHub Release。

## 触发与版本

- 推送到任意分支时自动构建 Windows x64、macOS Intel 与 Apple Silicon 安装包，并保存为 GitHub Actions artifacts。
- 推送 `v*` 标签时，例如 `v0.1.0`，执行同样的三套构建，构建全部成功后创建正式 GitHub Release 并上传所有安装包。
- 标签版本必须与 `package.json`、`src-tauri/tauri.conf.json` 和 `src-tauri/Cargo.toml` 中的应用版本一致；工作流会在构建前检查，不自动改版本。
- 普通构建仅需要 `contents: read`；发布 job 单独使用 `contents: write` 创建 Release 和上传安装包。

## 构建与发布

- 使用 Tauri CLI 在 Windows x64、macOS Intel (`x86_64-apple-darwin`) 与 Apple Silicon (`aarch64-apple-darwin`) 三个 runner 构建。
- 使用仓库锁文件通过 pnpm 安装前端依赖；Tauri 配置负责构建前端并生成平台 bundle。
- 构建产物以平台和架构作为文件名前缀，避免 macOS 两个架构产物重名。
- 发布 job 等待三个构建全部通过，再用 GitHub CLI 以触发标签创建或更新 Release，并上传安装包。
- macOS 构建使用 ad-hoc 签名，不要求仓库 Secret。它不会替代 Apple Developer ID 签名和 notarization；用户首次打开下载的 macOS 应用时，仍可能需要在系统隐私与安全设置中手动允许。

## 验收范围

- YAML 语法与 GitHub Actions 配置有效。
- 前端生产构建成功，Tauri capability / Rust 配置可检查。
- 推送代码到任意分支后的 Actions run 包含 Windows、macOS Intel 和 Apple Silicon 三组 artifacts。
- 推送 `v*` 标签后 Release 包含 Windows 安装包以及 macOS Intel、Apple Silicon 安装包。
- 普通分支推送不创建正式 Release。
