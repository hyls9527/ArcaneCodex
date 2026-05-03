# React Query 渐进式迁移任务

> 基于辩论结论：分层状态管理 + 渐进式 React Query 接入
> 核心原则：服务端状态 → React Query，UI 状态 → Zustand

---

## Phase 1: 基础设施（本周完成）

### Task 1.1: 配置 QueryClient
**文件**: `src/lib/query-client.ts` (新建)
**描述**: 创建全局 QueryClient 配置，包含合理的默认 staleTime、重试策略
**验收标准**:
- [ ] QueryClient 实例导出
- [ ] staleTime 默认 5 分钟
- [ ] 重试策略：网络错误重试 3 次，4xx 错误不重试
- [ ] gcTime 默认 10 分钟

### Task 1.2: 集成 QueryClientProvider
**文件**: `src/main.tsx`
**描述**: 在应用根节点包裹 QueryClientProvider
**验收标准**:
- [ ] App 组件被 QueryClientProvider 包裹
- [ ] 开发环境启用 React Query DevTools

### Task 1.3: 创建 Query Keys 管理规范
**文件**: `src/lib/query-keys.ts` (新建)
**描述**: 集中管理所有 query keys，避免硬编码
**验收标准**:
- [ ] 定义 images、search、ai、dedup、stats 等模块的 query key 工厂函数
- [ ] 示例: `images.list(page, filters)`, `images.detail(id)`

---

## Phase 2: API Hooks 封装（本周完成）

### Task 2.1: 创建 images Query Hooks
**文件**: `src/hooks/queries/useImages.ts` (新建)
**描述**: 封装 getImages API 为 useQuery hook
**验收标准**:
- [ ] `useImages(query: ImageQuery)` hook
- [ ] 支持分页和过滤
- [ ] 返回 data, isLoading, error, refetch

### Task 2.2: 创建 search Query Hooks
**文件**: `src/hooks/queries/useSearch.ts` (新建)
**描述**: 封装 searchImages API 为 useQuery hook
**验收标准**:
- [ ] `useSearch(query: string, filters: SearchFilters)` hook
- [ ] 支持 enabled 选项（query 为空时不执行）

### Task 2.3: 创建 stats Query Hooks
**文件**: `src/hooks/queries/useStats.ts` (新建)
**描述**: 封装 getLibraryStats 和 getAccuracyTrend API
**验收标准**:
- [ ] `useLibraryStats()` hook
- [ ] `useAccuracyTrend(days)` hook

### Task 2.4: 创建 mutation Hooks
**文件**: `src/hooks/mutations/useImageMutations.ts` (新建)
**描述**: 封装 deleteImages、importImages 等 mutation
**验收标准**:
- [ ] `useDeleteImages()` hook，成功后自动刷新 images query
- [ ] `useImportImages()` hook

---

## Phase 3: 组件迁移（按需进行）

### Task 3.1: 迁移 DashboardPage 统计数据
**文件**: `src/pages/DashboardPage.tsx`
**描述**: 将 stats 数据获取从 Zustand 迁移到 React Query
**验收标准**:
- [ ] 使用 `useLibraryStats()` 替代手动 fetch
- [ ] 使用 `useAccuracyTrend()` 替代手动 fetch
- [ ] 移除 DashboardPage 中相关的 useEffect 和 loading 状态

### Task 3.2: 迁移 ImageGrid 图片列表
**文件**: `src/components/gallery/ImageGrid.tsx`
**描述**: 将图片列表数据获取迁移到 React Query
**验收标准**:
- [ ] 使用 `useImages()` 替代 useImageStore.loadImages
- [ ] 保留 filters、page、pageSize 在 URL 或本地 state
- [ ] 支持缓存：返回列表页时不重新加载

### Task 3.3: 迁移 Search 功能
**文件**: `src/components/gallery/ImageFilter.tsx` 或相关搜索组件
**描述**: 将搜索功能迁移到 React Query
**验收标准**:
- [ ] 使用 `useSearch()` 替代 useImageStore 中的 search 逻辑
- [ ] 支持防抖（debounce）

---

## Phase 4: Store 清理（组件迁移完成后）

### Task 4.1: 清理 useImageStore
**文件**: `src/stores/useImageStore.ts`
**描述**: 移除已迁移到 React Query 的状态和方法
**验收标准**:
- [ ] 移除 `images`、`loading`、`error`、`loadImages`
- [ ] 保留 `selectedIds`、`filters`、`page`、`pageSize`（UI 状态）
- [ ] 更新 TypeScript 类型定义

### Task 4.2: 更新代码规范文档
**文件**: `docs/state-management.md` (新建)
**描述**: 记录状态管理规范，指导后续开发
**验收标准**:
- [ ] 明确服务端状态 vs UI 状态的定义
- [ ] 提供选择 React Query 或 Zustand 的决策流程图
- [ ] 示例代码展示正确用法

---

## 状态边界定义

| 状态类型 | 存储位置 | 示例 |
|---------|---------|------|
| 服务端状态 | React Query | images 列表、stats 数据、search 结果 |
| UI 状态 | Zustand | selectedIds、filters、page、pageSize、isSidebarOpen |
| 派生状态 | 计算属性/selector | 过滤后的 images、分页计算 |

---

## 禁止事项

- [ ] 禁止在 Zustand 中存储服务端数据（images、stats 等）
- [ ] 禁止在 React Query 中存储 UI 状态（selectedIds、modal 显隐等）
- [ ] 禁止新功能使用手动 fetch + Zustand 模式

---

## 参考文档

- [React Query 文档](https://tanstack.com/query/latest/docs/framework/react/overview)
- [Zustand 文档](https://docs.pmnd.rs/zustand)
