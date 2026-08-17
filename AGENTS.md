# DSH Launcher development rules

## Product scope

- Build a small cross-platform desktop controller for `@deepseek-ai/dsh`.
- The primary screen must make install, update, running state, and the next safe action obvious.
- Keep advanced diagnostics and preferences secondary to the core service controls.

## Architecture

- Use Tauri for the native shell and React + TypeScript for the UI.
- Put operating-system process, package-manager, and version checks in Rust commands.
- Keep presentation state and localization in the web UI; do not invoke shell commands from the renderer.
- Never pass user-controlled strings through a shell. Spawn executables with explicit argument arrays.

## Project structure

- `src/`: React UI, styles, localization, and frontend tests.
- `src-tauri/`: Tauri configuration and Rust backend.
- `docs/`: product and engineering decisions that are useful to contributors.
- Static artwork belongs in `public/`; generated build output is never committed.

## Verification

- Run `npm run typecheck`, `npm test`, and `npm run build` after UI changes.
- Run `cargo test --manifest-path src-tauri/Cargo.toml` after backend changes.
- Run `npm run tauri build -- --debug` before release-oriented handoff when the host toolchain supports it.

## Change discipline

- Keep changes limited to the requested product. Do not add accounts, analytics, cloud services, or auto-start without an explicit requirement.
- Do not modify credentials, signing settings, CI/CD, or release infrastructure without user approval.
- All user-visible strings must exist in both English and Simplified Chinese.
- Respect system light/dark preference and reduced-motion preference by default.
