# Changelog

All notable changes to DSH Launcher will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-08-20

### Fixed

- Detect DSH versions published under the npm `next` dist-tag, not just `latest`, so pre-release versions are no longer overlooked.
- Install the exact version reported by the update check and preserve `next` dist-tag handling when the launcher falls back to the npm CLI.
- Prefer pnpm for DSH's current dependency graph, with a bounded npm fallback, so installs do not remain stuck indefinitely in npm dependency resolution.
- Inject the `node` binary directory into the `PATH` of all `npm` subprocess calls. Without this, `npm`'s shebang (`#!/usr/bin/env node`) could not find `node` when the launcher was started from Finder/Spotlight, causing silent update and install failures on macOS.

### Added

- Changelog dialog: clicking "Update" now opens a popup showing the official GitHub release notes for the target version, rendered as Markdown, with an "Update now" button to perform the actual update.
- Link clicks inside the changelog dialog open in the system browser via `open_external`, which now accepts any HTTPS URL.

## [0.1.0] - 2026-08-17

### Added

- Initial macOS and Windows desktop launcher built with Tauri and React.
- Detection of app-managed, global npm, and npx-cached DSH runtimes.
- Explicit install and update actions for `@deepseek-ai/dsh`.
- Start, open, restart, and stop controls for verified DSH services.
- Management of verified DSH processes started from a terminal or the launcher.
- Working-directory selection, responsive window height, and draggable title bar.
- Simplified Chinese and English localization.
- Light, dark, and system appearance modes.

[Unreleased]: https://github.com/minorsnownight/dsh-launcher/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/minorsnownight/dsh-launcher/releases/tag/v0.1.1
[0.1.0]: https://github.com/minorsnownight/dsh-launcher/releases/tag/v0.1.0
