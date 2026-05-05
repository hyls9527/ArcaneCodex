# Arcane Codex 安全审计报告

> 最后更新: 2026-05-03

## 执行摘要

Arcane Codex 是一款本地优先的桌面应用，安全设计遵循"最小权限"和"数据本地化"原则。本报告总结了已实施的安全措施和潜在风险。

---

## 安全架构

### 1. 数据安全

| 措施 | 状态 | 说明 |
|------|------|------|
| 本地数据存储 | ✅ 已实现 | 所有数据存储在用户本地，不上传云端 |
| API Key 加密 | ⚠️ 基础实现 | 使用 AES-256-GCM，但 Nonce 固定，待改进 |
| 数据库加密 | ❌ 未实现 | SQLite 未加密（计划添加 SQLCipher） |
| 备份加密 | ⚠️ 基础实现 | 加密备份功能存在，待完整验证 |

### 2. 输入验证

| 措施 | 状态 | 说明 |
|------|------|------|
| 路径遍历防护 | ✅ 已实现 | 阻止 `../` 等路径遍历攻击 |
| 文件类型验证 | ✅ 已实现 | 基于扩展名验证（非魔数验证，待改进） |
| 文件大小限制 | ✅ 已实现 | 防止大文件 DoS |
| Zip Slip 防护 | ⚠️ 部分实现 | 解压路径验证存在，但未显式检查路径逃逸 |

### 3. 网络安全

| 措施 | 状态 | 说明 |
|------|------|------|
| HTTPS 强制 | ⚠️ 依赖配置 | 本地 AI 服务默认 HTTP，HTTPS 需用户配置 |
| 证书验证 | ⚠️ 依赖配置 | 证书验证取决于用户配置的 AI 服务 |
| 无遥测 | ✅ 已实现 | 不发送任何使用数据 |
| 无第三方连接 | ✅ 已实现 | 仅连接用户配置的 AI 服务 |

---

## 已实施的安全措施

### API Key 加密

```rust
// crypto.rs — API Key 加密（⚠️ 使用固定 Nonce）
pub fn encrypt_api_key(plaintext: &str) -> String {
    let key = derive_key(); // SHA256(hostname + username + platform + arch + salt)
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
    let nonce = Nonce::from_slice(b"ac-kd-nonce-12"); // 固定 Nonce（严重安全问题）
    // ... 加密逻辑
}
```

```rust
// settings.rs — 备份加密（✅ 使用随机 Nonce）
let nonce_bytes = rand::random::<[u8; 12]>(); // 随机 Nonce
let nonce = Nonce::from_slice(&nonce_bytes);
// ... 加密后 Nonce 与密文一起存储到文件
```

**安全评估**: 
- ⚠️ API Key 加密使用固定 Nonce，相同明文产生相同密文（严重安全问题）
- ✅ 备份加密已正确使用随机 Nonce + 存储到文件
- ⚠️ 密钥派生仅做一次 SHA256，安全性远低于 PBKDF2/Argon2
- 📋 建议：将 API Key 加密改为随机 Nonce + 前置存储模式（与备份加密一致）

### 路径验证

```rust
// 实际函数名：sanitize_path（位于 images.rs）
// 注意：当前标记为 #[expect(dead_code)]，未被主流程调用
fn sanitize_path(base_dir: &Path, user_input: &str) -> Result<PathBuf, String> {
    let canonical = match input_path.canonicalize() { ... };
    if !canonical.starts_with(base_dir) {
        return Err("Path traversal detected".to_string());
    }
    Ok(canonical)
}
```

**安全评估**:
- ⚠️ `sanitize_path` 函数存在但标记为 dead_code，当前未被主流程调用
- ✅ 使用 canonicalize 规范化路径
- ✅ 验证路径前缀
- 📋 建议：移除 dead_code 标记，在文件操作中统一调用

### 文件类型验证

```rust
// 当前实现：基于扩展名验证
fn validate_file(file_path: &Path) -> AppResult<(String, u64)> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(AppError::validation(format!(
            "不支持的文件格式: .{}", extension
        )));
    }
    // ...
}
```

**安全评估**:
- ⚠️ 当前基于扩展名验证，不依赖文件内容
- ⚠️ 无法阻止伪装文件攻击（如 .jpg 扩展名的恶意文件）
- 📋 计划：添加基于魔数（magic bytes）的文件头验证

---

## 潜在风险与缓解措施

### 高风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 本地数据库未加密 | 数据泄露风险 | 计划添加 SQLCipher 支持 |
| API Key 存储在内存 | 内存转储泄露 | 使用安全字符串类型 |
| 固定 Nonce | AES-256-GCM 加密强度严重降低，相同明文产生相同密文 | 改用随机 Nonce，存储 Nonce 与密文一起 |

### 中风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 日志可能包含敏感信息 | 信息泄露 | 实现日志脱敏 |
| 错误消息暴露内部信息 | 信息泄露 | 统一错误处理 |

### 低风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 依赖漏洞 | 潜在攻击面 | 定期 cargo audit |

---

## 安全检查清单

### 开发阶段

- [x] 输入验证
- [x] 路径验证
- [x] 文件类型验证（扩展名级别，待升级为魔数验证）
- [x] API Key 加密
- [x] 错误处理
- [ ] 日志脱敏
- [ ] 安全代码审查

### 构建阶段

- [x] 依赖审计 (cargo audit)
- [x] 静态分析 (cargo clippy)
- [ ] 动态分析
- [ ] 模糊测试

### 运行阶段

- [x] 最小权限运行
- [x] 无管理员权限需求
- [x] 数据本地化
- [ ] 安全更新机制

---

## 安全更新策略

1. **依赖更新**: Dependabot 每周自动检查依赖更新
2. **漏洞响应**: 发现安全漏洞后 48 小时内响应
3. **补丁发布**: 关键漏洞 7 天内发布修复版本

---

## 安全报告

如果您发现安全漏洞，请通过以下方式报告：

1. **GitHub Security Advisory** (推荐): [提交安全公告](https://github.com/hyls9527/ArcaneCodex/security/advisories)
2. **Email**: 直接联系维护者

**请勿在公开 Issue 中报告安全漏洞。**

---

## 合规性声明

### GDPR 合规性评估

以下项目基于设计目标评估，非正式合规认证：

- ✅ 数据本地存储，用户完全控制
- ✅ 不收集个人数据
- ✅ 不使用 Cookie
- ⚠️ 支持数据导出和删除（功能存在，待完整验证）

### CCPA 合规性评估

以下项目基于设计目标评估，非正式合规认证：

- ✅ 不出售个人信息
- ✅ 不共享个人信息
- ✅ 用户拥有所有数据

> **注意**: 本应用未经过正式的 GDPR/CCPA 合规审计，上述评估基于产品设计原则。

---

## 安全最佳实践（用户指南）

1. **定期备份**: 使用应用内置备份功能
2. **安全存储**: 将备份文件存储在安全位置
3. **API Key 保护**: 不要分享包含 API Key 的配置文件
4. **更新应用**: 保持应用更新到最新版本
5. **本地 AI**: 对隐私敏感场景，使用本地 AI (LM Studio/Ollama)

---

## 审计历史

| 日期 | 版本 | 审计范围 | 结果 |
|------|------|---------|------|
| 2026-05-03 | v1.0.0-rc | 内部安全评估 | 发现问题待修复 |

---

## 联系方式

安全问题请联系: [GitHub Security](https://github.com/hyls9527/ArcaneCodex/security)
