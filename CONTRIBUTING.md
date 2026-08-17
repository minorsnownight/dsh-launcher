# 贡献指南 / Contributing

感谢你改进 DSH Launcher。提交代码前，请先搜索现有 issue，避免重复工作。

Thanks for improving DSH Launcher. Before writing code, search existing issues to avoid duplicated work.

## 开发流程 / Development workflow

1. Fork 仓库并从 `main` 创建分支。/ Fork the repository and branch from `main`.
2. 保持改动聚焦；所有界面文案同时提供简体中文和英文。/ Keep changes focused; provide both Simplified Chinese and English for user-facing copy.
3. 不要提交密钥、运行日志、依赖目录或构建产物。/ Do not commit credentials, logs, dependencies, or build output.
4. 提交 Pull Request 前运行以下检查。/ Run these checks before opening a pull request.

```bash
npm run typecheck
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

影响安装、进程控制或端口检测的改动，应在 macOS 和 Windows 上分别说明验证结果。若无法验证某个平台，请在 Pull Request 中明确注明。

Changes affecting installation, process control, or port detection should include macOS and Windows verification notes. If you cannot test one platform, state that clearly in the pull request.

## 报告问题 / Reporting issues

普通缺陷请使用 GitHub issue 模板，并附上操作系统、应用版本、复现步骤和实际结果。安全漏洞不要提交公开 issue，请遵循 [SECURITY.md](SECURITY.md)。

Use the GitHub issue template for regular bugs and include the operating system, app version, reproduction steps, and actual result. Do not open a public issue for a vulnerability; follow [SECURITY.md](SECURITY.md).
