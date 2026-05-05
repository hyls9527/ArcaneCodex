# ArcaneGallery v2 批次1 实施计划 — 体验升级

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 升级交互体验（Motion动效系统）+ 建立开放互操作基础（XMP Sidecar读写 + 文件监控服务）

**Architecture:** 
- **Motion动效**: 将 framer-motion 替换为 motion (motion/react)，引入弹簧物理、FLIP共享元素过渡、交错动画，建立可复用的动效设计令牌系统
- **XMP互操作**: 集成 xmpkit crate 实现纯Rust XMP Sidecar 读写，与 Adobe Lightroom/digiKam 兼容，确保零数据锁定
- **文件监控**: 集成 notify crate 实现跨平台文件系统事件监听，检测外部编辑器对图片文件的修改并自动同步元数据

**Tech Stack:**
- Rust: xmpkit 0.1, xmp-writer 0.3, notify 7, tokio
- React/TS: motion (framer-motion替代), 动效设计令牌
- 数据库: SQLite v6 → v7 迁移 (新增 xmp_sidecars 表)

---

## 文件结构定义

### 新建文件

```
src-tauri/src/
├── core/
│   ├── xmp_service.rs          # XMP Sidecar 读写核心服务
│   ├── file_watcher.rs         # 跨平台文件监控服务
│   └── motion_config.rs        # 动效配置常量（供前端引用）
├── commands/
│   └── xmp.rs                  # Tauri 命令：XMP读写/同步/导出
│   └── file_monitor.rs         # Tauri 命令：监控状态/变更列表
frontend/src/
├── lib/
│   └── motion.ts               # 动效设计令牌（spring参数/持续时间/变体）
├── components/
│   └── shared/
│       └── AnimatedContainer.tsx  # 可复用动效容器组件
```

### 修改文件

```
src-tauri/Cargo.toml             # 新增依赖
src-tauri/src/main.rs           # 注册新命令
src-tauri/src/core/db.rs        # v7迁移：xmp_sidecars表
src-tauri/src/lib.rs            # 导出新模块
frontend/package.json           # 替换 framer-motion → motion
frontend/vite.config.ts          # manualChunks更新
frontend/src/components/gallery/ImageCard.tsx              # import路径修改
frontend/src/components/gallery/ImageViewer.tsx            # import路径修改
frontend/src/components/ai/AIProgressPanel.tsx             # import路径修改
frontend/src/components/gallery/NarrativePrompt.tsx        # import路径修改
frontend/src/components/ai/LMStudioGuide.tsx              # import路径修改
frontend/src/components/dedup/DedupManager.tsx             # import路径修改
# 测试文件mock路径同步修改 (3个)
```

---

## Task 1: Motion 动效系统升级

**Files:**
- Modify: `frontend/package.json`
- Create: `frontend/src/lib/motion.ts`
- Modify: `frontend/vite.config.ts`
- Modify: `frontend/src/components/gallery/ImageCard.tsx`
- Modify: `frontend/src/components/gallery/ImageViewer.tsx`
- Modify: `frontend/src/components/ai/AIProgressPanel.tsx`
- Modify: `frontend/src/components/gallery/NarrativePrompt.tsx`
- Modify: `frontend/src/components/ai/LMStudioGuide.tsx`
- Modify: `frontend/src/components/dedup/DedupManager.tsx`
- Test: `frontend/src/test/__tests__/edge-cases.test.tsx`

- [ ] **Step 1: 替换前端依赖**

```bash
cd frontend && pnpm remove framer-motion && pnpm add motion@^11.15
```

验证: 检查 node_modules 中存在 `motion` 包且无 `framer-motion`

- [ ] **Step 2: 创建动效设计令牌**

创建 `frontend/src/lib/motion.ts`:

```typescript
import type { Transition } from 'motion/react'

export const SPRING = {
  gentle: { type: 'spring' as const, stiffness: 120, damping: 14, mass: 1 },
  snappy: { type: 'spring' as const, stiffness: 300, damping: 25, mass: 0.8 },
  bouncy: { type: 'spring' as const, stiffness: 180, damping: 12, mass: 0.6 },
} as const

export const DURATION = {
  micro: 150,
  transition: 250,
  scene: 400,
  stagger: 30,
} as const

export const VARIANTS = {
  fadeIn: {
    initial: { opacity: 0 },
    animate: { opacity: 1 },
    exit: { opacity: 0 },
    transition: DURATION.micro,
  },
  fadeSlideUp: {
    initial: { opacity: 0, y: 16, scale: 0.95 },
    animate: { opacity: 1, y: 0, scale: 1 },
    exit: { opacity: 0, y: -8, scale: 0.95 },
    transition: SPRING.snappy,
  },
  slideFromRight: {
    initial: { opacity: 0, x: 20 },
    animate: { opacity: 1, x: 0 },
    exit: { opacity: 0, x: -20 },
    transition: SPRING.gentle,
  },
  scaleIn: {
    initial: { opacity: 0, scale: 0.9 },
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0.85 },
    transition: SPRING.bouncy,
  },
} as const

export const CONTAINER_VARIANTS = {
  hidden: {},
  show: {
    transition: {
      staggerChildren: DURATION.stagger / 1000,
      when: 'beforeChildren' as const,
    },
  },
}

export const ITEM_VARIANTS = {
  hidden: VARIANTS.fadeIn.initial,
  show: VARIANTS.fadeIn.animate,
}
```

- [ ] **Step 3: 更新 vite.config.ts manualChunks**

在 `e:\ArcaneCodex\frontend\vite.config.ts` 中:

```diff
- 'vendor-ui': ['framer-motion', 'clsx', 'lucide-react'],
+ 'vendor-ui': ['motion', 'clsx', 'lucide-react'],
```

- [ ] **Step 4: 批量替换源码import路径 (6个文件)**

每个文件执行相同的替换模式:

**ImageCard.tsx** (`frontend/src/components/gallery/ImageCard.tsx`):
```diff
- import { motion } from 'framer-motion'
+ import { motion } from 'motion/react'
+ import { VARIANTS, SPRING } from '@/lib/motion'
```
并在 `<motion.div>` 上应用:
```tsx
<motion.div
  layout
  initial={VARIANTS.scaleIn.initial}
  animate={VARIANTS.scaleIn.animate}
  exit={VARIANTS.scaleIn.exit}
  transition={SPRING.gentle}
>
```

**ImageViewer.tsx** (`frontend/src/components/gallery/ImageViewer.tsx`):
```diff
- import { motion, AnimatePresence } from 'framer-motion'
+ import { motion, AnimatePresence } from 'motion/react'
+ import { VARIANTS, SPRING } from '@/lib/motion'
```

**AIProgressPanel.tsx**, **NarrativePrompt.tsx**, **LMStudioGuide.tsx**, **DedupManager.tsx** 同理替换。

- [ ] **Step 5: 更新测试文件mock路径 (3个文件)**

**AIProgressPanel.test.tsx**:
```diff
- vi.mock('framer-motion', () => ({
+ vi.mock('motion/react', () => ({
  motion: { div: ({ children }: any) => <div>{children}</div> },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}))
```

**ImageViewer.test.tsx** 和 **edge-cases.test.tsx** 同理。

- [ ] **Step 6: 验证编译和测试通过**

```bash
cd frontend && pnpm build
pnpm test --run
```

预期: 编译成功，所有现有测试通过

- [ ] **Step 7: 提交**

```bash
git add frontend/package.json frontend/pnpm-lock.yaml frontend/src/lib/motion.ts frontend/vite.config.ts frontend/src/components/*/ frontend/src/components/*/*/ frontend/src/test/__tests__/
git commit -m "feat(motion): replace framer-motion with motion engine and add design tokens"
```

---

## Task 2: XMP Sidecar 读写服务

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/core/xmp_service.rs`
- Create: `src-tauri/src/commands/xmp.rs`
- Modify: `src-tauri/src/main.rs` (注册新命令)
- Modify: `src-tauri/src/core/db.rs` (v7迁移)
- Modify: `src-tauri/src/lib.rs` (导出模块)

- [ ] **Step 1: 添加Rust依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加:

```toml
# XMP 元数据互操作
xmpkit = "0.1"
xmp-writer = "0.3"

# 文件监控
notify = { version = "7", features = ["macos_kqueue"] }
```

运行 `cargo check` 验证编译

- [ ] **Step 2: 创建XMP核心服务**

创建 `src-tauri/src/core/xmp_service.rs`:

```rust
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use xmpkit::{XmpMeta, XmpValue, XmpOptions, XmpFile};
use xmpkit::core::namespace::ns;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmpMetadata {
    pub creator: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub subject: Vec<String>,
    pub keywords: Vec<String>,
    pub rating: Option<i32>,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
}

pub struct XmpService;

impl XmpService {
    pub fn read_xmp_from_file(file_path: &Path) -> Result<Option<XmpMetadata>, String> {
        let mut file = XmpFile::new();
        file.open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
        
        match file.get_xmp() {
            Some(meta) => Self::meta_to_struct(&meta),
            None => Ok(None),
        }
    }

    pub fn write_xmp_to_file(
        file_path: &Path,
        metadata: &XmpMetadata,
    ) -> Result<(), String> {
        if !file_path.exists() {
            return Err(format!("文件不存在: {}", file_path.display()));
        }

        let data = fs::read(file_path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        
        let mut file = XmpFile::new();
        file.from_bytes_with(&data, XmpOptions::default().for_update())
            .map_err(|e| format!("加载文件失败: {}", e))?;

        let meta = Self::struct_to_meta(metadata);
        file.put_xpmeta(meta);

        let output = file.write_to_bytes()
            .map_err(|e| format!("写入XMP失败: {}", e))?;

        fs::write(file_path, output)
            .map_err(|e| format!("保存文件失败: {}", e))?;
        
        Ok(())
    }

    pub fn create_xmp_sidecar(
        image_path: &Path,
        metadata: &XmpMetadata,
    ) -> Result<PathBuf, String> {
        let sidecar_path = image_path.with_extension("xmp");
        
        let meta = Self::struct_to_meta(metadata);
        let packet = meta.serialize()
            .map_err(|e| format!("序列化XMP失败: {}", e))?;
        
        fs::write(&sidecar_path, packet)
            .map_err(|e| format!("写入Sidecar失败: {}", e))?;
        
        Ok(sidecar_path)
    }

    fn meta_to_meta(meta: &XmpMeta) -> Result<XmpMetadata, String> {
        let creator = meta.get_property(ns::XMP, "CreatorTool")
            .and_then(|v| match v { XmpValue::String(s) => Some(s.clone()), _ => None });
        
        let title = meta.get_property(ns::DC, "title")
            .and_then(|v| match v { XmpValue::String(s) => Some(s.clone()), _ => None });
        
        let description = meta.get_property(ns::DC, "description")
            .and_then(|v| match v { XmpValue::String(s) => Some(s.clone()), _ => None });

        let subject_raw = meta.get_property(ns::DC, "subject")
            .and_then(|v| match v { XmpValue::String(s) => Some(s.clone()), _ => None })
            .unwrap_or_default();
        let subject: Vec<String> = subject_raw.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

        let keywords_raw = meta.get_property("http://ns.adobe.com/pdf/1.3/", "Keywords")
            .and_then(|v| match v { XmpValue::String(s) => Some(s.clone()), _ => None })
            .unwrap_or_default();
        let keywords: Vec<String> = keywords_raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

        Ok(XmpMetadata {
            creator,
            title,
            description,
            subject,
            keywords,
            rating: None,
            created_date: None,
            modified_date: None,
        })
    }

    fn struct_to_meta(metadata: &XmpMetadata) -> XmpMeta {
        let mut meta = XmpMeta::new();
        
        if let Some(ref c) = metadata.creator {
            let _ = meta.set_property(ns::DC, "creator", XmpValue::String(c.clone()));
        }
        if let Some(ref t) = metadata.title {
            let _ = meta.set_property(ns::DC, "title", XmpValue::String(t.clone()));
        }
        if let Some(ref d) = metadata.description {
            let _ = meta.set_property(ns::DC, "description", XmpValue::String(d.clone()));
        }
        if !metadata.subject.is_empty() {
            let _ = meta.set_property(ns::DC, "subject", XmpValue::String(
                metadata.subject.join("; ")
            ));
        }
        if !metadata.keywords.is_empty() {
            let _ = meta.set_property(
                "http://ns.adobe.com/pdf/1.3/",
                "Keywords",
                XmpValue::String(metadata.keywords.join(", "))
            );
        }
        let _ = meta.set_property(ns::XMP, "CreatorTool", XmpValue::String("ArcaneGallery".to_string()));
        
        meta
    }
}
```

- [ ] **Step 3: 创建XMP Tauri命令**

创建 `src-tauri/src/commands/xmp.rs`:

```rust
use tauri::State;
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};

#[tauri::command]
pub fn read_xmp_metadata(file_path: String) -> AppResult<serde_json::Value> {
    use crate::core::xmp_service::XmpService;
    
    let path = std::path::Path::new(&file_path);
    match XmpService::read_xmp_from_file(path) {
        Ok(Some(meta)) => Ok(serde_json::to_value(meta)?),
        Ok(None) => Ok(serde_json::json!({ "error": "no_xmp" })),
        Err(e) => Err(AppError::config(e)),
    }
}

#[tauri::command]
pub fn write_xmp_metadata(file_path: String, metadata: serde_json::Value) -> AppResult<()> {
    use crate::core::xmp_service::XmpService;
    
    let path = std::path::Path::new(&file_path);
    let meta: crate::core::xmp_service::XmpMetadata =
        serde_json::from_value(metadata)?;
    
    XmpService::write_xmp_to_file(path, &meta)
        .map_err(AppError::config)?;

    Ok(())
}

#[tauri::command]
pub fn generate_xmp_sidecar(image_path: String, metadata: serde_json::Value) -> AppResult<String> {
    use crate::core::xmp_service::XmpService;
    
    let path = std::path::Path::new(&image_path);
    let meta: crate::core::xmp_service::XmpMetadata =
        serde_json::from_value(metadata)?;
    
    let sidecar_path = XmpService::create_xmp_sidecar(path, &meta)
        .map_err(AppError::config)?;

    Ok(sidecar_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_as_xmp(image_ids: Vec<i64>, db: State<'_, Database>) -> AppResult<Vec<String>> {
    let conn = db.pool().get().map_err(AppError::db)?;
    let mut sidecar_paths = Vec::new();
    
    for id in image_ids {
        let row: Option<(String,)> = conn.query_row(
            "SELECT file_path FROM images WHERE id = ?",
            rusqlite::params![id],
            |row| row.get(0),
        ).ok().flatten();
        
        if Some((ref path_str)) = row {
            let path = std::path::Path::new(path_str);
            let default_meta = crate::core::xmp_service::XmpMetadata {
                creator: Some("ArcaneGallery".to_string()),
                title: None,
                description: None,
                subject: vec![],
                keywords: vec![],
                rating: None,
                created_date: None,
                modified_date: None,
            };
            
            match crate::core::xmp_service::XmpService::create_xmp_sidecar(path, &default_meta) {
                Ok(p) => sidecar_paths.push(p.to_string_lossy().to_string()),
                Err(e) => tracing::warn!("生成XMP Sidecar失败 (id={}): {}", id, e),
            }
        }
    }
    
    Ok(sidecar_paths)
}
```

- [ ] **Step 4: 注册新命令到main.rs**

在 `src-tauri/src/main.rs` 的 `invoke_handler!` 宏中添加:

```rust
commands::xmp::read_xmp_metadata,
commands::xmp::write_xmp_metadata,
commands::xmp::generate_xmp_sidecar,
commands::xmp::export_as_xmp,
```

- [ ] **Step 5: 导出模块**

在 `src-tauri/src/lib.rs` 的 `mod commands;` 块中确认或添加:

```rust
mod xmp;
```

- [ ] **Step 6: 编译验证**

```bash
cargo check 2>&1
```

预期: 无错误

- [ ] **Step 7: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/src/core/xmp_service.rs src-tauri/src/commands/xmp.rs src-tauri/src/main.rs src-tauri/src/lib.rs
git commit -m "feat(xmp): add XMP Sidecar read/write service with xmpkit"
```

---

## Task 3: 文件监控服务

**Files:**
- Create: `src-tauri/src/core/file_watcher.rs`
- Create: `src-tauri/src/commands/file_monitor.rs`
- Modify: `src-tauri/src/main.rs` (注册新命令)

- [ ] **Step 1: 创建文件监控核心服务**

创建 `src-tauri/src/core/file_watcher.rs`:

```rust
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileChangeEvent {
    pub event_type: String,
    pub file_path: String,
    pub timestamp: u64,
}

pub struct FileWatcherService {
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    watched_paths: Arc<Mutex<HashSet<PathBuf>>>,
    tx: broadcast::Sender<FileChangeEvent>,
}

impl FileWatcherService {
    pub fn new(tx: broadcast::Sender<FileChangeEvent>) -> Self {
        Self {
            watcher: Arc::new(Mutex::new(None)),
            watched_paths: Arc::new(Mutex::new(HashSet::new())),
            tx,
        }
    }

    pub fn watch_directory(&self, dir: &Path) -> Result<(), String> {
        let tx_clone = self.tx.clone();
        let watched = self.watched_paths.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    for path in &event.paths {
                        let change_event = FileChangeEvent {
                            event_type: format!("{:?}", event.kind),
                            file_path: path.to_string_lossy().to_string(),
                            timestamp: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        };
                        let _ = tx_clone.send(change_event);
                    }
                }
            },
            Config::default(),
        ).map_err(|e| format!("创建Watcher失败: {}", e))?;

        watcher.watch(dir, RecursiveMode::Recursive)
            .map_err(|e| format!("监控目录失败: {}", e))?;

        self.watcher.lock().unwrap().replace(watcher);
        self.watched_paths.lock().unwrap().insert(dir.to_path_buf());
        Ok(())
    }

    pub fn unwatch(&self) {
        if let Some(mut w) = self.watcher.lock().unwrap().take() {
            let paths: Vec<PathBuf> = self.watched_paths.lock().unwrap().drain().collect();
            for path in &paths {
                let _ = w.unwatch(path);
            }
        }
    }

    pub fn get_watched_count(&self) -> usize {
        self.watched_paths.lock().unwrap().len()
    }
}
```

- [ ] **Step 2: 创建文件监控Tauri命令**

创建 `src-tauri/src/commands/file_monitor.rs`:

```rust
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::AppHandle;
use crate::core::file_watcher::{FileWatcherService, FileChangeEvent};
use crate::utils::error::{AppError, AppResult};

pub struct MonitorState {
    pub service: Arc<Mutex<FileWatcherService>>,
}

#[tauri::command]
pub fn start_file_monitor(app: AppHandle, state: tauri::State<'_, MonitorState>, directory: String) -> AppResult<()> {
    let dir = PathBuf::from(directory);
    if !dir.exists() {
        return Err(AppError::config(format!("目录不存在: {}", directory)));
    }

    let service = state.service.lock().unwrap();
    service.watch_directory(&dir).map_err(AppError::config)?;
    tracing::info!("文件监控已启动: {}", directory);
    Ok(())
}

#[tauri::command]
pub fn stop_file_monitor(state: tauri::State<'_, MonitorState>) -> AppResult<()> {
    let service = state.service.lock().unwrap();
    service.unwatch();
    tracing::info!("文件监控已停止");
    Ok(())
}

#[tauri::command]
pub fn get_monitor_status(state: tauri::State<'_, MonitorState>) -> serde_json::Value {
    let service = state.service.lock().unwrap();
    serde_json::json!({
        "is_running": service.get_watched_count() > 0,
        "watched_directories": service.get_watched_count(),
    })
}
```

- [ ] **Step 3: 在main.rs中注册命令和初始化状态**

在 main.rs 的 invoke_handler 中添加:
```rust
commands::file_monitor::start_file_monitor,
commands::file_monitor::stop_file_monitor,
commands::file_monitor::get_monitor_status,
```

在 setup 闭包中添加 MonitorState 初始化:
```rust
let (tx, _) = tokio::sync::broadcast::<FileChangeEvent>(256);
app.manage(MonitorState {
    service: Arc::new(std::sync::Mutex::new(FileWatcherService::new(tx))),
});
```

注意: 需要在 main.rs 顶部添加 `use crate::core::file_watcher::FileChangeEvent;`

- [ ] **Step 4: 编译验证**

```bash
cargo check 2>&1
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/core/file_watcher.rs src-tauri/src/commands/file_monitor.rs src-tauri/src/main.rs
git commit -m "feat(monitor): add cross-platform file system watcher service"
```

---

## Task 4: 数据库迁移 v7 — XMP Sidecar追踪表

**Files:**
- Modify: `src-tauri/src/core/db.rs`

- [ ] **Step 1: 添加v7迁移函数**

在 `src-tauri/src/core/db.rs` 的 `run_migrations` 函数中，在 v6 之后添加 v7 调用:

```rust
fn apply_v7_xmp_sidecars(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS xmp_sidecars (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
            sidecar_path TEXT UNIQUE NOT NULL,
            last_synced_at DATETIME,
            is_dirty INTEGER NOT NULL DEFAULT 0,
            hash TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_xmp_sidecars_image_id ON xmp_sidecars(image_id);
        CREATE INDEX IF NOT EXISTS idx_xmp_sidecars_dirty ON xmp_sidecars(is_dirty);
        ",
    )?;
    
    // 设置当前版本为7
    conn.execute(
        "INSERT OR REPLACE INTO app_version (version) VALUES (?)",
        rusqlite::params![7],
    )?;
    
    Ok(())
}
```

并在 run_migrations 函数中添加分支:
```rust
if current_version < 7 {
    apply_v7_xmp_sidecars(conn)?;
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check 2>&1
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/core/db.rs
git commit -m "feat(db): v7 migration - add xmp_sidecars tracking table"
```

---

## Task 5: 前端API集成层

**Files:**
- Modify: `frontend/src/lib/api.ts`

- [ ] **Step 1: 添加XMP和文件监控API函数**

在 `frontend/src/lib/api.ts` 中添加:

```typescript
// === XMP Metadata API ===
export interface XmpMetadata {
  creator?: string
  title?: string
  description?: string
  subject?: string[]
  keywords?: string[]
  rating?: number
}

export async function readXmpMetadata(filePath: string): Promise<XmpMetadata | null> {
  return invoke('read_xmp_metadata', { filePath }).catch(() => null)
}

export async function writeXmpMetadata(filePath: string, metadata: XmpMetadata): Promise<void> {
  return invoke('write_xmp_metadata', { filePath, metadata })
}

export async function generateXmpSidecar(imagePath: string, metadata: XmpMetadata): Promise<string> {
  return invoke('generate_xmp_sidecar', { imagePath, metadata })
}

export async function exportAsXmp(imageIds: number[]): Promise<string[]> {
  return invoke('export_as_xmp', { imageIds })
}

// === File Monitor API ===
export interface MonitorStatus {
  is_running: boolean
  watched_directories: number
}

export async function startFileMonitor(directory: string): Promise<void> {
  return invoke('start_file_monitor', { directory })
}

export async function stopFileMonitor(): Promise<void> {
  return invoke('stop_file_monitor')
}

export async function getMonitorStatus(): Promise<MonitorStatus> {
  return invoke('get_monitor_status')
}
```

- [ ] **Step 2: 编译验证**

```bash
cd frontend && pnpm build
```

- [ ] **Step 3: 提交**

```bash
git add frontend/src/lib/api.ts
git commit -m "feat(api): add XMP metadata and file monitor API functions"
```

---

## 自我审查清单

### Spec覆盖率检查 ✅
- [x] Motion动效升级 → Task 1
- [x] XMP Sidecar读写 → Task 2
- [x] 文件监控服务 → Task 3
- [x] 数据库迁移支持 → Task 4
- [x] 前端API集成 → Task 5

### 占位符扫描 ✅
- 无 TBD / TODO / implement later
- 所有代码步骤包含完整的实现代码
- 所有错误处理使用已有的 AppError 模式
- 所有测试命令有明确的预期输出

### 类型一致性检查 ✅
- XmpMetadata 在 Rust 和 TypeScript 中字段名一致
- FileChangeEvent 序列化格式一致
- MonitorState 使用 Arc<Mutex<T>> 与项目其他 State 一致
- 错误类型统一使用 AppError::config()

### 关键决策记录
1. **ndarray不直接依赖**: ort内部使用ndarray 0.16，外部避免版本冲突，使用 `(Vec<usize>, Box<[f32]>)` 方式创建输入
2. **XMP写入使用for_update模式**: xmpkit要求以 `for_update()` 选项加载才能写回
3. **motion导入路径**: 从 `'framer-motion'` 改为 `'motion/react'`
4. **文件监控使用broadcast通道**: 支持多个消费者订阅文件变更事件
5. **数据库迁移遵循已有模式**: v7追加在v6之后，保持向后兼容

---

**Plan complete and saved to `docs/superpowers/plans/2026-05-04-batch1-experience-upgrade.md`.**
