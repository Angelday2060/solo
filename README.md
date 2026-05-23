# solo

本地优先的个人信息与学习工作台：**日记、记账、笔记、日程与设置**，数据保存在本机 SQLite，无需登录。

| | |
|---|---|
| 作者 | Angelday（3063508819@qq.com） |
| 版本 | 0.1.0 |
| 技术栈 | Tauri 2 · Vue 3 · Vite · TypeScript · Naive UI · Chart.js · SQLite |

## 功能概览

| 模块 | 状态 | 说明 |
|------|------|------|
| 日记 | ✅ 可用 | 按日记录，支持标题、正文、天气与心情；列表按年月分组，自动保存 |
| 记账 | ✅ 可用 | 账单列表与单笔新建/编辑；CNY / JPY、收入/支出、自定义分类；统计视图含折线图、柱状图、饼图与周期汇总 |
| 笔记 | ✅ 可用 | 笔记本树形结构、笔记 CRUD、标签与附件占位；置顶与最近编辑排序 |
| 日程 | 🚧 占位 | 路由与布局已接入，业务逻辑待实现 |
| 设置 | ✅ 可用 | 浅色/深色主题、记账分类维护等 |

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/)（建议 LTS）
- [Rust](https://rustup.rs/)（`rustup` 安装后可用 `cargo`）
- **Windows 打包**：Visual Studio C++ 构建工具

### 安装与运行

```bash
git clone https://github.com/Angelday2060/solo.git
cd solo
npm install
npm run tauri dev
```

仅调试前端 UI（无 Tauri 后端能力）：

```bash
npm run dev
```

类型检查：

```bash
npx vue-tsc --noEmit
```

## 常用命令

| 命令 | 说明 |
|------|------|
| `npm run tauri dev` | 开发模式（Vite + Tauri） |
| `npm run tauri build` | 生产构建并打包桌面安装包 |
| `npm run build` | 仅构建前端静态资源，输出 `dist/` |
| `npm run preview` | 预览 `dist/`（无桌面壳） |

## 打包发布

```bash
npm run tauri build
```

- **Windows**：合并 `tauri.conf.json` 与 `tauri.windows.conf.json`，默认产出 **NSIS** 安装包（避免依赖 WiX，国内网络更友好）
- 安装包路径：`src-tauri/target/release/bundle/`

<details>
<summary>构建时走 HTTP 代理（可选）</summary>

PowerShell 示例（本机代理端口 7890）：

```powershell
$env:HTTP_PROXY  = "http://127.0.0.1:7890"
$env:HTTPS_PROXY = "http://127.0.0.1:7890"
npm run tauri build
```

Rust / Cargo 同样识别 `HTTP_PROXY` / `HTTPS_PROXY`。Tauri Bundler 拉取外部工具时还可查阅官方文档中的 `TAURI_BUNDLER_TOOLS_GITHUB_MIRROR` 等变量。
</details>

## 数据存储

数据库由应用数据目录解析，默认路径：

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\com.solo\solo.db` |
| macOS / Linux | `dirs::data_dir()` 下的 `com.solo/solo.db` |

备份时直接复制 `solo.db` 即可（建议先退出应用）。

## 项目结构

```
solo/
├── src/                    # Vue 3 前端
│   ├── views/              # 页面（日记、记账、笔记、日程、设置）
│   ├── services/           # Tauri invoke 封装
│   ├── router/             # 路由
│   ├── styles/             # 全局样式
│   └── utils/              # 工具函数
├── src-tauri/              # Rust 后端
│   ├── src/                # 业务模块（diary / ledger / notes / db）
│   ├── sql/schema.sql      # SQLite 表结构
│   ├── icons/              # 应用图标
│   ├── tauri.conf.json
│   └── tauri.windows.conf.json
├── docs/                   # 需求、原型与实体设计文档
├── package.json
└── vite.config.js
```

更详细的架构说明见 [`docs/开发文档-技术栈与结构.md`](docs/开发文档-技术栈与结构.md)。

## 许可

本项目采用 [MIT License](LICENSE)（Copyright © 2026 陈雨轩）。
