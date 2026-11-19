# Open Project in IDE Implementation Plan

> **For agentic workers:** Implement this approved design inline, task by task. Keep launcher discovery and UI responsibilities separate.

**Goal:** Add an “Open in IDE” project context-menu submenu for installed Codex Desktop, VS Code, and Cursor clients.

**Architecture:** A focused Rust editor module detects the three supported Windows clients and launches them through the registered Codex URI protocol or direct GUI executables. Tauri commands resolve project paths from saved settings and accept only a closed editor ID enum. `ProjectDock.vue` owns scan state and submenu interaction; the parent handles launch failures through the existing toast.

**Tech Stack:** Vue 3 Composition API + TypeScript, Tauri 2 commands/opener plugin, Rust standard process APIs, Windows StartApps/`where.exe`, existing Lucide icons.

---

## File Map

- Create `src-tauri/src/editors.rs`: editor IDs, Windows detection, path resolution, Codex deep-link URL construction, and safe process launch.
- Modify `src-tauri/src/commands.rs`: expose installed-editor list and project-ID scoped launch command.
- Modify `src-tauri/src/lib.rs`: register the editor module and commands.
- Create `src/types/editor.ts`: frontend editor ID and installed-editor types.
- Modify `src/services/tauri.ts`: typed scan and launch wrappers.
- Modify `src/components/ProjectDock.vue`: scan on context-menu open, nested editor submenu, refresh, and launch event.
- Modify `src/App.vue`: call launch API and surface errors through `showToast`.
- Modify `src/style.css`: nested menu positioning, states, and hover/focus styles.
- Modify `todo.md`: record the feature after verification.

## Task 1: Add safe editor discovery and launch support

**Files:** Create `src-tauri/src/editors.rs`; modify `src-tauri/src/lib.rs`.

- [x] Define a serializable and deserializable closed `EditorId` enum with stable JSON values `codex`, `vscode`, and `cursor`; define a serializable `{ id, name }` installed-editor item.
- [x] On Windows, detect Codex Desktop through the stable StartApps AppID prefix `OpenAI.Codex_*!App`; build a `codex://threads/new?path=<percent-encoded absolute path>` URL and launch it through `tauri_plugin_opener::OpenerExt`.
- [x] Resolve VS Code and Cursor launchers with `where.exe code` / `where.exe cursor`; for `.cmd` launchers walk parent directories to find `Code.exe` / `Cursor.exe`. Also check `%LOCALAPPDATA%\Programs` and `%ProgramFiles%` common install locations.
- [x] Launch VS Code with the project path as one argument. Launch Cursor with the project path and `--classic` as separate arguments. Use `Command::new(exe).args(...).spawn()`; never use `cmd /C` or concatenate the project path into a shell command.
- [x] Keep the detection order stable: Codex, VS Code, Cursor. Exclude editors that cannot be resolved to a launchable executable/protocol.
- [x] On non-Windows targets return an explicit unsupported error and an empty detected list.
- [x] Add `mod editors;` in the library root.

## Task 2: Expose project-scoped Tauri commands

**Files:** Modify `src-tauri/src/commands.rs`; modify `src-tauri/src/lib.rs`.

- [x] Add `list_installed_editors()` returning only detected `{ id, name }` entries.
- [x] Add `open_project_in_editor(state, project_id, editor_id)` that resolves `project_id` through saved settings, verifies the folder still exists, and delegates to the editor module.
- [x] Map unknown project IDs to `项目不存在`; map missing editor or launch errors to concise strings that the UI can show.
- [x] Register both commands in `tauri::generate_handler!`; do not add project JSON fields or new runtime permissions.

## Task 3: Add typed frontend wrappers

**Files:** Create `src/types/editor.ts`; modify `src/services/tauri.ts`.

- [x] Define `EditorId = 'codex' | 'vscode' | 'cursor'` and `InstalledEditor { id: EditorId; name: string }`.
- [x] Add `listInstalledEditors()` invoking `list_installed_editors`.
- [x] Add `openProjectInEditor(projectId, editorId)` invoking `open_project_in_editor` with `{ projectId, editorId }`.

## Task 4: Add the context-menu submenu

**Files:** Modify `src/components/ProjectDock.vue`; modify `src/App.vue`.

- [x] Extend context-menu state with submenu open state and detected editor/loading/error state. Scan asynchronously each time a project context menu opens.
- [x] Add “在 IDE 中打开” as the first project-menu item. Its child menu displays detected editors in stable order, a loading state, or “未检测到 IDE”; include a rescan action.
- [x] Keep edit/delete actions below the IDE entry and preserve their existing emits.
- [x] Select an editor by emitting `openProjectInEditor(projectId, editorId)` and closing both menus. In `App.vue`, call the Tauri API and send launch errors to `showToast`.
- [x] Ensure scan completion does not update a closed or replaced context menu; cancel or ignore stale scan results.
- [x] Support Escape/outside click for both levels and focus-safe accessible menu labels/roles.

## Task 5: Style and verify

**Files:** Modify `src/style.css`; modify `todo.md`.

- [x] Expand context-menu dimensions and position the IDE submenu to the right when there is room, otherwise to the left; clamp both menus to the app window bounds.
- [x] Style detected, loading, empty, refresh, hover, focus, and disabled states to match the existing context-menu palette.
- [x] Add the feature to the completed section of `todo.md` after implementation.
- [x] Run `pnpm build`, `cargo fmt --check`, and `cargo check --offline`; run `git diff --check`.
- [x] Inspect the resulting diff and confirm the project-root path is always passed as a discrete argument or encoded URI value.

## Design Coverage Review

- The project-specific right-click menu placement and retained edit/delete actions are covered by Task 4.
- Detection of Codex Desktop, VS Code, Cursor, refresh, and no-editor states is covered by Tasks 1 and 4.
- Safe launch of the exact saved project root and launch errors is covered by Tasks 1–2 and 4.
- Space/Unicode paths and avoiding shell interpolation are covered by Task 1.
- The non-Windows behavior and no-persistence boundary are covered by Task 1 and Task 2.
