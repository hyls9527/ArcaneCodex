# Arcane Codex 性能优化指南

> 最后更新: 2026-05-03

## 数据库优化

### SQLite 配置

```sql
-- 启用 WAL 模式（已实现）
PRAGMA journal_mode = WAL;

-- 同步模式设置
PRAGMA synchronous = NORMAL;

-- 缓存大小（单位：页，每页 4KB）
PRAGMA cache_size = -64000; -- 256MB

-- 临时存储
PRAGMA temp_store = MEMORY;

-- 外键约束
PRAGMA foreign_keys = ON;

-- 忙碌超时
PRAGMA busy_timeout = 5000;
```

### 索引优化

已创建的索引：

```sql
-- 图片搜索
CREATE INDEX idx_images_created_at ON images(created_at DESC);
CREATE INDEX idx_images_file_path ON images(file_path);
CREATE INDEX idx_images_file_hash ON images(file_hash);

-- 标签搜索
CREATE INDEX idx_image_tags_image_id ON image_tags(image_id);
CREATE INDEX idx_image_tags_tag_id ON image_tags(tag_id);
CREATE INDEX idx_tags_name ON tags(name);

-- 语义搜索（jieba-rs 分词 + 倒排索引）
CREATE INDEX idx_search_index_term ON search_index(term);
CREATE INDEX idx_search_index_image_id ON search_index(image_id);
```

> **注意**: 项目使用 jieba-rs 中文分词 + 自建倒排索引实现语义搜索，未使用 SQLite FTS5。

---

## 缓存策略

### 搜索缓存

```rust
// 已实现：HashMap 缓存 + TTL 5 分钟（非 LRU）
const SEARCH_CACHE_TTL: Duration = Duration::from_secs(300);
static SEARCH_CACHE: OnceLock<Mutex<HashMap<u64, (Instant, Vec<SearchResult>)>>> = OnceLock::new();
```

### 缩略图缓存

- 生成缩略图：300x200px（当前实现）
- 按需加载，懒加载策略
- 缓存目录：`%APPDATA%/arcane-codex/thumbnails/`

---

## 前端优化

### 代码分割

```typescript
// vite.config.ts 实际配置
build: {
  rollupOptions: {
    output: {
      manualChunks: {
        'vendor-react': ['react', 'react-dom'],
        'vendor-state': ['zustand'],
        'vendor-ui': ['motion', 'clsx', 'lucide-react'],
        'vendor-i18n': ['i18next', 'react-i18next'],
        'vendor-virtual': ['@tanstack/react-virtual'],
        'vendor-dropzone': ['react-dropzone'],
      },
    },
  },
}
```

### 虚拟列表

大图库使用 `@tanstack/react-virtual` 实现虚拟滚动（当前实现）：

```tsx
import { useVirtualizer } from '@tanstack/react-virtual'

const virtualizer = useVirtualizer({
  count: images.length,
  getScrollElement: () => parentRef.current,
  estimateSize: () => cellSize,
})
```

---

## 并发处理

### AI 批量处理

```rust
// 使用 tokio 并发控制
let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
let mut tasks = Vec::new();

for image in images {
    let permit = semaphore.clone().acquire_owned().await?;
    let task = tokio::spawn(async move {
        let _permit = permit;
        process_image(&image).await
    });
    tasks.push(task);
}
```

### 数据库连接池

```rust
// 使用 r2d2 连接池（默认配置，未设置 max_size）
let manager = SqliteConnectionManager::file(&db_path)
    .with_init(|conn| {
        conn.execute_batch(PRAGMA_CONFIG)?;
        Ok(())
    });
let pool = Pool::new(manager)?;
```

---

## 内存优化

### 图片处理

- 流式处理大文件，避免一次性加载到内存
- 缩略图生成使用内存限制
- 及时释放图片资源

### 搜索索引

- 按需加载索引
- 定期清理过期缓存
- 限制索引大小

---

## 监控指标

### 性能指标

| 指标 | 目标值 | 说明 |
|------|--------|------|
| 图片导入 | < 500ms/张 | 含缩略图生成，目标值，未实测 |
| 搜索响应 | < 500ms | 10000 张图片内，目标值，未实测 |
| AI 打标 | 5-30s/张 | 取决于本地 AI 模型和硬件配置，经验估算 |
| 启动时间 | < 5s | 冷启动，目标值，未实测 |
| 内存占用 | < 500MB | 10000 张图片，目标值，未实测 |

> **注意**: 以上性能指标均为设计目标值，尚未进行系统化基准测试验证。

### 监控代码

```rust
use tracing::{info, instrument};

#[instrument(skip(db))]
pub async fn search_images(db: &Database, query: &str) -> AppResult<Vec<Image>> {
    let start = Instant::now();
    // ... 搜索逻辑
    info!("Search completed in {:?}", start.elapsed());
    Ok(results)
}
```

---

## 性能测试

> **注意**: 项目尚未实现系统化基准测试。以下为规划中的测试方案。

### 基准测试（规划中）

```bash
# 计划：添加 Rust 基准测试
# cargo bench

# 前端性能分析
npm run build -- --mode production
npx vite-bundle-visualizer
```

---

## 优化建议

### 短期

1. ✅ 启用 SQLite WAL 模式
2. ✅ 实现搜索缓存
3. ✅ 前端代码分割
4. ⏳ 添加数据库索引监控

### 中期

1. ⏳ 实现增量索引更新
2. ⏳ 添加性能监控面板
3. ⏳ 优化大图库加载速度

### 长期

1. ⏳ 支持分布式处理
2. ⏳ GPU 加速图片处理
3. ⏳ 智能预加载
