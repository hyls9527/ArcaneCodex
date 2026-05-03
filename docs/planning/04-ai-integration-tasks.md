# AI 集成架构实现任务清单

> 基于辩论结论的分层自适应架构

---

## 任务 1：创建架构设计文档 ✅
**状态**：已完成  
**产出**：`docs/planning/ai-integration-architecture.md`

---

## 任务 2：实现前端路由层
**状态**：待执行  
**优先级**：高  
**文件**：`frontend/src/lib/ai-router.ts`

### 需求
- 检测模型类型（本地/云端）
- 自动选择直连或代理模式
- 统一接口封装

### 伪代码
```typescript
// ai-router.ts
export type ModelProvider = 
  | { type: 'local'; endpoint: string }  // Ollama/LM Studio
  | { type: 'cloud'; provider: 'zhipu' | 'openrouter' };  // 需代理

export async function* streamChat(
  provider: ModelProvider,
  messages: Message[]
): AsyncGenerator<string> {
  if (provider.type === 'local') {
    // 直连模式
    yield* streamLocal(provider.endpoint, messages);
  } else {
    // 代理模式
    yield* invoke('stream_chat_proxy', { provider, messages });
  }
}
```

---

## 任务 3：实现 Tauri 主进程代理
**状态**：待执行  
**优先级**：高  
**文件**：`src-tauri/src/commands/ai_proxy.rs`

### 需求
- 云端 API 统一中转
- 流式响应透传
- 错误处理与重试

### 伪代码
```rust
// ai_proxy.rs
#[tauri::command]
async fn stream_chat_proxy(
    provider: CloudProvider,
    messages: Vec<Message>,
) -> Result<StreamResponse, Error> {
    // 1. 从密钥链获取 API Key（临时）
    let api_key = keyvault::get_key(&provider).await?;
    
    // 2. 发送请求
    let stream = http_client::stream_chat(&provider, &api_key, &messages).await?;
    
    // 3. 立即清除密钥
    drop(api_key);
    
    // 4. 透传流式响应
    Ok(StreamResponse::new(stream))
}
```

---

## 任务 4：集成系统密钥链存储
**状态**：待执行  
**优先级**：高  
**文件**：`src-tauri/src/core/keyvault.rs`

### 需求
- 使用 `tauri-plugin-stronghold` 或 `keytar-rs`
- 安全存储/读取 API Key
- 跨平台支持（Windows/macOS/Linux）

### 伪代码
```rust
// keyvault.rs
use tauri_plugin_stronghold::Stronghold;

pub async fn store_key(provider: &str, api_key: &str) -> Result<()> {
    let stronghold = Stronghold::new(...);
    stronghold.insert(provider, api_key).await?;
    Ok(())
}

pub async fn get_key(provider: &str) -> Result<SecureString> {
    let stronghold = Stronghold::new(...);
    let key = stronghold.get(provider).await?;
    Ok(SecureString::new(key)) // 自动清零的字符串类型
}
```

---

## 任务 5：实现密钥"用完即焚"机制
**状态**：待执行  
**优先级**：中  
**文件**：`src-tauri/src/utils/secure_string.rs`

### 需求
- 自定义 `SecureString` 类型
- Drop 时自动清零内存
- 防止密钥长期驻留

### 伪代码
```rust
// secure_string.rs
pub struct SecureString {
    inner: Vec<u8>,
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // 显式清零内存
        for byte in &mut self.inner {
            *byte = 0;
        }
    }
}

impl SecureString {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.inner).unwrap()
    }
}
```

---

## 任务 6：添加模型配置界面
**状态**：待执行  
**优先级**：中  
**文件**：`frontend/src/components/settings/AIProviderConfig.tsx`

### 需求
- 添加/编辑模型提供商
- 本地模型：配置 endpoint
- 云端模型：输入 API Key（仅存储到密钥链）
- 测试连接按钮

### 界面草图
```
┌─────────────────────────────────────┐
│ AI 模型配置                          │
├─────────────────────────────────────┤
│ [添加提供商]                         │
│                                     │
│ ┌─ Ollama (本地) ─────────────────┐ │
│ │ 地址: http://localhost:11434    │ │
│ │ 状态: ✅ 已连接                  │ │
│ └─────────────────────────────────┘ │
│                                     │
│ ┌─ 智谱 AI (云端) ────────────────┐ │
│ │ API Key: [****************]     │ │
│ │ 模型: GLM-4-Flash               │ │
│ │ 状态: ✅ 已连接                  │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

---

## 任务 7：编写架构测试用例
**状态**：待执行  
**优先级**：中  
**文件**：`src-tauri/tests/security_test.rs`

### 需求
- 验证密钥不泄露到前端
- 验证密钥使用后立即清除
- 验证系统密钥链集成正常

### 测试用例
```rust
#[test]
fn test_api_key_not_in_frontend() {
    // 确保前端代码中不包含任何硬编码密钥
    let frontend_code = fs::read_to_string("...").unwrap();
    assert!(!frontend_code.contains("sk-")); // OpenAI 格式
    assert!(!frontend_code.contains("glm-")); // 智谱格式
}

#[test]
fn test_secure_string_zero_on_drop() {
    let ptr: *const u8;
    {
        let s = SecureString::from("secret");
        ptr = s.as_ptr();
    } // drop 后
    
    unsafe {
        assert_eq!(*ptr, 0); // 内存已清零
    }
}
```

---

## 依赖安装命令

### Rust 依赖
```bash
cd src-tauri
cargo add tauri-plugin-stronghold
cargo add reqwest --features stream
cargo add zeroize  # 安全清零内存
```

### Tauri 配置更新
```json
// tauri.conf.json
{
  "plugins": {
    "stronghold": {
      "password": "..."
    }
  }
}
```

---

## 执行顺序建议

```
任务 1 (设计文档) ✅
    ↓
任务 4 (密钥存储) → 任务 5 (安全字符串)
    ↓
任务 3 (代理层) ← 依赖密钥存储
    ↓
任务 2 (前端路由) ← 依赖代理层接口
    ↓
任务 6 (配置界面) ← 依赖前端路由
    ↓
任务 7 (测试) ← 依赖全部实现
```

---

## 验收检查清单

- [ ] 本地 Ollama 调用延迟 < 100ms (TTFT)
- [ ] 云端 API Key 不出现在前端代码/内存
- [ ] 密钥仅存在于系统密钥链，按需解密
- [ ] 支持本地/云端模型无缝切换
- [ ] 单元测试验证密钥不泄露
- [ ] 手动测试：抓包验证密钥不传输到前端
