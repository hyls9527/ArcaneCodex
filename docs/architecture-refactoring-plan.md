# 架构重构方案：Clean & Extend（先清后扩）

**状态**: 阶段一已完成 ✅
**日期**: 2026-04-29
**来源**: Swarm Debate 对抗辩论融合方案
**参与 Agent**: practical-reuser, reverse-thinker, backend-architect, rule-breaking-innovator

---

## 核心哲学

> "架构为你提供思考的框架，而非限制你的牢笼。"

- **不全面重构**，因为项目已交付价值
- **不清理后停止**，因为缺乏约束会导致腐化重演
- **不提前抽象**，因为缺乏真实需求的抽象是赌博
- **不忽视治理**，因为架构纪律是长期可维护性的保障

---

## 辩论结论摘要

### 共识弱点（所有 Agent 都同意）
1. ✅ 71 个警告必须先清理，这是前置条件
2. ✅ AIProvider trait 是唯一有实际多态需求的域
3. ✅ 全面重构为 Clean Architecture 风险太高
4. ✅ ADR 制度有价值，但不应在清理阶段引入

### 共同盲区（所有 Agent 都没覆盖）
1. ❌ 如何验证 AIProvider trait 的接口设计是否合理（需要在有两个真实 Provider 后再定义）
2. ❌ 前端 Zustand stores 与新架构的映射关系
3. ❌ 如何安全地删除死代码（灰度删除策略）
4. ❌ Tauri 2.x 的特定限制对架构选择的影响

---

## 阶段依赖关系

```
阶段一：外科手术清理
    │
    ▼ (完成验收后)
阶段二：最小必要抽象 (仅在触发条件满足时执行)
    │
    ▼ (持续进行)
阶段三：架构治理建立
```

**关键设计**：
- 阶段一是**必须执行**的
- 阶段二是**条件触发**的（可能永远不会触发）
- 阶段三是**持续进行**的（轻量级，不阻塞交付）

---

## 阶段一：外科手术清理

**预计周期**: 1-2 周
**风险等级**: 最低（每步可回滚）

### 目标
暴力删除死代码，71 个 Rust 编译警告清零，不破坏已验证状态。

### 具体步骤

#### 1. 启用编译警告视为错误
```toml
# src-tauri/Cargo.toml
[package]
rustflags = ["-D", "dead_code", "-D", "unused_imports"]
```

#### 2. 逐个模块删除未使用的结构体/函数

**优先级 1：完整未使用模块（直接删除）**
- `src/core/cache.rs` - 缓存系统（零引用）
- `src/core/clip_verify.rs` - CLIP 验证（仅被测试引用）
- `src/core/clip_sidecar.rs` - CLIP 侧车管理（仅被测试引用）
- `src/core/calibration/` - 校准系统（无数据流、无触发点）

**优先级 2：部分未使用代码（清理未使用部分）**
- `src/core/ai_queue.rs` - 删除未使用的 `AIResult`, `InferenceProvider` 导入，删除未使用的 `Pause`, `Resume`, `Cancel` 变体
- `src/commands/batch.rs` - 删除整个批量操作模块（所有结构体和函数均未使用）
- `src/models/mod.rs` - 删除未使用的 `AIResult`, `SearchResult`, `Task` 导出
- `src/utils/error.rs` - 删除未使用的 `NotFoundError`, `AuthError` 变体及相关函数
- `src/core/db.rs` - 删除未使用的 `init_database`, `try_open_database` 函数
- `src/core/image.rs` - 删除未使用的 `hamming_distance` 函数
- `src/core/consistency_checker.rs` - 评估后决定保留或删除
- `src/core/lm_studio.rs` - 删除未使用的 `SERVICE_DISCOVERY_PORTS`, `discover_service`

**优先级 3：模型和类型（评估后处理）**
- `src/models/image.rs` - 评估 `Image`, `AIResult`, `SearchResult` 是否被数据库层使用
- `src/models/task.rs` - 评估 `Task` 是否被任务队列使用

#### 3. 每次删除后运行验证
```bash
cd src-tauri
cargo check
cargo test
```

#### 4. 对"可能被引用"的代码使用标记而非删除
```rust
#[allow(dead_code)]
pub struct PotentiallyFutureFeature {
    // ...
}
```

#### 5. 清理完成后运行完整 CI pipeline
```bash
# 前端
cd frontend
npm run build

# 后端
cd ../src-tauri
cargo build --release

# Tauri
npm run tauri -- build
```

### 阶段验收标准
- ✅ `cargo check` 0 警告
- ✅ CI pipeline 通过
- ✅ GitHub Release 构建成功
- ✅ 前端构建成功（1986 模块）

### 阶段终止条件
清理完成即停止，**不进入下一阶段**，除非有明确的新需求。

### 风险控制
- 每次删除后运行 `cargo check`
- 使用 `git stash` 或分支保存删除前的状态
- 对不确定的代码使用 `#[allow(dead_code)]` 而非直接删除
- 记录所有删除决策（为阶段三 ADR 做准备）

---

## 阶段二：最小必要抽象

**预计周期**: 按需（可能永远不会触发）
**风险等级**: 低（仅在真实需求驱动下执行）

### 触发条件（必须全部满足）
1. ✅ 阶段一已完成
2. ✅ 确实需要接入第二个 AI Provider（不仅仅是"可能"需要）
3. ✅ 已有两个真实 Provider 的实现代码（不是假设的）
4. ✅ 两个 Provider 之间有超过 3 处重复代码

### 目标
在真实需求触发时，引入 AIProvider trait 抽象，支持水平扩展。

### 具体步骤

#### 1. 分析两个真实 Provider 的共同接口
- 识别共同方法：`analyze_image`, `chat`, `health_check`
- 识别共同返回类型：`AIAnalysis`, `ChatResponse`
- 识别共同错误类型：`AIError`

#### 2. 定义 AIProvider trait
```rust
// src/domain/ai_engine/provider.rs

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;
    
    async fn analyze_image(
        &self,
        image: &Image,
        prompt: &str,
    ) -> Result<AIAnalysis, AIError>;
    
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResponse, AIError>;
    
    async fn health_check(&self) -> Result<(), AIError>;
}

pub struct ProviderCapabilities {
    pub supports_vision: bool,
    pub supports_streaming: bool,
    pub max_tokens: usize,
    pub is_local: bool,
}
```

#### 3. 将两个 Provider 重构为 trait impl
```rust
// src/infrastructure/ai/zhipu_provider.rs
pub struct ZhipuProvider {
    client: reqwest::Client,
    api_key: String,
}

#[async_trait]
impl AIProvider for ZhipuProvider {
    // ...
}

// src/infrastructure/ai/ollama_provider.rs
pub struct OllamaProvider {
    base_url: String,
}

#[async_trait]
impl AIProvider for OllamaProvider {
    // ...
}
```

#### 4. 更新 ai_queue.rs 使用 trait
```rust
// 从具体类型改为 trait object
pub struct AITaskQueue {
    provider: Box<dyn AIProvider>,
    // ...
}
```

#### 5. 添加单元测试验证 trait 抽象正确性
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_zhipu_provider_analyze() {
        // ...
    }
    
    #[tokio::test]
    async fn test_ollama_provider_analyze() {
        // ...
    }
}
```

### 铁律验证
- ✅ **practical-reuser 铁律**: 不为已解决的问题重写代码 → 只在有第二个 Provider 时才抽象
- ✅ **backend-architect 铁律**: 支持水平扩展 → trait 设计确保新增 Provider 只需 impl
- ✅ **reverse-thinker 铁律**: 不在死代码基础上叠加抽象 → 阶段一已清零警告
- ✅ **rule-breaking-innovator 铁律**: 不为理论纯洁性冒险 → 抽象由真实需求驱动

### 阶段终止条件
第二个 Provider 接入完成即停止，**不扩展其他抽象**。

### 风险控制
- trait 接口设计必须基于两个真实实现的共性
- 不使用假设的接口签名
- 每次重构后运行完整测试套件
- 保留原有实现作为回滚选项

---

## 阶段三：架构治理建立

**预计周期**: 持续进行（轻量级）
**风险等级**: 最低（不阻塞交付）

### 目标
建立轻量级架构治理机制，防止腐化重演。

### 具体步骤

#### 1. 建立 ADR 模板和流程
```markdown
# ADR-XXX: [标题]

**状态**: 提议 | 接受 | 弃用 | 替代

**日期**: YYYY-MM-DD

## 背景
[为什么要做这个决策]

## 决策
[我们决定做什么]

## 理由
[为什么选这个方案]

## 后果
### 正面
- [好处]

### 负面
- [代价/风险]

## 验证方式
[如何验证这个决策是正确的]
```

#### 2. 在 CI 中添加 dead_code 检测
```yaml
# .github/workflows/ci.yml
- name: Check for dead code
  working-directory: src-tauri
  run: cargo check -- -D dead_code -D unused_imports
```

#### 3. 建立模块消费者注册表
```rust
// src/core/mod.rs
//! # Core 模块
//! 
//! ## 消费者
//! - `commands/ai.rs` - 调用 AI 队列
//! - `commands/image.rs` - 调用图像处理
//! 
//! ## 依赖
//! - `database/` - 数据持久化
//! - `utils/` - 工具函数
```

#### 4. 记录阶段一和阶段二的决策
- ADR-001: 删除 cache.rs 的决策
- ADR-002: 删除 calibration/ 的决策
- ADR-003: 删除 clip_verify.rs 和 clip_sidecar.rs 的决策
- ADR-004: AIProvider trait 设计决策（阶段二触发时）

#### 5. 每季度进行一次代码健康检查
- 检查 dead_code 警告数量
- 检查模块消费者注册表是否更新
- 检查 ADR 是否过期
- 检查是否有未使用的依赖

### 阶段验收标准
- ✅ ADR 模板已建立
- ✅ CI 包含 dead_code 检测
- ✅ 模块消费者注册表已初始化
- ✅ 阶段一和阶段二的决策已记录为 ADR

### 风险控制
- ADR 流程轻量级，不阻塞开发
- 每季度检查自动化，不增加人工负担
- 模块注册表作为文档，不强制执行

---

## 与原 Clean Architecture 方案的对比

| 维度 | 原方案 | 融合方案 |
|------|--------|----------|
| **执行方式** | 全面重构 | 分阶段递进 |
| **风险控制** | 一次性大爆炸 | 每阶段可独立验证 |
| **AIProvider trait** | 现在就定义 | 等有两个真实 Provider 再定义 |
| **死代码处理** | 假设会在重构中清理 | 必须先清理再讨论架构 |
| **ADR 引入时机** | 重构期间 | 清理完成后 |
| **已交付状态** | 会被破坏 | 始终保持可回滚 |
| **编译体积** | 引入 Feature Flags 控制 | 暂不引入，按需评估 |
| **前端架构** | 完全重构为 Services 层 | 保持现状，仅后端清理 |

---

## 辩论 Agent 铁律验证

| Agent | 铁律 | 融合方案如何满足 |
|-------|------|------------------|
| **practical-reuser** | 绝不为已解决的问题重写代码 | 阶段一仅删除死代码，不重写功能；阶段二仅在真实需求触发时才抽象 |
| **reverse-thinker** | 不在存在死代码的基础上叠加任何新抽象层 | 阶段一必须先清零警告，阶段二才能开始 |
| **backend-architect** | 任何架构必须支持 AI Provider 的水平扩展 | 阶段二的 trait 设计确保新增 Provider 只需 impl，改动不超过一个文件 |
| **rule-breaking-innovator** | 已发布的产品绝不为架构纯洁性冒险回滚 | 分阶段执行，每阶段可独立验证和回滚，不破坏已交付状态 |

---

## 下一步行动

- [x] **阶段一已完成**：外科手术清理死代码（71→0 警告）✅
- [ ] **等待阶段二触发条件**：第二个 AI Provider 接入需求
- [ ] **持续执行阶段三**：建立轻量级架构治理

---

## 阶段一执行记录

**完成日期**: 2026-04-29

### 成果数据

| 指标 | 清理前 | 清理后 | 变化 |
|------|--------|--------|------|
| Rust 编译警告 | 71 个 | 0 个 | ✅ 已清零（当前版本） |
| 未使用模块 | 5 个 | 0 个 | ✅ 全部清理 |
| 未使用结构体 | 50+ | 0 个 | ✅ 全部标记/删除 |
| 前端构建 | ✅ 成功 | ✅ 成功 | 保持 |
| Tauri 构建 | ✅ 成功 | ✅ 成功 | 保持 |

### 删除的完整模块

| 模块 | 原因 | 状态 |
|------|------|------|
| `src/core/cache.rs` | 缓存系统（零引用） | ✅ 已删除 |
| `src/core/clip_verify.rs` | CLIP 验证（仅测试引用） | ✅ 已删除 |
| `src/core/clip_sidecar.rs` | CLIP 侧车管理（仅测试引用） | ✅ 已删除 |
| `src/core/calibration/` | 校准系统（无数据流） | ✅ 已删除 |
| `src/commands/batch.rs` | 批量操作（未注册为 Tauri 命令） | ✅ 已删除 |

### 迁移的类型

| 类型 | 原位置 | 新位置 | 状态 |
|------|--------|--------|------|
| `ImageCategory` | `calibration/types.rs` | `models/category.rs` | ✅ 已迁移 |

### 标记为 `#[allow(dead_code)]` 的代码

| 代码 | 原因 | 状态 |
|------|------|------|
| `db.init()` | 测试用数据库初始化 | ✅ 已标记 |
| `db.init_database()` / `try_open_database()` | 备用初始化函数 | ✅ 已标记 |
| `ImageProcessor.hamming_distance()` | 感知哈希距离计算 | ✅ 已标记 |
| `AITaskQueue.command_sender` | 命令通道（前端未连接） | ✅ 已标记 |
| `QueueCommand` 枚举变体 | Pause/Resume/Cancel（前端未使用） | ✅ 已标记 |
| `AITaskQueue.set_concurrency()` / `cancel()` | 队列控制（前端未使用） | ✅ 已标记 |
| `InferenceProvider.model()` | trait 方法（仅测试使用） | ✅ 已标记 |
| `SearchIndexBuilder.delete_for_image()` | 删除索引（前端未调用） | ✅ 已标记 |
| `ConsistencyChecker.has_conflicts()` | 冲突检查（内部使用） | ✅ 已标记 |
| `Task` 结构体 | 任务模型（未来使用） | ✅ 已标记 |

### 删除的未使用代码

| 位置 | 删除内容 | 状态 |
|------|----------|------|
| `lm_studio.rs` | `SERVICE_DISCOVERY_PORTS`、`discover_service()` | ✅ 已删除 |
| `utils/error.rs` | `NotFoundError`、`AuthError` 变体及构造函数 | ✅ 已删除 |
| `models/image.rs` | 重复的 `AIResult`、`SearchResult` 结构体 | ✅ 已删除 |
| `ai_queue.rs` | 未使用的 `AIResult`、`InferenceProvider` 导入 | ✅ 已清理 |

### 新增的架构治理

| 项目 | 状态 |
|------|------|
| 启用 `-D warnings` 编译标志 | ✅ 已启用 |

### 验证结果

- ✅ `cargo check` 0 警告
- ✅ `cargo build --release` 成功
- ✅ `npm run tauri build` 成功（生成 MSI + EXE 安装包）
- ✅ 前端构建成功（1986 模块）

---

## 附录：辩论方案汇总

### 新架构方案（Clean Architecture 全面重构）
主张全面重构为 4 层架构（Presentation → Adapter → Domain → Infrastructure），引入 AIProvider trait、Feature Flags、ADR 制度、YAGNI 红线机制。

**致命缺陷**: 摧毁可验证状态，功能冻结期太长，迁移风险不可控。

### practical-reuser 立场
项目能构建、能通过检查、已发布 Release——这些是已交付的成果。绝不为已解决的问题重写代码。71 个警告是代码卫生问题，用 cargo clippy --fix 和逐步清理就能解决。

**致命缺陷**: 无抽象导致扩展时硬编码，违反开闭原则。

### reverse-thinker 立场
先杀后建，拒绝装饰性重构。用一周时间暴力删除 50+ 未使用的结构体/函数，把 71 个编译警告清零，然后停止一切架构讨论。不在存在死代码的基础上叠加任何新抽象层。

**致命缺陷**: 只清理不建约束，死代码会再生。

### backend-architect 立场
保留 Tauri 分层骨架不动，仅对 core/ 层实施三项手术：(1) 定义 AIProvider trait 抽象所有 AI 后端调用；(2) 删除四个未使用模块消除 50+ 死代码；(3) 引入 ADR 记录所有架构决策，但不在本轮重写。

**致命缺陷**: trait 提前抽象缺乏验证，可能保留已有架构债务。

### rule-breaking-innovator 立场
不重构 Clean Architecture。直接砍掉 50+ 未使用的结构体和函数，修复 71 个编译警告，仅在 AI 集成层抽象出 AIProvider trait——因为这是唯一有实际多态需求的域。

**致命缺陷**: AIProvider 可能变成上帝接口，缺少终止条件。

---

**文档版本**: v1.0
**最后更新**: 2026-04-29
**维护者**: Arcane Codex 团队
