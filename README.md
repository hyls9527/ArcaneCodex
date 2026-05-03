# Arcane Codex

> **你的照片，就是记忆** | Your Photos, Your Memories

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey)]()
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)]()
[![React](https://img.shields.io/badge/React-18-61dafb.svg)]()
[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB.svg)]()
[![Tests](https://img.shields.io/badge/Tests-223%20passing-brightgreen)]()
[![Code Quality](https://img.shields.io/badge/Code%20Quality-A-brightgreen)]()

---

你拍了很多照片。硬盘里躺着几万张，手机相册里还有几万张。找一张特定照片的时候，就像在垃圾堆里翻宝贝——明明知道它在那里，就是找不到。

试过各种工具。系统相册只能按时间排序，云相册要把照片传到别人的服务器上，专业 DAM 软件贵得离谱还不好用。

所以写了 Arcane Codex。它用 AI 理解照片内容，用自然语言找到你想要的那一张，全部运行在本地——你的照片从来不离开你的硬盘。

## ✨ 核心特性

| 特性 | 说明 |
|------|------|
| 🔒 **本地优先** | 所有数据存储和处理都在本地完成，无需上传，无需联网 |
| 🤖 **AI 理解内容** | 不是文件名搜索，是真正理解画面——"那只橘猫"、"去年夏天的海边" |
| 🔍 **智能去重** | BK-Tree + 感知哈希，几万张图片依然流畅 |
| 💬 **语义搜索** | 自然语言查询，不是关键词匹配 |
| 🛡️ **隐私无忧** | 照片不上传，标签不上传，一切都在你的硬盘上 |

## 🎯 AI 打标效果

```
Before:  IMG_20240315_001.jpg
After:   日落时分的海滩，金色阳光洒在波浪上
         Tags: 日落 / sunset / 海滩 / beach / 海浪 / waves / 金色 / golden
         Category: 风景 / landscape
         Confidence: 0.94

Before:  DSC_0087.jpg
After:   一只橘猫蜷缩在沙发上打盹
         Tags: 猫 / cat / 橘猫 / orange cat / 沙发 / sofa / 打盹 / napping
         Category: 动物 / animal
         Confidence: 0.91
```

## 🚀 快速开始

### 一键安装

1. 前往 [Releases](https://github.com/hyls9527/ArcaneCodex/releases) 下载最新安装包
2. 双击运行安装程序
3. 启动应用，拖入照片，开始体验

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/hyls9527/ArcaneCodex.git
cd ArcaneCodex

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建发布版本
npm run tauri build
```

**前置条件：**
- Node.js 20+
- Rust 1.75+
- Windows 10+

## 📚 文档

- [API 文档](docs/api.md) - 完整的 Tauri 命令 API 参考
- [架构文档](docs/architecture.md) - 系统架构和技术细节
- [贡献指南](CONTRIBUTING.md) - 如何参与开发
- [安全政策](SECURITY.md) - 安全报告和最佳实践

## 🛠️ 技术栈

| 层级 | 技术 | 为什么选它 |
|------|------|-----------|
| 前端 | React 18 + TypeScript + Tailwind CSS | 类型安全 + 快速开发 |
| 后端 | Rust + Tauri 2.0 | 性能 + 安全 + 跨平台潜力 |
| 数据库 | SQLite (WAL) | 零配置、单文件、可靠 |
| AI | OpenAI 兼容 API | 接入任何 LLM 服务 |
| 状态管理 | Zustand | 轻量、无 boilerplate |
| 测试 | Vitest + Cargo test | 223 个前端测试 + Rust 内联测试 |

## 🔐 隐私保护

Arcane Codex 的设计哲学是**本地优先**：

- ✅ 照片文件始终保存在你的硬盘上，应用不会复制或上传任何原图
- ✅ AI 分析在本地运行（LM Studio / Ollama），或通过你自行配置的 API 完成
- ✅ 不收集任何使用数据、不发送遥测信息、不连接任何第三方服务器
- ✅ 数据库是本地 SQLite 文件，你完全拥有和控制所有数据
- ✅ 支持数据备份和恢复，迁移自由

## 📋 功能矩阵

| 功能 | 状态 | 说明 |
|------|------|------|
| 批量导入 | ✅ | 拖拽文件夹，递归扫描 |
| AI 自动打标 | ✅ | 支持 6 种 LLM 后端 |
| 语义搜索 | ✅ | 自然语言查询 |
| 重复检测 | ✅ | BK-Tree + 感知哈希 |
| EXIF 提取 | ✅ | 拍摄时间、相机型号、GPS |
| 图片归档 | ✅ | 标记归档，不删除 |
| 安全导出 | ✅ | 导出到指定目录 |
| 数据备份/恢复 | ✅ | SQLite 数据库备份 |
| 中英双语 | ✅ | i18n 完整支持 |
| 暗黑/明亮主题 | ✅ | 跟随系统或手动切换 |
| 键盘快捷键 | ✅ | / 搜索、Esc 关闭、d 切换主题 |
| 仪表盘统计 | ✅ | 分类分布、准确率趋势 |
| HEIC/HEIF | ❌ | 暂不支持，需先转换格式 |
| macOS/Linux | ❌ | 暂仅支持 Windows |

## 📁 项目结构

```
ArcaneCodex/
├── .github/              # GitHub 配置 (CI/CD, Issue 模板)
├── docs/                 # 文档
│   ├── api.md            # API 文档
│   ├── architecture.md   # 架构文档
│   └── screenshots/      # 截图
├── frontend/             # React + TypeScript 前端
│   ├── src/
│   │   ├── components/   # UI 组件
│   │   ├── hooks/        # 自定义 Hooks
│   │   ├── stores/       # Zustand 状态管理
│   │   ├── lib/          # API 层 + 工具函数
│   │   └── i18n/         # 国际化
│   └── tests/            # 测试文件
├── src-tauri/            # Rust 后端
│   ├── src/
│   │   ├── commands/     # Tauri 命令
│   │   ├── core/         # 核心逻辑
│   │   └── utils/        # 工具函数
│   └── Cargo.toml
└── package.json          # 根配置
```

## 🤝 贡献

欢迎提 Issue 和 PR！请阅读 [贡献指南](CONTRIBUTING.md) 了解详情。

```bash
# 开发流程
git clone https://github.com/hyls9527/ArcaneCodex.git
cd ArcaneCodex
npm install
npm run tauri dev
```

## 📜 名字由来

- **Arcane** = 神秘的、隐秘的
- **Codex** = 古书、法典

你的照片就是你的记忆法典，而记忆是神秘的——有时候你记得拍过某张照片，却怎么也找不到。这个工具帮你解开记忆的封印。

## 📄 许可证

[MIT License](LICENSE)

---

**不是云相册的替代品，是你硬盘的放大镜。**
