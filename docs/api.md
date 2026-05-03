# Arcane Codex API 文档

> 最后更新: 2026-05-03

## 概述

Arcane Codex 使用 Tauri 架构，前端通过 `@tauri-apps/api` 调用后端 Rust 命令。

---

## 核心 API

### 图片管理 (`images`)

#### `import_images`
导入图片到库中。

```typescript
import { invoke } from '@tauri-apps/api/core'

interface ImportResult {
  imported: number
  skipped: number
  errors: string[]
}

const result = await invoke<ImportResult>('import_images', {
  paths: ['/path/to/image1.jpg', '/path/to/image2.png']
})
```

#### `get_images`
获取图片列表。

```typescript
interface ImageInfo {
  image_id: number
  file_name: string
  file_path: string
  width: number
  height: number
  file_size: number
  thumbnail_path: string | null
  created_at: string
  updated_at: string
}

interface PaginatedResult<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}

const result = await invoke<PaginatedResult<ImageInfo>>('get_images', {
  page: 1,
  pageSize: 50,
  sortBy: 'created_at',
  sortOrder: 'desc'
})
```

#### `search_images`
语义搜索图片。

```typescript
interface SearchResult {
  images: ImageInfo[]
  query: string
  total: number
}

const result = await invoke<SearchResult>('search_images', {
  query: 'sunset beach',
  limit: 20
})
```

#### `delete_images`
删除图片。

```typescript
await invoke('delete_images', {
  imageIds: [1, 2, 3]
})
```

---

### 标签管理 (`tags`)

#### `get_tags`
获取所有标签。

```typescript
interface Tag {
  tag_id: number
  name: string
  category: string
  count: number
}

const tags = await invoke<Tag[]>('get_tags')
```

#### `add_tag_to_image`
为图片添加标签。

```typescript
await invoke('add_tag_to_image', {
  imageId: 1,
  tagName: 'landscape'
})
```

#### `remove_tag_from_image`
移除图片标签。

```typescript
await invoke('remove_tag_from_image', {
  imageId: 1,
  tagId: 5
})
```

---

### AI 推理 (`inference`)

#### `tag_image`
使用 AI 为图片打标签。

```typescript
interface TaggingResult {
  tags: Array<{
    name: string
    confidence: number
    category: string
  }>
  model: string
  processing_time_ms: number
}

const result = await invoke<TaggingResult>('tag_image', {
  imageId: 1,
  provider: 'openai', // 'openai' | 'anthropic' | 'local'
  model: 'gpt-4o'
})
```

#### `get_available_models`
获取可用的 AI 模型列表。

```typescript
interface ModelInfo {
  id: string
  name: string
  provider: string
  capabilities: string[]
}

const models = await invoke<ModelInfo[]>('get_available_models')
```

---

### 设置管理 (`settings`)

#### `get_settings`
获取应用设置。

```typescript
interface AppSettings {
  theme: 'light' | 'dark' | 'system'
  language: 'zh' | 'en'
  thumbnail_size: number
  ai_provider: string
  api_key_encrypted: string | null
}

const settings = await invoke<AppSettings>('get_settings')
```

#### `update_settings`
更新设置。

```typescript
await invoke('update_settings', {
  settings: {
    theme: 'dark',
    language: 'zh'
  }
})
```

#### `backup_database`
备份数据库。

```typescript
const backupPath = await invoke<string>('backup_database', {
  outputPath: '/path/to/backup.db'
})
```

#### `restore_database`
恢复数据库。

```typescript
await invoke('restore_database', {
  backupPath: '/path/to/backup.db'
})
```

---

### 去重功能 (`dedup`)

#### `find_duplicates`
查找重复图片。

```typescript
interface DuplicateGroup {
  id: string
  images: ImageInfo[]
  image_ids: number[]
  similarity: number
}

const groups = await invoke<DuplicateGroup[]>('find_duplicates', {
  threshold: 0.9 // 相似度阈值 0-1
})
```

#### `delete_duplicates`
删除重复图片。

```typescript
await invoke('delete_duplicates', {
  groupIds: ['group-1', 'group-2'],
  keepOriginal: true
})
```

---

### 存储管理 (`storage`)

#### `get_storage_stats`
获取存储统计。

```typescript
interface StorageStats {
  total_images: number
  total_size: number
  thumbnails_size: number
  database_size: number
  available_space: number
}

const stats = await invoke<StorageStats>('get_storage_stats')
```

#### `cleanup_thumbnails`
清理缩略图缓存。

```typescript
const cleaned = await invoke<number>('cleanup_thumbnails')
```

---

## 错误处理

所有 API 调用可能抛出以下错误：

```typescript
try {
  await invoke('some_command')
} catch (error) {
  // error 结构:
  // {
  //   code: 'DATABASE_ERROR' | 'IO_ERROR' | 'INVALID_INPUT' | 'NOT_FOUND',
  //   message: '详细错误信息',
  //   details: { ... }
  // }
}
```

---

## 事件监听

### 数据库变更事件

```typescript
import { listen } from '@tauri-apps/api/event'

await listen('database-changed', (event) => {
  console.log('Database changed:', event.payload)
})
```

### AI 处理进度

```typescript
await listen('ai-progress', (event) => {
  const { current, total, status } = event.payload
  console.log(`Processing ${current}/${total}: ${status}`)
})
```

---

## 类型定义

完整类型定义请参考 `frontend/src/lib/api.ts`。
