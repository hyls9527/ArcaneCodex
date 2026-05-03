# Arcane Codex 系统架构

> 最后更新: 2026-05-03

## 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Arcane Codex Desktop App                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Frontend (React + TypeScript)                 │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │   │
│  │  │   Gallery   │  │   Search    │  │   Settings  │               │   │
│  │  │   View      │  │   View      │  │   View      │               │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │   │
│  │  │   Dedup     │  │   Tags      │  │   Import    │               │   │
│  │  │   Manager   │  │   Manager   │  │   Wizard    │               │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │   │
│  │                                                                   │   │
│  │  ┌─────────────────────────────────────────────────────────────┐ │   │
│  │  │                    State Management (Zustand)                │ │   │
│  │  └─────────────────────────────────────────────────────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────────────┐ │   │
│  │  │                    API Layer (@tauri-apps/api)               │ │   │
│  │  └─────────────────────────────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│                                    │ IPC (JSON-RPC)                      │
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Backend (Rust + Tauri)                        │   │
│  │  ┌─────────────────────────────────────────────────────────────┐ │   │
│  │  │                      Commands Layer                          │ │   │
│  │  │  images │ tags │ inference │ settings │ dedup │ storage     │ │   │
│  │  └─────────────────────────────────────────────────────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────────────┐ │   │
│  │  │                      Core Layer                              │ │   │
│  │  │  Database │ AI Engine │ File System │ Cache │ Crypto        │ │   │
│  │  └─────────────────────────────────────────────────────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────────────┐ │   │
│  │  │                      Utils Layer                             │ │   │
│  │  │  Error Handling │ Logging │ Validation │ Path Utils         │ │   │
│  │  └─────────────────────────────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                        External Services                          │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │   │
│  │  │   OpenAI    │  │  Anthropic  │  │  LM Studio  │               │   │
│  │  │    API      │  │    API      │  │   (Local)   │               │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 数据流

### 图片导入流程

```
用户选择图片
      │
      ▼
┌─────────────┐
│  Frontend   │ invoke('import_images', { paths })
└─────────────┘
      │
      ▼
┌─────────────┐
│   Backend   │ 验证路径、检查格式
└─────────────┘
      │
      ▼
┌─────────────┐
│  Database   │ 创建图片记录
└─────────────┘
      │
      ▼
┌─────────────┐
│ File System │ 生成缩略图、复制文件
└─────────────┘
      │
      ▼
┌─────────────┐
│   Return    │ { imported, skipped, errors }
└─────────────┘
```

### AI 打标流程

```
用户请求打标
      │
      ▼
┌─────────────┐
│  Frontend   │ invoke('tag_image', { imageId, provider })
└─────────────┘
      │
      ▼
┌─────────────┐
│   Backend   │ 读取图片、准备 prompt
└─────────────┘
      │
      ▼
┌─────────────────────────────────────────────┐
│              AI Provider Selection           │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │ OpenAI  │  │Anthropic│  │  LM Studio  │  │
│  └─────────┘  └─────────┘  └─────────────┘  │
└─────────────────────────────────────────────┘
      │
      ▼
┌─────────────┐
│   Backend   │ 解析响应、提取标签
└─────────────┘
      │
      ▼
┌─────────────┐
│  Database   │ 存储标签关联
└─────────────┘
      │
      ▼
┌─────────────┐
│   Return    │ { tags, model, processing_time }
└─────────────┘
```

---

## 目录结构

```
ArcaneCodex/
├── .github/                    # GitHub 配置
│   ├── workflows/              # CI/CD 工作流
│   ├── ISSUE_TEMPLATE/         # Issue 模板
│   ├── dependabot.yml          # 依赖更新配置
│   └── CODEOWNERS              # 代码所有者
│
├── .husky/                     # Git Hooks
│   ├── commit-msg              # Commitlint
│   └── pre-commit              # Lint 检查
│
├── .trae/                      # Trae IDE 配置
│   ├── rules/                  # 项目规则
│   └── skills/                 # 自定义 Skills
│
├── docs/                       # 文档
│   ├── planning/               # 规划文档
│   ├── api.md                  # API 文档
│   ├── architecture.md         # 架构文档
│   └── screenshots/            # 截图
│
├── frontend/                   # 前端代码
│   ├── src/
│   │   ├── components/         # React 组件
│   │   │   ├── gallery/        # 图库组件
│   │   │   ├── search/         # 搜索组件
│   │   │   ├── settings/       # 设置组件
│   │   │   ├── dedup/          # 去重组件
│   │   │   └── tags/           # 标签组件
│   │   ├── hooks/              # 自定义 Hooks
│   │   ├── lib/                # 工具库
│   │   ├── stores/             # Zustand 状态
│   │   ├── i18n/               # 国际化
│   │   └── types/              # TypeScript 类型
│   ├── public/                 # 静态资源
│   ├── tests/                  # 测试文件
│   └── package.json
│
├── src-tauri/                  # 后端 Rust 代码
│   ├── src/
│   │   ├── commands/           # Tauri 命令
│   │   │   ├── images.rs       # 图片管理
│   │   │   ├── tags.rs         # 标签管理
│   │   │   ├── inference.rs    # AI 推理
│   │   │   ├── settings.rs     # 设置管理
│   │   │   ├── dedup.rs        # 去重功能
│   │   │   └── storage.rs      # 存储管理
│   │   ├── core/               # 核心逻辑
│   │   │   ├── database.rs     # 数据库
│   │   │   ├── inference.rs    # AI 引擎
│   │   │   ├── cache.rs        # 缓存
│   │   │   └── dedup.rs        # 去重算法
│   │   ├── utils/              # 工具函数
│   │   │   ├── error.rs        # 错误处理
│   │   │   ├── crypto.rs       # 加密
│   │   │   └── path.rs         # 路径处理
│   │   └── lib.rs              # 入口
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── CHANGELOG.md                # 变更日志
├── CODE_OF_CONDUCT.md          # 行为准则
├── CONTRIBUTING.md             # 贡献指南
├── LICENSE                     # 许可证
├── README.md                   # 项目说明
├── SECURITY.md                 # 安全政策
└── package.json                # 根 package.json
```

---

## 技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| React | 18.x | UI 框架 |
| TypeScript | 5.x | 类型安全 |
| Vite | 5.x | 构建工具 |
| Tailwind CSS | 3.x | 样式框架 |
| Zustand | 4.x | 状态管理 |
| i18next | 23.x | 国际化 |
| Vitest | 2.x | 测试框架 |
| Playwright | 1.59 | E2E 测试 |

### 后端

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 1.75+ | 系统语言 |
| Tauri | 2.x | 桌面框架 |
| SQLite (rusqlite) | 0.31 | 数据库 |
| reqwest | 0.12 | HTTP 客户端 |
| tokio | 1.x | 异步运行时 |
| serde | 1.x | 序列化 |
| tracing | 0.1 | 日志 |

### AI 集成

| Provider | API | 用途 |
|----------|-----|------|
| OpenAI | GPT-4o / GPT-4o-mini | 图片描述、标签生成 |
| Anthropic | Claude 3.5 Sonnet | 图片描述、标签生成 |
| Local | LM Studio / Ollama | 本地推理 |

---

## 安全架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Security Layers                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Input Validation                       │   │
│  │  • Path sanitization (prevent traversal attacks)         │   │
│  │  • File type validation (magic bytes check)              │   │
│  │  • Input size limits                                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Data Protection                        │   │
│  │  • API keys encrypted with AES-256-GCM                   │   │
│  │  • Database stored locally only                          │   │
│  │  • No telemetry / data collection                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    File Operations                        │   │
│  │  • Zip Slip attack prevention                            │   │
│  │  • Disk space validation before operations               │   │
│  │  • Atomic file operations                                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Network Security                       │   │
│  │  • HTTPS only for API calls                              │   │
│  │  • No external connections except AI providers           │   │
│  │  • Certificate validation                                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 性能优化

### 前端

- **代码分割**: 路由级别懒加载
- **虚拟列表**: 大图库使用 react-window
- **图片懒加载**: 缩略图按需加载
- **缓存**: React Query / Zustand 持久化

### 后端

- **数据库索引**: 常用查询字段索引
- **LRU 缓存**: 搜索结果缓存
- **异步处理**: tokio 并发
- **缩略图**: 预生成多尺寸缩略图

---

## 扩展性

### 添加新的 AI Provider

1. 在 `src-tauri/src/core/inference.rs` 添加新的 Provider 实现
2. 在 `src-tauri/src/commands/inference.rs` 添加路由逻辑
3. 在前端设置页面添加配置选项

### 添加新的图片格式支持

1. 在 `src-tauri/src/commands/images.rs` 添加格式验证
2. 更新前端文件选择过滤器
3. 添加相应的测试用例

### 添加新的语言支持

1. 在 `frontend/src/i18n/locales/` 添加新的 JSON 文件
2. 更新 `frontend/src/i18n/index.ts` 导入
3. 在设置页面添加语言选项
