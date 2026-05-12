# ArcaneCodex 项目治理路线图

> **版本**: v1.0 | **日期**: 2026-05-12 | **状态**: 待审批
> **扫描范围**: Rust 后端 (46 文件) / React 前端 (全量) / 安全配置 (全局)

---

## 一、执行摘要

| 维度 | 当前评分 | 目标评分 | 差距 |
|------|---------|---------|------|
| Rust 后端质量 | ⭐ 4.2/5 | ⭐ 4.8/5 | -0.6 |
| 前端架构健康度 | ⭐ 3.8/5 | ⭐ 4.5/5 | -0.7 |
| 安全与配置合规 | ⭐ 3.5/5 | ⭐ 4.8/5 | -1.3 |
| **综合** | **⭐ 3.9/5** | **⭐ 4.7/5** | **-0.8** |

**核心发现**: 项目架构设计优秀，安全基础扎实。主要债务集中在：
1. CI/CD 供应链安全（30 分钟可修完）
2. 前端巨型组件拆分（10 小时工作量）
3. 密码学实践升级（需评估迁移策略）

---

## 二、Phase 0：紧急修复（本次迭代，~2 小时）

> 目标：消除运行时崩溃风险 + 源码泄露 + CI 供应链风险

### P0-01: 修复生产代码 panic!（Rust 后端）

- **文件**: `src-tauri/src/core/inference.rs:907,926`
- **问题**: 遇到未知 AI Provider 时调用 `panic!()` 导致应用崩溃
- **修复方案**: 改为返回 `Err(AppError::UnknownProvider(...))`
- **指派**: backend-architect agent
- **验收**: `cargo clippy --lib -- -D warnings` 通过

### P0-02: 关闭生产环境 sourcemap（前端构建）

- **文件**: `frontend/vite.config.ts:21`
- **问题**: `sourcemap: true` 在生产构建中泄露完整源码映射
- **修复方案**: `sourcemap: process.env.NODE_ENV === 'development'`
- **指派**: frontend-architect agent
- **验收**: `npm run build` 后检查无 .map 文件

### P0-03: Pin CI Action 版本 + 添加 permissions（CI/CD）

- **文件**: `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- **问题**:
  - `tauri-apps/tauri-action@v0` 未使用 SHA pin（供应链风险）
  - 缺少 `permissions:` 声明（权限过大）
- **修复方案**:
  ```yaml
  # ci.yml
  permissions:
    contents: read
  
  # release.yml  
  permissions:
    contents: write
    packages: write
  
  # 所有 action 使用精确 SHA 或 major.minor.patch
  - uses: tauri-apps/tauri-action@v0.10.0  # 精确版本
  ```
- **指派**: devops-architect agent
- **验收**: 新 CI 运行通过

### P0-04: 修复 App.tsx ai_tags 双重解析（前端）

- **文件**: `frontend/src/App.tsx`
- **问题**: `ai_tags` 字段被 JSON.parse 两次，可能导致运行时异常
- **修复方案**: 统一解析逻辑，添加类型守卫
- **指派**: frontend-architect agent
- **验收**: TypeScript check 通过 + 手动验证 Gallery 页面正常

### Phase 0 验收标准

- [ ] Clippy 零 error/warning
- [ ] ESLint 零 error/warning
- [ ] TypeScript `--noEmit` 通过
- [ ] 前端测试 223 用例全部通过
- [ ] CI 流水线全绿

---

## 三、Phase 1：短期治理（1-2 周，~15 小时）

> 目标：提升代码可维护性 + 消除 MEDIUM 风险项

### P1-01: 拆分 KnowledgeGraphView 巨型组件（前端）

- **当前行数**: 533 行
- **目标**: 拆分为 ≤150 行的子组件
- **建议拆分**:
  - `GraphCanvas` — 图渲染核心（~200 行）
  - `NodeDetailPanel` — 节点详情面板（~120 行）
  - `GraphControls` — 缩放/筛选控制栏（~80 行）
  - `KnowledgeGraphView` — 容器组件（~130 行）
- **指派**: frontend-architect agent
- **预估**: 4 小时

### P1-02: 拆分 ImageViewer 巨型组件（前端）

- **当前行数**: 461 行
- **目标**: ≤150 行
- **建议拆分**:
  - `ImageDisplay` — 图片显示区域
  - `ImageMetadata` — EXIF/AI 信息面板
  - `ImageNavigation` — 上一张/下一张控制
- **指派**: frontend-architect agent
- **预估**: 3 小时

### P1-03: 拆分 GalleryPage 巨型组件（前端）

- **当前行数**: 441 行
- **目标**: ≤200 行（页面级组件允许稍大）
- **建议拆分**:
  - `GalleryToolbar` — 工具栏（筛选/排序/视图切换）
  - `GalleryGrid` — 图片网格容器
  - `GallerySidebar` — 侧边信息面板
- **指派**: frontend-architect agent
- **预估**: 3 小时

### P1-04: 移除 CSP unsafe-inline（Tauri 安全）

- **文件**: `src-tauri/tauri.conf.json:15`
- **当前**: `"style-src 'self' 'unsafe-inline';"`
- **目标**: `"style-src 'self';"` （Tailwind 完全通过 class 实现）
- **风险**: 需验证无内联 style 残留
- **指派**: compliance-checker agent
- **预估**: 30 分钟

### P1-05: 创建 .env.example（运维）

- **位置**: 项目根目录 `.env.example`
- **内容**: 所有必要环境变量模板 + 注释说明
- **指派**: devops-architect agent
- **预估**: 15 分钟

### P1-06: 清理 notify crate 多余平台特性（依赖）

- **文件**: `src-tauri/Cargo.toml:80`
- **当前**: `notify = { version = "7", features = ["macos_kqueue"] }`
- **问题**: Windows-only 项目启用了 macOS 特性
- **修复**: 移除 `macos_kqueue` feature
- **指派**: backend-architect agent
- **预估**: 5 分钟

### P1-07: stores DRY 重构（前端）

- **文件**: `frontend/src/stores/useConfigStore.ts`
- **问题**: 重复 switch-case 模式
- **方案**: 提取通用配置访问器 hook
- **指派**: frontend-architect agent
- **预估**: 1 小时

### Phase 1 验收标准

- [ ] 无超过 300 行的组件
- [ ] CSP 不含 unsafe-inline
- [ ] `.env.example` 存在且完整
- [ ] Cargo.toml 无多余平台特性
- [ ] 全部现有测试通过

---

## 四、Phase 2：中期加固（2-4 周，~20 小时）

> 目标：密码学升级 + 数据库事务支持 + 测试代码质量

### P2-01: 升级密钥派生算法（密码学）

- **文件**: `src-tauri/src/utils/crypto.rs:13-28`
- **当前**: 单次 SHA256 hash
- **目标**: PBKDF2-HMAC-SHA256 (600,000 iterations)
- **依赖**: 已有 `pbkdf2` crate（用于 API Key Vault）
- **风险评估**:
  - ✅ 新加密数据自动使用新算法
  - ⚠️ 已有加密数据需要迁移策略（或标记为 legacy）
- **决策点**: 是否需要向后兼容旧格式？
- **指派**: backend-architect agent
- **预估**: 2 小时（实现）+ 评估迁移策略

### P2-02: 数据库事务支持（Rust 后端）

- **现状**: 仅 1 处使用事务（应有多处）
- **需要事务的操作**:
  - 图片导入（批量 INSERT + 索引更新）
  - 批量删除（DELETE images + DELETE image_tags + DELETE search_index）
  - AI 标签写入（UPDATE images + INSERT/UPDATE tags + INSERT image_tags）
- **指派**: backend-architect agent
- **预估**: 4-8 小时

### P2-03: 测试代码 unwrap 清理（Rust 后端）

- **范围**: `src-tauri/src/commands/images.rs:1668-2320`（85+ 处）
- **方案**: 替换为 `?` 操作符或 `expect("context")`
- **优先级**: 低（不影响生产代码），但防止模式蔓延
- **指派**: backend-architect agent
- **预估**: 2-3 小时

### P2-04: SQLite 加密评估（数据安全）

- **选项 A**: SQLCipher (`rusqlite bundled-sqlcipher` feature)
  - 优点：透明加密，应用层无感知
  - 缺点：性能下降 20-40%，编译时间增加
- **选项 B**: 应用层加密（敏感字段单独 AES 加密）
  - 优点：灵活控制，仅加密高敏字段
  - 缺点：无法保护查询模式
- **选项 C**: 保持明文 + 文档告知用户
  - 优点：零成本
  - 缺点：设备丢失时数据暴露
- **决策点**: 需要用户调研（隐私 vs 性能权衡）
- **指派**: compliance-checker agent（输出决策文档）
- **预估**: 2 小时（评估）+ 实现待定

### P2-05: unsafe 代码 SAFETY 文档补全（Rust 后端）

- **范围**: `images.rs:372,405,407`（3 处 libc statvfs 调用）
- **要求**: 每处 unsafe 块添加 `// SAFETY:` 注释说明为什么需要
- **指派**: backend-architect agent
- **预估**: 30 分钟

### Phase 2 验收标准

- [ ] 密钥派生使用 PBKDF2 (600K iterations)
- [ ] 关键数据库操作使用事务
- [ ] 测试代码零 unwrap()（或仅限明确标注处）
- [ ] SQLCipher 加密方案已决策
- [ ] 全部 unsafe 有 SAFETY 注释

---

## 五、Phase 3：长期规划（1-3 月）

> 目标：架构现代化 + 可观测性 + 性能基准

### P3-01: 架构演进评估
- [ ] 评估是否需要引入第二层抽象（阶段二触发条件检查）
- [ ] Provider 抽象层（当接入第 3 个 AI Provider 时）
- [ ] 事件驱动架构（当模块间耦合度过高时）

### P3-02: 可观测性建设
- [ ] 结构化日志标准化（JSON 格式）
- [ ] 性能指标采集（AI 推理延迟、查询响应时间）
- [ ] 错误追踪集成（sentry 或自建）

### P3-03: 性能基准测试
- [ ] pHash 计算性能基线
- [ ] CLIP embedding 吞吐量测试
- [ ] 大规模导入（10000+ 图片）压力测试

### P3-04: 国际化完善
- [ ] KnowledgeGraphView 硬编码字符串提取（已发现遗漏）
- [ ] i18n 覆盖率从 92% → 100%

---

## 六、资源估算汇总

| Phase | 预估工时 | Agent 分配 | 里程碑 |
|-------|---------|-----------|--------|
| **Phase 0** | ~2h | 并行 3 agent | CI 全绿 + 零 Critical |
| **Phase 1** | ~15h | 主要 frontend-architect | 零巨型组件 |
| **Phase 2** | ~20h | backend-architect 为主 | 密码学达标 |
| **Phase 3** | 待评估 | 按需分配 | 架构就绪 |

**总计**: **~37 小时**（Phase 0-2）+ Phase 3 待定

---

## 七、风险与依赖

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 子Agent生成代码未经验证 | 引入新 bug | 每个任务必须本地 lint + test 验证 |
| CI 优化后仍有失败 | 阻塞合并 | 每次 push 后自动监控 |
| 密钥派生升级影响已有数据 | 用户无法解密旧数据 | 提供 migration tool 或 dual-mode |
| 巨型组件拆分引入回归 | 功能异常 | 拆分前后截图对比 + E2E 验证 |

---

## 八、正面实践记录（保持不变）

以下措施已正确实施，后续维护时不得回退：

1. Tauri 权限最小化（12 个精确权限声明）
2. CSP connect-src 白名单限制
3. freezePrototype + withGlobalTauri: false
4. API Key 加密 v2（随机 Nonce）
5. 路径验证 5 层防御体系
6. 日志脱敏系统（正则全覆盖）
7. 100% 参数化 SQL 查询
8. spawn_blocking 正确使用（CPU 密集操作）
9. panic = "abort" + strip = true (release)
10. .gitignore 敏感文件排除完整

---

> **文档维护者**: 虫群意志（Swarm Will）
> **下次评审**: Phase 0 完成后或收到新的战略命令
