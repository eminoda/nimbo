# GitHub Release CI Implementation Plan

> **For agentic workers:** Execute the checked steps in order in the current repository. The user requested inline implementation and automatic builds on pushes to any branch.

**Goal:** Build Windows x64 and macOS Intel/Apple Silicon installers on every branch push, and publish those same installers to GitHub Releases on `v*` tag pushes.

**Architecture:** A read-only matrix build job installs the locked pnpm dependencies, installs the matching Rust target, runs the Tauri production build, prefixes and uploads installer files as Actions artifacts. A dependent publish job runs only for version tags, downloads all matrix artifacts, and uses the job-scoped write token to create or update the GitHub Release.

**Tech Stack:** GitHub Actions, pnpm 10.32.0, Node.js 22, stable Rust, Tauri 2 CLI, GitHub CLI.

---

### Task 1: Add the desktop build and release workflow

**Files:**
- Create: `.github/workflows/desktop-build.yml`
- Create: `scripts/collect-tauri-installers.mjs`
- Create: `scripts/validate-release-version.mjs`
- Modify: `docs/superpowers/specs/2026-11-19-github-release-ci-design.md`

- [x] **Step 1: Trigger desktop builds on all branches and version tags**

Set workflow triggers to all branch pushes (`push.branches: ["**"]`) and `push.tags: ["v*"]`. Keep workflow-level contents permission at read-only. On tags, compare the tag version with `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` before building.

- [x] **Step 2: Build all supported desktop targets in parallel**

Use a matrix for `windows-latest` with `x86_64-pc-windows-msvc`, and `macos-latest` with `x86_64-apple-darwin` and `aarch64-apple-darwin`. Install Node 22, pnpm 10.32.0, stable Rust and the matrix target; install dependencies with `pnpm install --frozen-lockfile`; run `pnpm tauri build --target <matrix target>`. Set `APPLE_SIGNING_IDENTITY=-` for macOS builds.

- [x] **Step 3: Upload uniquely named installer artifacts**

Collect `.exe`, `.msi`, `.dmg`, and `.app.tar.gz` files from the matching target bundle directory. Prefix each filename with `nimbo-<platform>-<architecture>-` and upload one Actions artifact per matrix entry. Fail the build if no installer files are found.

- [x] **Step 4: Publish only version-tag builds**

Add a job depending on the full build matrix. Guard it with `startsWith(github.ref, 'refs/tags/v')`, grant only this job `contents: write`, download all `nimbo-*` artifacts, create the release if missing, and use `gh release upload --clobber` so rerunning a tag replaces assets instead of failing.

### Task 2: Validate and publish the change

**Files:**
- Verify: `.github/workflows/desktop-build.yml`

- [x] **Step 1: Validate local project builds and workflow syntax**

Run `pnpm build`, `cargo check` from `src-tauri`, `git diff --check`, and an Action workflow linter when available. Review that release permissions are scoped to the publish job and all three matrix artifacts are included.

- [x] **Step 2: Commit and push the workflow**

Commit the design update, plan, installer collection script, and workflow with a message describing cross-platform push builds and tagged releases. Push to `main` so GitHub Actions starts the first build automatically.
