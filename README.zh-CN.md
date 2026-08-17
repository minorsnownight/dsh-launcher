<div align="center">
  <img src="src-tauri/icons/icon.png" width="96" height="96" alt="DSH Launcher logo">
  <h1>DSH Launcher</h1>
  <p>一个用于安装、更新和管理 DeepSeek Harness Web 服务的轻量桌面应用。</p>
  <p>
    <a href="README.md">English</a> ·
    <a href="https://github.com/minorsnownight/dsh-launcher/releases">下载</a> ·
    <a href="https://github.com/minorsnownight/dsh-launcher/issues">反馈问题</a>
  </p>
</div>

> [!IMPORTANT]
> DSH Launcher 目前处于开发者预览阶段。它是社区维护的非官方工具，与 DeepSeek 没有隶属或背书关系。

![DSH Launcher 预览](images/preview.png)

## 它解决什么问题

DeepSeek Harness 的 Web 服务通常通过 `npx @deepseek-ai/dsh web` 启动。DSH Launcher 把安装、版本检查和进程管理整合为一个桌面界面，让日常使用不再依赖重复输入命令。

## 功能

- 识别应用托管、全局 npm 和 npx 缓存中的 `@deepseek-ai/dsh` 运行时
- 检查 npm 上的新版本，由用户手动确认更新
- 启动、打开、重启或关闭 DSH Web 服务
- 管理由终端或 DSH Launcher 启动的 DSH 进程
- 选择 DSH 启动时使用的工作目录
- 防止误关占用 `3080` 端口的非 DSH 进程
- 支持 macOS、Windows、简体中文和 English
- 支持浅色、深色和跟随系统外观

## 安装

预编译安装包将在 [GitHub Releases](https://github.com/minorsnownight/dsh-launcher/releases) 中提供。当前版本可以直接从源码运行。

运行 DSH Launcher 前，系统需要已安装 Node.js 和 npm。应用内安装的 DSH 运行时保存在应用数据目录中，不会修改全局 npm 包，也不需要管理员权限。

## 从源码运行

### 环境要求

- Node.js 与 npm
- Rust stable 工具链
- [Tauri 2 的系统依赖](https://v2.tauri.app/start/prerequisites/)

```bash
git clone https://github.com/minorsnownight/dsh-launcher.git
cd dsh-launcher
npm install
npm run desktop
```

构建本机安装包：

```bash
npm run dist
```

## 工作方式

DSH Launcher 按以下顺序查找可用运行时：

1. 应用数据目录中的托管运行时
2. 全局 npm 安装
3. 本机 npx 缓存

服务固定运行在 `http://127.0.0.1:3080`。当端口已被占用时，应用会检查监听进程的命令；只有确认它属于 `@deepseek-ai/dsh` 后，才会提供重启和关闭操作。

“工作目录”是启动 DSH 时的当前目录。DSH 会在这里读取和操作项目文件；更换目录前需要先停止正在运行的服务。

## 隐私与安全

- 不包含账号系统、遥测或分析服务
- 不会静默安装或更新 DSH
- 不会把用户输入拼接进 shell 命令
- 仅在本机回环地址上检查 DSH 服务
- 不会终止无法确认身份的端口占用进程

## 开发

```bash
npm run typecheck
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

项目结构和产品边界见 [AGENTS.md](AGENTS.md) 与 [docs/PRODUCT.md](docs/PRODUCT.md)。提交改动前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。安全问题请按 [SECURITY.md](SECURITY.md) 私下报告。

## 许可证

DSH Launcher 基于 [MIT License](LICENSE) 开源。

“DeepSeek”及相关标识归其各自权利人所有。本项目只管理用户本机安装的 `@deepseek-ai/dsh`，不包含或重新分发该软件包。
