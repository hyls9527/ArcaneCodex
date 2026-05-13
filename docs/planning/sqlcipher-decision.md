# SQLite 加密方案决策文档

> 评估日期：2026-05-13 | 评估人：合规检查器 | 版本：v1.0

---

## 一、现状分析

### 1.1 当前数据库架构

ArcaneCodex 使用 SQLite 作为嵌入式数据库，通过 rusqlite 0.31 + r2d2 连接池访问。

**关键配置**（[Cargo.toml](file:///e:/ArcaneCodex/src-tauri/Cargo.toml#L22)）：
```toml
rusqlite = { version = "0.31", features = ["bundled", "chrono"] }
r2d2 = "0.8"
r2d2_sqlite = "0.24"
```

**数据库初始化**（[db.rs](file:///e:/ArcaneCodex/src-tauri/src/core/db.rs#L20-L24)）：
```rust
const PRAGMA_CONFIG: &'static str = "
    PRAGMA journal_mode=WAL;
    PRAGMA foreign_keys=ON;
    PRAGMA busy_timeout=5000;
";
```

**数据库路径**：`%APPDATA%\ArcaneCodex\arcanecodex.db`（明文存储）

**当前迁移版本**：v8（8 个迁移，13 张表）

### 1.2 敏感数据清单

通过代码审计，识别出以下敏感数据：

| 敏感度 | 数据 | 存储位置 | 当前保护 | 暴露风险 |
|--------|------|----------|----------|----------|
| **高** | AI 推理 API Key | `settings` 表 `inference_api_key` 行 | AES-256-GCM v2 加密（[crypto.rs](file:///e:/ArcaneCodex/src-tauri/src/utils/crypto.rs#L30)） | 密文存储在明文 DB 中，密钥由机器指纹派生 |
| **中** | 图片文件路径 | `images.file_path` | 无 | 暴露用户目录结构和文件名 |
| **中** | EXIF 元数据 | `images.exif_data` (JSON) | 无 | 可能含 GPS 坐标、拍摄时间、设备信息 |
| **中** | AI 分析描述 | `images.ai_description` | 无 | 可能含隐私内容描述 |
| **低** | AI 标签 | `images.ai_tags` (JSON) | 无 | 内容分类信息 |
| **低** | 叙事内容 | `narratives.content` | 无 | AI 生成的图片叙事 |
| **低** | 知识图谱属性 | `kg_nodes.properties_json` | 无 | 实体属性数据 |
| **低** | 系统配置 | `settings` 表其他行 | 无 | 推理提供者、模型名称、超时等 |

**关键发现**：

1. **API Key 已有应用层加密**：`encrypt_api_key()` 使用 AES-256-GCM + 随机 Nonce（v2），密钥由 `hostname:username:platform:arch` + 固定盐派生。v1 固定 Nonce 漏洞已废弃。
2. **EXIF 数据是最大隐私风险**：GPS 坐标可直接定位用户位置，但该字段需要参与搜索索引，加密会破坏查询能力。
3. **文件路径暴露目录结构**：攻击者可推断用户名、工作目录等信息。

### 1.3 威胁模型

| 威胁 | 场景 | 可能性 | 影响 |
|------|------|--------|------|
| 设备丢失/被盗 | 笔记本丢失，攻击者直接读取 DB 文件 | 中 | 高：EXIF GPS + 文件路径 + API Key 密文 |
| 恶意软件读取 | 恶意软件扫描 `%APPDATA%` 目录 | 中 | 高：同上 |
| 同机用户读取 | 共享 Windows 账户的其他人 | 低 | 中：文件路径 + EXIF |
| 远程攻击 | 通过网络获取 DB 文件 | 低 | 高：全部数据 |
| 备份泄露 | 用户将未加密备份上传到云存储 | 中 | 高：全部数据 |

**威胁模型总结**：ArcaneCodex 是本地优先桌面应用，主要威胁是**物理访问**和**恶意软件**。数据库文件位于用户可读目录，任何能读取文件系统的进程都能获取全部明文数据（API Key 除外，已有加密）。

---

## 二、方案对比

### 2.1 方案 A：SQLCipher (rusqlite bundled-sqlcipher)

**技术方案**：将 `rusqlite` 的 feature 从 `bundled` 切换为 `bundled-sqlcipher`（或 `bundled-sqlcipher-vendored-openssl`），在连接初始化时通过 `PRAGMA key` 设置加密密钥。

**rusqlite 0.31 兼容性**：已确认 `bundled-sqlcipher` feature 在 rusqlite 0.31 中可用。[source: crates.io/crates/rusqlite, 置信度: 已确认]

**代码变更示例**：
```toml
# Cargo.toml
rusqlite = { version = "0.31", features = ["bundled-sqlcipher-vendored-openssl", "chrono"] }
```
```rust
// db.rs - 连接初始化
fn create_pool(db_path: &PathBuf) -> Result<SqlitePool> {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.pragma_update(None, "key", &derive_db_key())?;  // 新增
        conn.execute_batch(Self::PRAGMA_CONFIG)?;
        Ok(())
    });
    // ...
}
```

| 维度 | 评估 |
|------|------|
| **安全性** | **高** - 全库 AES-256-CBC 加密 + HMAC-SHA512 完整性校验，文件头不可识别，无密钥无法读取任何数据 |
| **性能** | **中等影响** - SQLCipher 官方基准：CRUD 操作约 5-15% 开销 [source: zetetic.net/sqlcipher/performance, 置信度: 已确认]。实测数据：顺序写入 +13.6%，随机读取 +10.8%，批量更新 +8.8% [source: toutiao.com 基准测试, 置信度: 猜测]。首次连接密钥派生（PBKDF2 256000 轮）约 200-500ms，可用 RAW key 规避 |
| **编译影响** | **显著** - 需要编译 OpenSSL（vendored）+ SQLCipher C 代码。预估增量编译时间 +2-5 分钟，release 二进制大小 +1-2 MB [source: github.com/keychainpgp/issues/24, 置信度: 猜测]。CI 构建时间显著增加 |
| **迁移成本** | **中等** - 需编写迁移逻辑：检测明文 DB → 导出数据 → 用 SQLCipher 重建 → 导入。`PRAGMA key` 必须在任何 SQL 操作前执行。需处理密钥管理（派生/存储/丢失恢复） |
| **查询能力** | **完全保留** - 透明加密，所有 SQL 查询、索引、全文搜索正常工作。jieba 分词 + 倒排索引不受影响 |
| **合规性** | **优秀** - AES-256 加密满足 GDPR 第 32 条"适当技术措施"要求、CCPA 合理安全程序要求 |

### 2.2 方案 B：应用层加密（敏感字段单独 AES 加密）

**技术方案**：复用现有 `crypto.rs` 的 AES-256-GCM 加密，对 EXIF 数据、文件路径等敏感字段在写入前加密、读取后解密。

**已有基础设施**：
- [crypto.rs](file:///e:/ArcaneCodex/src-tauri/src/utils/crypto.rs) - AES-256-GCM 加解密，v2 随机 Nonce
- `encrypt_api_key()` / `decrypt_api_key()` - 已用于 API Key 保护
- 机器指纹密钥派生 - `hostname:username:platform:arch` + 盐

| 维度 | 评估 |
|------|------|
| **安全性** | **中等** - 仅保护指定字段，非敏感数据仍明文。数据库文件头仍可识别为 SQLite。攻击者可看到表结构、标签分布、时间戳等元数据。密钥派生依赖机器指纹，同机攻击者可复现密钥 |
| **性能** | **低影响** - 仅加解密少量字段，每次操作增加微秒级延迟。不影响批量查询性能 |
| **编译影响** | **零** - 无需新增依赖，`aes-gcm` 已在 Cargo.toml 中 |
| **迁移成本** | **低** - 逐字段添加加密/解密逻辑，可分批迁移。需修改 `images` 表的读写代码（约 10+ 处 SQL 查询） |
| **查询能力** | **严重受损** - 加密字段无法建立索引、无法参与 WHERE 条件、无法排序。EXIF 数据加密后无法搜索 GPS 位置；文件路径加密后无法按路径过滤。这是此方案的致命缺陷 |
| **合规性** | **部分满足** - 敏感字段加密满足数据最小化原则，但数据库整体仍可被识别和部分读取，不完全满足 GDPR 第 32 条"适当技术措施" |

### 2.3 方案 C：保持明文 + 文档告知用户

**技术方案**：不修改代码，在隐私政策和文档中明确告知用户数据库未加密，建议使用操作系统级加密（BitLocker/FileVault）。

| 维度 | 评估 |
|------|------|
| **安全性** | **低** - 数据库完全明文，任何文件系统访问均可读取。依赖操作系统级加密作为唯一防线 |
| **性能** | **零影响** - 无任何变更 |
| **编译影响** | **零** - 无任何变更 |
| **迁移成本** | **零** - 无任何变更 |
| **查询能力** | **完全保留** |
| **合规性** | **不足** - GDPR 第 32 条要求"适当的技术和组织措施"保护个人数据。仅依赖操作系统加密可能不被视为"适当"，尤其当应用本身存储了 GPS 坐标等敏感个人信息 |

---

## 三、方案综合对比

| 维度 | A: SQLCipher | B: 应用层加密 | C: 明文+文档 |
|------|-------------|-------------|-------------|
| **安全性** | 高（全库加密） | 中（字段级加密） | 低（无加密） |
| **性能** | 5-15% CRUD 开销 | 微秒级字段加解密 | 无影响 |
| **编译影响** | +2-5 分钟编译，+1-2 MB 二进制 | 无 | 无 |
| **迁移成本** | 中（需迁移脚本+密钥管理） | 低（逐字段修改） | 无 |
| **查询能力** | 完全保留 | 严重受损（加密字段不可索引/搜索） | 完全保留 |
| **合规性** | 优秀（满足 GDPR/CCPA） | 部分（敏感字段保护） | 不足 |
| **密钥管理** | 需新增（PRAGMA key） | 复用现有机器指纹 | 无 |
| **备份兼容** | 加密备份需密钥 | 加密字段在备份中仍加密 | 明文 |
| **开发工作量** | 约 3-5 天 | 约 2-3 天 | 0 |
| **用户体验影响** | 首次连接 +200-500ms | 无感知 | 无 |

---

## 四、推荐方案

### 推荐：方案 B（应用层加密）作为当前方案，方案 A（SQLCipher）作为路线图目标

**理由**：

1. **当前阶段的核心矛盾**：ArcaneCodex 是单人开发的本地优先桌面应用，v1.0.0 刚发布。当前最大风险不是远程攻击，而是**设备丢失场景下的 API Key 泄露**——这已被 `crypto.rs` 的 AES-256-GCM 加密覆盖。

2. **方案 A 的时机问题**：SQLCipher 引入的编译时间增加（+2-5 分钟）和 CI 复杂度对单人开发迭代效率影响显著。`bundled-sqlcipher-vendored-openssl` 需要编译 OpenSSL，这在 Windows CI 环境中可能遇到兼容性问题。

3. **方案 B 的实用性**：
   - 已有 `crypto.rs` 基础设施，扩展成本低
   - EXIF 数据的 GPS 字段可以单独提取加密，其余 EXIF 保留明文以支持搜索
   - 文件路径加密后虽然不能 SQL 搜索，但可通过应用层过滤实现

4. **方案 A 的长期价值**：当用户量增长、出现合规需求（如企业用户要求）、或项目引入多用户功能时，SQLCipher 是正确答案。

### 实施路线图

#### 阶段 1：增强应用层加密（当前，1-2 天）

- [ ] 对 `images.exif_data` 中的 GPS 坐标字段单独加密（提取 `GPSLatitude`/`GPSLongitude` → 加密存储 → 读取时解密还原）
- [ ] 对 `images.file_path` 添加可选加密（设置项控制，默认关闭）
- [ ] 在隐私设置页面添加"数据库加密"说明，告知用户当前保护级别

#### 阶段 2：SQLCipher 集成准备（v1.1 或 v1.2，3-5 天）

- [ ] 在 feature flag 后实现 SQLCipher 支持：`#[cfg(feature = "sqlcipher")]`
- [ ] 实现密钥管理：从 OS 凭据管理器读取/存储加密密钥
- [ ] 实现迁移逻辑：检测明文 DB → `PRAGMA rekey` 加密
- [ ] CI 配置：添加 `sqlcipher` feature 的独立构建任务
- [ ] 性能基准测试：对比明文 vs SQLCipher 的 CRUD 延迟

#### 阶段 3：SQLCipher 默认启用（v2.0）

- [ ] 新安装默认启用 SQLCipher
- [ ] 旧用户首次启动时自动迁移
- [ ] 提供降级路径（`PRAGMA rekey = ''` 可恢复明文）

### 风险缓解措施

| 风险 | 缓解措施 |
|------|----------|
| 机器指纹变更导致密钥失效 | 提供密钥导出/导入功能；加密备份使用用户密码而非机器指纹 |
| SQLCipher 编译失败 | 使用 `bundled-sqlcipher-vendored-openssl` 避免系统 OpenSSL 依赖；feature flag 允许降级到明文模式 |
| 迁移过程中数据丢失 | 迁移前自动创建明文备份；迁移失败时回滚 |
| 性能不达标 | 使用 RAW key 跳过 PBKDF2；调整 `kdf_iter` 和 `cipher_page_size` |
| 备份文件泄露 | 加密备份功能已实现（AES-256-GCM + 用户密码），确保备份流程始终加密 |

---

## 五、决策点（需用户确认）

1. **是否接受阶段 1 只加密 GPS 坐标，其余 EXIF 字段保留明文？**
   - 选项 A：仅加密 GPS 坐标（推荐，平衡隐私与搜索能力）
   - 选项 B：加密整个 EXIF JSON（更安全但丧失 EXIF 搜索能力）
   - 选项 C：不加密 EXIF，依赖操作系统级保护

2. **SQLCipher 的密钥管理策略？**
   - 选项 A：机器指纹派生（当前 API Key 方案，同机攻击者可复现）
   - 选项 B：用户首次启动时设置主密码（更安全，但增加使用门槛）
   - 选项 C：Windows DPAPI / macOS Keychain 存储（平台特定，最安全）

3. **是否接受 SQLCipher 对 CI 构建时间的影响？**
   - 选项 A：接受，CI 增量时间可接受
   - 选项 B：仅 release 构建启用 SQLCipher，dev 构建使用明文
   - 选项 C：暂不引入 SQLCipher，等待 rusqlite 升级到 0.37+ 后再评估

4. **合规要求的紧迫性？**
   - 选项 A：当前无外部合规压力，按路线图推进即可
   - 选项 B：已有企业用户/合规要求，需立即实施 SQLCipher
   - 选项 C：不确定，需要进一步调研

---

## 附录 A：rusqlite SQLCipher Feature 参考

rusqlite 提供 3 个 SQLCipher 相关 feature：

| Feature | 说明 | 适用场景 |
|---------|------|----------|
| `sqlcipher` | 链接系统安装的 SQLCipher 库 | Linux/macOS，需预装 libsqlcipher |
| `bundled-sqlcipher` | 编译捆绑的 SQLCipher + 链接系统 OpenSSL | Windows，需系统有 OpenSSL |
| `bundled-sqlcipher-vendored-openssl` | 编译捆绑的 SQLCipher + 捆绑的 OpenSSL | 全平台零依赖，推荐 Windows 使用 |

**当前项目使用 `bundled` feature**，切换到 `bundled-sqlcipher-vendored-openssl` 是最小改动路径。

[source: docs.rs/crate/rusqlite/latest/features, 置信度: 已确认]

## 附录 B：SQLCipher 性能优化参数

```sql
-- 使用 RAW key 跳过 PBKDF2 密钥派生（首次连接从 ~500ms 降至 <1ms）
PRAGMA key = "x'2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99'";

-- 降低 KDF 迭代次数（默认 256000，降至 64000 可提速 4 倍，安全性仍可接受）
PRAGMA kdf_iter = 64000;

-- 禁用 HMAC 页校验（提升读写性能约 10-15%，但丧失防篡改能力）
PRAGMA cipher_integrity_check = OFF;

-- 使用更快的加密算法
PRAGMA cipher = 'aes-256-gcm';  -- SQLCipher 4.5+ 支持
```

[source: zetetic.net/sqlcipher/performance, 置信度: 已确认]

## 附录 C：UNVERIFIED 标记

| 项目 | 原因 |
|------|------|
| SQLCipher 编译增量时间 +2-5 分钟 | 基于 GitHub issue 估算，未在 ArcaneCodex 项目实际编译验证 |
| SQLCipher 二进制增量 +1-2 MB | 基于 KeychainPGP 项目估算，ArcaneCodex 的 Tauri + ONNX Runtime 已有较大二进制，增量比例可能不同 |
| 实测性能数据（+13.6% 写入 / +10.8% 读取） | 来自第三方基准测试，非 ArcaneCodex 场景，实际影响取决于数据量和查询模式 |
| rusqlite 0.31 `bundled-sqlcipher-vendored-openssl` 在 Windows MSVC 工具链的编译成功率 | 未实际验证，可能需要额外的构建配置 |
