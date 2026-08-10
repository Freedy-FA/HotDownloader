# 🎵 HotDownloader

> 一个基于 Tauri 2 和 Vue 3 的桌面音乐下载工具，支持多任务并发、断点续传、自动降级与便捷的任务管理。

![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.77.2+-orange.svg)
![Node](https://img.shields.io/badge/node-18+-green.svg)

---

## ✨ 特性

- 🔍 **音乐搜索**：支持关键词搜索、搜索建议、热搜词，快速定位歌曲。
- ⬇️ **智能下载**：多任务并发下载（可调整并发数），支持断点续传（暂停/恢复）。
- 🔄 **自动降级**：当指定音质不可用时，可按预设顺序自动降级至可用品质。
- 📋 **完整任务管理**：任务状态（等待/下载/暂停/完成/错误）清晰分类，支持批量删除、重试、打开文件位置。
- ⚙️ **个性化设置**：自定义下载目录、文件命名模板、默认音质、是否自动跳转任务页等。
- 🎨 **深色模式**：跟随系统主题，沉浸式视觉体验。
- 💾 **本地持久化**：所有任务记录、设置、搜索历史均保存于本地，重启不丢失。

---

## 🖥️ 技术栈

| 前端                    | 后端                        |
| ----------------------- | --------------------------- |
| Vue 3 (Composition API) | Rust (Tauri 2)              |
| TypeScript              | Tokio (异步运行时)          |
| Vite                    | Reqwest (HTTP 客户端)       |
| Pinia (状态管理)        | Tauri Store Plugin (持久化) |
| Vue Router (Hash 模式)  | tauri-plugin-log            |
| Naive UI (组件库)       |                             |

---

## 📦 环境要求

- [Rust](https://www.rust-lang.org/) (版本 ≥ 1.77.2)
- [Node.js](https://nodejs.org/) (版本 ≥ 18)
- [pnpm](https://pnpm.io/) / npm / yarn

---

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/lerdb/HotDownloader.git
cd HotDownloader
```

### 2. 安装前端依赖

```bash
npm install
```

### 3. 开发模式运行

```bash
npm run tauri dev
```

> 首次运行会下载 Tauri 依赖并编译 Rust 后端，请耐心等待。

### 4. 生产构建

```bash
npm run tauri build
```

构建产物将位于 `src-tauri/target/release/bundle/` 目录下。

---

## 📄 许可证

本项目采用 [Apache License 2.0](LICENSE) 开源协议。

---

## ⚠️ 免责声明

**HotDownloader 仅用于学习和研究目的。**

用户需自行承担使用本软件所带来的法律责任。请确保你下载的音乐文件拥有合法的使用权，遵守相关音乐平台的版权规定。本项目开发者不对任何侵权行为负责。
