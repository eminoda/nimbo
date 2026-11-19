# Git Branch Switcher Implementation Plan

> **For agentic workers:** Implement the approved design task by task. Use inline execution in the current session; do not delegate.

**Goal:** Add a compact project-level selector that lists and switches local Git branches safely.

**Architecture:** A focused Rust Git module runs `git` with argument arrays and resolves project paths from saved settings. Tauri commands expose list and switch operations, with the switch command holding the sessions lock and rejecting projects with active sessions. A Vue branch selector owns its dropdown, filtering, loading, and error UI and is placed between command tabs and project actions.

**Tech Stack:** Vue 3 Composition API + TypeScript, Tauri 2 commands, Rust `std::process::Command`, existing CSS and Lucide icons.

---

## File Map

- Create `src-tauri/src/git.rs`: Git process execution and branch status parsing, independent of Tauri UI state.
- Modify `src-tauri/src/lib.rs`: declare `git` module and register two Git commands.
- Modify `src-tauri/src/commands.rs`: resolve saved project paths, list branch state, and safely switch a local branch.
- Modify `src/services/tauri.ts`: add typed `listProjectBranches` and `switchProjectBranch` wrappers.
- Create `src/components/BranchSwitcher.vue`: branch button, searchable local branch popover, refresh, busy guard, and accessible feedback.
- Modify `src/App.vue`: pass project ID and active-project status, and mount the selector beside command tabs.
- Modify `src/style.css`: compact branch control and popover styles matching the current dark theme.
- Modify `todo.md`: record the branch selector feature after implementation and build verification.

## Task 1: Implement the Rust Git adapter

**Files:** Create `src-tauri/src/git.rs`; modify `src-tauri/src/lib.rs`.

- [x] Define a serializable `GitBranchInfo` containing `current_branch: Option<String>` and `local_branches: Vec<String>` with camelCase JSON names.
- [x] Add a helper that runs `git -C <project-path> ...` using `std::process::Command`, captures output, and returns a readable error from stderr (falling back to stdout or process status).
- [x] Implement branch listing with `git branch --format=%(refname:short)` and current branch lookup with `git branch --show-current`; trim empty lines and sort branch names.
- [x] Return `None` current branch when Git reports detached HEAD, allowing the UI to label it as detached.
- [x] Declare `mod git;` in the Tauri library root.
- [x] Run `cargo fmt --check` after the later command changes are complete.

## Task 2: Add project-scoped Tauri commands with a backend safety check

**Files:** Modify `src-tauri/src/commands.rs`; modify `src-tauri/src/lib.rs`.

- [x] Add a private helper that looks up a project ID in saved settings and returns its configured path; return `项目不存在` for unknown IDs.
- [x] Add `list_project_branches(state, project_id)` which gets the saved path and returns `GitBranchInfo` from the Git adapter.
- [x] Add `switch_project_branch(state, project_id, branch_name)` which rejects empty names, holds `state.sessions` while checking all sessions for the project, and rejects `Starting`, `Running`, or `Stopping` states with `请先停止此项目中的运行命令，再切换分支。`.
- [x] While the sessions lock is held, resolve the saved project path, check the exact name is in the local branch list, execute `git switch -- <branch_name>`, and return refreshed branch info. This serializes branch mutation against process startup, which also takes the sessions lock.
- [x] Never invoke a shell, interpolate a command string, force a checkout, or accept an arbitrary frontend path.
- [x] Register both commands in `tauri::generate_handler!`.

## Task 3: Add typed frontend API wrappers

**Files:** Modify `src/services/tauri.ts`.

- [x] Define `GitBranchInfo` in the frontend service or a focused `src/types/git.ts` file, with `currentBranch: string | null` and `localBranches: string[]`.
- [x] Add `listProjectBranches(projectId)` invoking `list_project_branches` with `{ projectId }`.
- [x] Add `switchProjectBranch(projectId, branchName)` invoking `switch_project_branch` with `{ projectId, branchName }`.

## Task 4: Build the branch selector UI

**Files:** Create `src/components/BranchSwitcher.vue`.

- [x] Define typed props `{ projectId: string; busy: boolean }` and local refs for open state, query, branch info, loading, switching, and error.
- [x] Fetch branch info when the popover opens and provide a refresh action; filter branches case-insensitively from the query.
- [x] Render a compact `GitBranch` trigger showing the current branch or `HEAD` detached label; when `busy`, disable choosing branches and expose a title explaining that active commands must be stopped.
- [x] Render a bounded popover with current branch, search input, scrollable branch list, current-branch check mark, and refresh control. Show clear empty, non-Git/error, loading, and busy states.
- [x] On branch selection, call `switchProjectBranch`; on success replace branch info and close the popover; on error keep the popover open and show the message. Prevent selecting the current branch.
- [x] Close on Escape and outside click, remove global listeners on unmount, and provide accessible labels and roles.

## Task 5: Place the selector and style it

**Files:** Modify `src/App.vue`; modify `src/style.css`.

- [x] Mount `BranchSwitcher` between `CommandTabs` and `ProjectHeader` in `.command-tabs-row`, passing `project.id` and `runningProjectIds.includes(project.id)`.
- [x] Ensure project-keyed remounting refreshes state when a different project is selected.
- [x] Style the trigger and popover to match existing compact dark controls, constrain popup height, keep it above terminal content, and preserve horizontal command-tab scrolling at narrow window widths.
- [x] Keep project edit/delete actions visible and do not change command or session behavior.

## Task 6: Update tracking and verify

**Files:** Modify `todo.md`.

- [x] Add the feature to the completed list after implementation.
- [x] Run `pnpm build` and `cargo fmt --check` and `cargo check --offline` from `src-tauri`.
- [x] Inspect `git diff --check` and `git status`; resolve formatting or unrelated-file changes before committing.

## Design Coverage Review

- Branch placement and compact popover are covered by Tasks 4–5.
- Current/local branch display, search, refresh, and error states are covered by Tasks 1, 3, and 4.
- Project-path resolution and safe Git arguments are covered by Tasks 1–2.
- Active-session checks on frontend and backend plus synchronization with process startup are covered by Tasks 2 and 5.
- Non-forced switching and Git conflict reporting are covered by Tasks 1–2 and 4.
- Non-Git directories and detached HEAD are covered by Tasks 1 and 4.
- No test suite changes are planned because the user has not requested tests; compile/build checks will verify the implementation.
