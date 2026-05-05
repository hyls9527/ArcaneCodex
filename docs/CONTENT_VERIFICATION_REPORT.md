# 内容真实性修复记录

> 生成时间：2026-05-05 | 审核轮次：2 轮（初筛 + 二次审核）

---

## 一、修复总览

| 严重程度 | 数量 | 说明 |
|----------|------|------|
| 严重（虚假信息） | 7 | 与代码实现直接矛盾 |
| 中等（夸大/遗漏） | 9 | 夸大表述或关键信息遗漏 |
| 轻微（不精确） | 4 | 信息不精确但影响较小 |
| **合计** | **20** | |

---

## 二、严重问题修复记录

### #1 FTS5 全文搜索虚假声明

| 项目 | 内容 |
|------|------|
| **文件** | `docs/performance.md` |
| **修复前** | 声称使用 SQLite FTS5 实现全文搜索 |
| **修复后** | 标注"项目使用 jieba-rs 中文分词 + 自建倒排索引实现语义搜索，未使用 SQLite FTS5" |
| **依据** | 搜索代码库无任何 FTS5 引用；实际搜索实现在 `core/search_index.rs`，使用 jieba-rs 分词 + 倒排索引 |
| **轮次** | 第 1 轮 |

### #2 LRU 缓存虚假声明（performance.md）

| 项目 | 内容 |
|------|------|
| **文件** | `docs/performance.md` |
| **修复前** | 声称实现 LRU 搜索缓存 |
| **修复后** | 标注"已实现：HashMap 缓存 + TTL 5 分钟（非 LRU）" |
| **依据** | `core/search_index.rs` 中 `SEARCH_CACHE` 类型为 `HashMap<u64, (Instant, Vec<SearchResult>)>`，无淘汰策略，仅 TTL 过期 |
| **轮次** | 第 1 轮 |

### #3 LRU 缓存虚假声明（CHANGELOG.md）

| 项目 | 内容 |
|------|------|
| **文件** | `CHANGELOG.md` |
| **修复前** | `LRU 搜索缓存` |
| **修复后** | `搜索缓存（HashMap + TTL 5 分钟）` |
| **依据** | 同 #2 |
| **轮次** | 第 2 轮（初筛遗漏） |

### #4 validate_path 函数名错误 + dead_code 状态隐瞒

| 项目 | 内容 |
|------|------|
| **文件** | `docs/security-audit.md` |
| **修复前** | 展示 `validate_path(path, base_dir)` 函数，声称"✅ 阻止路径遍历攻击" |
| **修复后** | 修正为 `sanitize_path(base_dir, user_input)`，标注"⚠️ 标记为 dead_code，当前未被主流程调用" |
| **依据** | 代码中实际函数名为 `sanitize_path`（`commands/images.rs:55`），且标记为 `#[expect(dead_code)]` |
| **轮次** | 第 2 轮 |

### #5 CLIP 集成状态错误

| 项目 | 内容 |
|------|------|
| **文件** | `CHANGELOG.md` |
| **修复前** | `CLIP zero-shot 接口（架构预留，代码存在但未集成到主流程）` |
| **修复后** | `CLIP embedding + ONNX Runtime（已集成到主流程，需 ONNX 模型文件运行；知识图谱引擎依赖此模块）` |
| **依据** | `main.rs:105` 注册 `embed_image_clip` 命令；`main.rs:177` 初始化 `ClipEmbedder`；`knowledge_graph.rs` 依赖 `ClipEmbedder` |
| **轮次** | 第 2 轮 |

### #6 代码分割示例虚假

| 项目 | 内容 |
|------|------|
| **文件** | `docs/performance.md` |
| **修复前** | 展示 `vendor: ['react', 'react-dom', 'react-router-dom']` 和 `ui: ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu']` |
| **修复后** | 替换为实际 `vite.config.ts` 中的配置：`vendor-react`, `vendor-state`, `vendor-ui`, `vendor-i18n`, `vendor-virtual`, `vendor-dropzone` |
| **依据** | 项目未使用 `react-router-dom` 和 `@radix-ui`（package.json 无此依赖），实际配置见 `frontend/vite.config.ts:25-32` |
| **轮次** | 第 2 轮 |

### #7 @tanstack/react-query 虚假依赖

| 项目 | 内容 |
|------|------|
| **文件** | `.trae/rules/记忆.md` |
| **修复前** | `Zustand + @tanstack/react-query | Zustand 4.5, React Query 5` |
| **修复后** | `Zustand | Zustand 4.5` |
| **依据** | `frontend/package.json` 中只有 `@tanstack/react-virtual`，无 `@tanstack/react-query`；代码中无 `useQuery` 导入 |
| **轮次** | 第 2 轮 |

---

## 三、中等问题修复记录

### #8 文件类型验证方法错误描述

| 项目 | 内容 |
|------|------|
| **文件** | `docs/security-audit.md` |
| **修复前** | 声称基于魔数（magic bytes）验证 |
| **修复后** | 标注"基于扩展名验证（非魔数验证，待改进）" |
| **依据** | `commands/images.rs` 中 `validate_file` 仅检查 `file_path.extension()`，无魔数检查 |
| **轮次** | 第 1 轮 |

### #9 固定 Nonce 安全风险等级低估

| 项目 | 内容 |
|------|------|
| **文件** | `docs/security-audit.md` |
| **修复前** | 固定 Nonce 归类为"低风险" |
| **修复后** | 归类为"高风险"，标注"AES-256-GCM 加密强度严重降低，相同明文产生相同密文" |
| **依据** | AES-GCM 规范要求 Nonce 唯一，重复使用导致密钥流复用，可通过 XOR 恢复明文 |
| **轮次** | 第 1 轮 |

### #10 API Key 加密与备份加密未区分

| 项目 | 内容 |
|------|------|
| **文件** | `docs/security-audit.md` |
| **修复前** | 仅展示 API Key 加密的固定 Nonce 代码，暗示整个加密系统都有问题 |
| **修复后** | 同时展示 API Key 加密（⚠️ 固定 Nonce）和备份加密（✅ 随机 Nonce），明确区分两种实现 |
| **依据** | `utils/crypto.rs:34` 使用固定 Nonce；`commands/settings.rs:336` 使用 `rand::random::<[u8; 12]>()` |
| **轮次** | 第 2 轮 |

### #11 Zip Slip 防护夸大

| 项目 | 内容 |
|------|------|
| **文件** | `CHANGELOG.md` |
| **修复前** | `Zip Slip 攻击防护` |
| **修复后** | `Zip Slip 攻击防护（部分实现，解压路径验证存在但未显式检查路径逃逸）` |
| **依据** | security-audit.md 自身已标注"部分实现"；代码中解压时 canonicalize 失败的 else 分支仍允许写入 |
| **轮次** | 第 2 轮 |

### #12 ECE 算法实现状态夸大

| 项目 | 内容 |
|------|------|
| **文件** | `CHANGELOG.md` + `.trae/rules/记忆.md` |
| **修复前** | `置信度校准：ECE 计算 + 按类别独立校准曲线` |
| **修复后** | `置信度校准：数据库表已创建（calibration_samples/reports/curves），ECE 计算逻辑待实现` |
| **依据** | 代码中只有数据库表的创建（`db.rs:343-397`），无 ECE 计算逻辑；`CalibrationService` 标注为 `#[ignore]` |
| **轮次** | 第 2 轮 |

### #13 后端架构模块遗漏近半

| 项目 | 内容 |
|------|------|
| **文件** | `.trae/rules/记忆.md` |
| **修复前** | commands 层 10 个模块、core 层 9 个模块 |
| **修复后** | commands 层 17 个模块、core 层 18 个模块，补充 ai_core、batch_ops、knowledge_graph、xmp、file_monitor、seed_data、inference_settings、clip_embedder、onnx_runtime、image_classifier、face_detector、vector_index、knowledge_graph、file_watcher、xmp_service、crypto.rs |
| **依据** | 实际目录结构验证（`src-tauri/src/commands/` 和 `src-tauri/src/core/`） |
| **轮次** | 第 2 轮 |

### #14 前端依赖列表虚假

| 项目 | 内容 |
|------|------|
| **文件** | `.trae/rules/记忆.md` |
| **修复前** | `clsx, date-fns, react-dropzone, framer-motion` |
| **修复后** | `clsx, motion (framer-motion 新包名), react-dropzone, @tanstack/react-virtual, lucide-react` |
| **依据** | `frontend/package.json` 无 `date-fns` 和 `framer-motion`，实际使用 `motion` (v11.18.2) |
| **轮次** | 第 2 轮 |

### #15 项目路径错误

| 项目 | 内容 |
|------|------|
| **文件** | `.trae/rules/记忆.md` |
| **修复前** | `e:\knowledge base\` |
| **修复后** | `e:\ArcaneCodex\` |
| **依据** | 实际项目路径 |
| **轮次** | 第 2 轮 |

### #16 Tauri Commands API 列表严重不完整

| 项目 | 内容 |
|------|------|
| **文件** | `.trae/rules/记忆.md` |
| **修复前** | 列出 14 个命令 |
| **修复后** | 列出 33+ 个命令（按功能分组），标注"完整列表见 main.rs（共 50+ 个命令）" |
| **依据** | `main.rs:31-121` 中 `generate_handler!` 注册了 50+ 个命令 |
| **轮次** | 第 2 轮 |

---

## 四、轻微问题修复记录

### #17 数据库连接池配置与代码不符

| 项目 | 内容 |
|------|------|
| **文件** | `docs/performance.md` |
| **修复前** | `r2d2::Pool::builder().max_size(10).build(manager)?` |
| **修复后** | `Pool::new(manager)?`（默认配置，未设置 max_size），补充 `with_init` 中的 PRAGMA 配置 |
| **依据** | `core/db.rs:25-31` 使用 `Pool::new(manager)` 默认配置 |
| **轮次** | 第 2 轮 |

### #18 测试覆盖率/数量虚假

| 项目 | 内容 |
|------|------|
| **文件** | `README.md` + `.trae/rules/记忆.md` |
| **修复前** | "223 个前端测试" / "96.7%测试覆盖率" |
| **修复后** | "17 个前端测试文件（基于 mock）"，移除覆盖率百分比 |
| **依据** | 静态分析显示 17 个测试文件；无覆盖率工具配置（无 cargo-llvm-cov、无 istanbul/nyc） |
| **轮次** | 第 1 轮 + 第 2 轮 |

### #19 错误信息脱敏虚假声明

| 项目 | 内容 |
|------|------|
| **文件** | `.trae/rules/记忆.md` |
| **修复前** | `错误信息脱敏：避免泄露敏感信息` |
| **修复后** | 替换为 `API Key 加密：⚠️ 固定 Nonce` + `备份加密：✅ 随机 Nonce` + `日志脱敏：❌ 未实现` |
| **依据** | security-audit.md 明确标注 `[ ] 日志脱敏` 未完成 |
| **轮次** | 第 2 轮 |

### #20 HEIC/HEIF 描述内部矛盾

| 项目 | 内容 |
|------|------|
| **文件** | `README.md` |
| **修复前** | `暂不支持，需先转换格式` |
| **修复后** | `MIME 类型已注册但解码不支持，需先转换格式` |
| **依据** | `commands/images.rs:29-30` 的 `SUPPORTED_MIME_TYPES` 包含 `image/heic` 和 `image/heif`，但实际解码不支持 |
| **轮次** | 第 2 轮 |

---

## 五、修复文件清单

| 文件 | 修复项数 | 严重 | 中等 | 轻微 |
|------|----------|------|------|------|
| `CHANGELOG.md` | 4 | 2 | 2 | 0 |
| `docs/security-audit.md` | 4 | 1 | 2 | 1 |
| `docs/performance.md` | 4 | 2 | 0 | 2 |
| `.trae/rules/记忆.md` | 10 | 1 | 6 | 3 |
| `README.md` | 3 | 0 | 0 | 3 |
| **合计** | **25** | **6** | **10** | **9** |

> 注：部分修复项在多个文件中交叉出现，按实际修改计为 25 处。

---

## 六、未修复项（需代码层面解决）

| 问题 | 说明 | 建议 |
|------|------|------|
| `crypto.rs` 固定 Nonce | 安全漏洞，文档已标注但代码未修复 | 改为随机 Nonce + 前置存储 |
| `sanitize_path` dead_code | 安全函数未被调用 | 移除 dead_code 标记，统一调用 |
| ECE 计算逻辑缺失 | 数据库表已建但无计算代码 | 实现 CalibrationService |
| 日志脱敏未实现 | 安全审计中风险项 | 实现日志脱敏过滤器 |
