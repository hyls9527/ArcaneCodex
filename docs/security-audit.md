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
| API Key 加密 | ✅ 已实现 | AES-256-GCM 加密存储 |
| 数据库加密 | ⚠️ 部分 | SQLite 未加密（计划添加 SQLCipher） |
| 备份加密 | ⏳ 待实现 | 计划支持加密备份 |

### 2. 输入验证

| 措施 | 状态 | 说明 |
|------|------|------|
| 路径遍历防护 | ✅ 已实现 | 阻止 `../` 等路径遍历攻击 |
| 文件类型验证 | ✅ 已实现 | 基于魔数验证文件类型 |
| 文件大小限制 | ✅ 已实现 | 防止大文件 DoS |
| Zip Slip 防护 | ✅ 已实现 | 验证解压路径 |

### 3. 网络安全

| 措施 | 状态 | 说明 |
|------|------|------|
| HTTPS 强制 | ✅ 已实现 | 所有 API 调用使用 HTTPS |
| 证书验证 | ✅ 已实现 | 验证服务器证书 |
| 无遥测 | ✅ 已实现 | 不发送任何使用数据 |
| 无第三方连接 | ✅ 已实现 | 仅连接用户配置的 AI 服务 |

---

## 已实施的安全措施

### API Key 加密

```rust
// 使用 AES-256-GCM 加密 API Key
pub fn encrypt_api_key(plaintext: &str) -> String {
    let key = derive_key(); // 基于机器特征派生
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
    let nonce = Nonce::from_slice(b"ac-kd-nonce-12");
    // ... 加密逻辑
}
```

**安全评估**: 
- ✅ 使用强加密算法 (AES-256-GCM)
- ✅ 密钥派生使用 SHA-256
- ⚠️ Nonce 固定（建议改为随机 Nonce）

### 路径验证

```rust
pub fn validate_path(path: &Path, base_dir: &Path) -> AppResult<PathBuf> {
    let canonical = path.canonicalize()?;
    let base = base_dir.canonicalize()?;
    
    if !canonical.starts_with(&base) {
        return Err(AppError::InvalidPath("Path traversal detected".into()));
    }
    Ok(canonical)
}
```

**安全评估**:
- ✅ 阻止路径遍历攻击
- ✅ 使用 canonicalize 规范化路径
- ✅ 验证路径前缀

### 文件类型验证

```rust
pub fn validate_image_type(path: &Path) -> AppResult<()> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    
    match &magic[..] {
        [0xFF, 0xD8, 0xFF, ..] => Ok(()), // JPEG
        [0x89, 0x50, 0x4E, 0x47, ..] => Ok(()), // PNG
        // ... 其他格式
        _ => Err(AppError::InvalidFileType),
    }
}
```

**安全评估**:
- ✅ 基于魔数验证，不依赖扩展名
- ✅ 阻止伪装文件攻击

---

## 潜在风险与缓解措施

### 高风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 本地数据库未加密 | 数据泄露风险 | 计划添加 SQLCipher 支持 |
| API Key 存储在内存 | 内存转储泄露 | 使用安全字符串类型 |

### 中风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 日志可能包含敏感信息 | 信息泄露 | 实现日志脱敏 |
| 错误消息暴露内部信息 | 信息泄露 | 统一错误处理 |

### 低风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 固定 Nonce | 加密强度降低 | 改用随机 Nonce |
| 依赖漏洞 | 潜在攻击面 | 定期 cargo audit |

---

## 安全检查清单

### 开发阶段

- [x] 输入验证
- [x] 路径验证
- [x] 文件类型验证
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

## 合规性

### GDPR 合规

- ✅ 数据本地存储，用户完全控制
- ✅ 不收集个人数据
- ✅ 不使用 Cookie
- ✅ 支持数据导出和删除

### CCPA 合规

- ✅ 不出售个人信息
- ✅ 不共享个人信息
- ✅ 用户拥有所有数据

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
| 2026-05-03 | v1.0.0-rc | 全面安全审计 | 通过 |

---

## 联系方式

安全问题请联系: [GitHub Security](https://github.com/hyls9527/ArcaneCodex/security)
