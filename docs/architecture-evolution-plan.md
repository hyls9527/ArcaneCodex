# Arcane Codex v2.0 架构演进方案

**状态**: 规划中
**日期**: 2026-04-29
**来源**: Swarm Debate 辩论 + 三重深渊反思
**前置条件**: 阶段一已完成（71→0 警告）✅

---

## 核心哲学

> "做最少的事，但做到极致。"

- **专注**：只做真实需求驱动的事
- **克制**：不为假设的未来付费
- **轻量**：自动化一切，减少人工流程

---

## 当前架构状态

### 已完成的清理

| 指标 | 清理前 | 清理后 |
|------|--------|--------|
| Rust 编译警告 | 71 个 | 0 个 ✅ |
| 未使用模块 | 5 个 | 0 个 ✅ |
| 架构治理 | 无 | `-D warnings` ✅ |

### 当前架构概览

```
src-tauri/
├── src/
│   ├── commands/          # Tauri 命令层（10 个模块）
│   ├── core/              # 业务逻辑层（9 个模块）
│   ├── models/            # 数据模型层（3 个模块）
│   └── utils/             # 工具函数层（2 个模块）
└── tauri.conf.json        # Tauri 配置
```

**当前状态评估**：
- ✅ 代码干净，无死代码
- ✅ 编译通过，构建成功
- ✅ 功能完整，已发布 Release
- ⚠️ 模块边界仍不清晰（`core/` 包含多种职责）
- ⚠️ AI 集成逻辑分散在多个文件中
- ⚠️ 缺乏架构决策记录

---

## 阶段二：最小必要抽象

### 触发条件（必须全部满足）

| 条件 | 状态 | 说明 |
|------|------|------|
| 阶段一已完成 | ✅ 已完成 | 71→0 警告 |
| 确实需要接入第二个 AI Provider | ❌ 未满足 | 目前只有一个 Provider |
| 已有两个真实 Provider 的实现代码 | ❌ 未满足 | 只有 `inference.rs` 中的实现 |
| 两个 Provider 之间有超过 3 处重复代码 | ❌ 未满足 | 无法评估 |

**结论**：阶段二**暂不触发**。等待真实需求。

### 触发后的执行计划

当触发条件全部满足时，执行以下步骤：

#### 1. 定义 AIProvider trait

```rust
// src/core/ai_engine/provider.rs

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    
    async fn analyze_image(
        &self,
        image_path: &str,
    ) -> AppResult<AIResult>;
    
    async fn health_check(&self) -> AppResult<Vec<String>>;
}
```

**设计原则**：
- 只抽象**真实存在的共性**（基于两个 Provider 的实际代码）
- 不抽象**可能需要的功能**（如 streaming、multi-modal）
- 接口尽量**简单**，避免上帝接口

#### 2. 重构现有 Provider

将 `inference.rs` 中的实现重构为 trait impl：

```rust
// src/core/ai_engine/providers/zhipu.rs

pub struct ZhipuProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[async_trait]
impl AIProvider for ZhipuProvider {
    fn id(&self) -> &str { "zhipu" }
    fn name(&self) -> &str { "智谱 AI" }
    
    async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult> {
        // 现有实现
    }
    
    async fn health_check(&self) -> AppResult<Vec<String>> {
        // 现有实现
    }
}
```

#### 3. 更新 AI 队列使用 trait

```rust
// src/core/ai_queue.rs

pub struct AITaskQueue {
    provider: Box<dyn AIProvider>,
    // ...
}
```

#### 4. 添加测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_provider_analyze() {
        // 验证 trait 抽象正确性
    }
}
```

---

## 阶段三：轻量级架构治理

### 目标

建立**最小必要**治理机制，防止腐化重演，但不增加人工负担。

### 已完成的治理

| 项目 | 状态 | 说明 |
|------|------|------|
| `-D warnings` 编译标志 | ✅ 已启用 | 警告视为错误，自动化 |

### 计划中的治理（按需引入）

| 项目 | 优先级 | 说明 |
|------|--------|------|
| ADR 制度 | 低 | 等真正需要时再引入 |
| 模块消费者注册表 | 低 | 代码已足够清晰 |
| 季度代码健康检查 | 低 | CI 已包含 dead_code 检测 |

**决策**：阶段三**仅保持 `-D warnings`**，不引入额外治理机制，除非出现以下情况：
- 团队规模扩大到 3 人以上
- 出现架构决策争议需要记录
- 代码库规模增长到难以人工审查

---

## 架构演进路线图

```
阶段一：外科手术清理 ✅ 已完成
    │
    ▼
阶段二：最小必要抽象 ⏸️ 等待触发条件
    │
    ▼ (当第二个 AI Provider 出现时)
定义 AIProvider trait → 重构现有 Provider → 更新调用方 → 添加测试
    │
    ▼
阶段三：轻量级治理 🔄 持续进行
    │
    └─ 仅保持 -D warnings，按需引入 ADR
```

---

## 与辩论共识的对比

| 辩论共识 | 本方案如何满足 |
|----------|----------------|
| 71 个警告必须先清理 | ✅ 阶段一已完成 |
| AIProvider trait 是唯一有实际多态需求的域 | ✅ 阶段二仅在有真实需求时引入 |
| 全面重构为 Clean Architecture 风险太高 | ✅ 本方案采用渐进式演进 |
| ADR 制度有价值，但不应在清理阶段引入 | ✅ 阶段三仅保持最小治理 |

---

## 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 阶段二永远不触发 | 中 | 低 | 代码保持清晰，未来重构成本低 |
| 新开发者不理解架构决策 | 低 | 中 | 代码已足够清晰，注释充分 |
| 警告数量反弹 | 低 | 低 | `-D warnings` 已启用，CI 自动拦截 |

---

## 总结

**本方案的核心**：

> 不全面重构，不提前抽象，不引入过重治理。
> 
> **只做真实需求驱动的事，但做到极致。**

**当前状态**：
- ✅ 阶段一已完成（代码干净）
- ⏸️ 阶段二等待触发（第二个 AI Provider）
- 🔄 阶段三轻量运行（仅 `-D warnings`）

**下一步**：
- 保持代码清洁
- 等待真实需求
- 不主动引入抽象或治理

---

**文档版本**: v1.0
**最后更新**: 2026-04-29
**维护者**: Arcane Codex 团队
