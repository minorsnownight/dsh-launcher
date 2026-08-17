# DSH Launcher product brief

## Promise

DSH Launcher turns DeepSeek Harness from a command users must remember into a dependable desktop utility. Opening the app should answer, within one glance: is the runtime ready, is it current, is the web service available, and what is the next action?

## Primary user journey

1. Open DSH Launcher.
2. If no existing global, npx-cached, or app-managed runtime is found, choose **Install** once.
3. Choose **Start DSH**.
4. The Harness Web UI opens at `http://127.0.0.1:3080` when ready.
5. Return to the launcher to open, restart, or stop the service.

## Product decisions

- The launcher detects an existing app-managed runtime, global npm installation, or local npx cache, in that priority order. New installations are isolated inside the application-data directory, so they never require administrator access or change global npm packages.
- Update checks use the public npm registry and never install silently.
- A service already listening on port 3080 is resolved to its owning process. If its command is verified as DSH, Launcher can open, restart, or stop it regardless of whether it was started in a terminal or by the app. An unverified process is never terminated.
- Node.js remains a prerequisite because DSH itself is distributed as a Node CLI. Missing prerequisites are explained in the UI.
- The first release intentionally omits accounts, telemetry, auto-start, custom ports, and plugin management.

## Interface states

| Runtime | Service | Primary action | Secondary actions |
| --- | --- | --- | --- |
| Checking | Checking | Disabled | None |
| Missing | Stopped | Install DSH | Refresh |
| Installed | Stopped | Start DSH | Update when available |
| Installed | Starting | Starting… | None |
| Installed | Running (managed or terminal) | Open DSH | Restart, Stop |
| Any | Port occupied by an unverified process | Refresh | None |
| Any | Error | Try again | Show concise error |

## Experience principles

- One calm status surface, not a dashboard full of metrics.
- Immediate press feedback and short, interruptible-looking transitions.
- Translucency is used only for hierarchy; content remains legible without it.
- System typography, semantic color, keyboard focus, reduced motion, and reduced transparency are first-class.
- Chinese and English copy are equally authored, not machine-shaped variants of one another.
