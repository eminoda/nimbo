# Nimbo Terminal

Nimbo Terminal is a Windows-first desktop workspace for saving local projects and running their development commands from one window. Each command keeps its own process state and bounded output history while you switch between projects.

## Run locally

Requirements: Node.js, pnpm, Rust, and the platform dependencies required by Tauri 2.

```powershell
pnpm install
pnpm tauri dev
```

To build the frontend, run `pnpm build`. To produce a desktop installer, run `pnpm tauri build`.

## Architecture

- Vue 3 Composition API and TypeScript own the workspace UI and local view state.
- Typed Tauri commands persist project bookmarks as JSON in the app data directory.
- Rust owns process sessions and uses Tauri's Shell plugin with a Windows Job Object to stop command process trees.
- Adding a command scans the project's `package.json` and offers its scripts using the detected package manager; selection only fills the command form.
- xterm.js renders bounded stdout and stderr history. The MVP does not include a PTY or interactive shell input.
- Native folder selection, external link opening, the system tray, and window controls use Tauri 2 APIs.

Commands are local programs configured by the user and run from the configured working directory. The command text and working directory stay visible in the UI. Project bookmarks never modify or delete project files.
