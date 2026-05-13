# Phase 1 前端组件拆分蓝图

> **生成日期**: 2026-05-12 | **状态**: 待执行
> **基于**: frontend-architect agent 全量审计结果

---

## 一、拆分优先级排序

| 优先级 | 组件 | 当前行数 | 拆分后 | 风险等级 | 核心产出 |
|--------|------|---------|--------|---------|---------|
| **P0** | ImageViewer | 461 行 | 5 模块 (~70 行/模块) | 🟢 低 | `useImageZoom` (通用 hook) |
| **P1** | KnowledgeGraphView | 533 行 | 6 模块 (~90 行/模块) | 🟡 中 | `useForceSimulation` |
| **P2** | GalleryPage | 441 行 | 4 模块 (~110 行/模块) | 🟢 低 | `useGalleryActions` |

---

## 二、ImageViewer 拆分方案 (P0 — 推荐首选)

### 当前问题
- 缩放/拖拽/键盘交互逻辑 (~60 行) 与 UI 渲染耦合
- Info Panel (~120 行) 和 Bottom Bar (~40 行) 数据重复展示
- 工具函数 (formatFileSize, parseExifData) 散落在组件内部

### 目标结构
```
components/gallery/ImageViewer/
├── index.tsx              # 组合层 (~70行): 布局编排
├── useImageZoom.ts        # 自定义 Hook (~60行): 缩放/拖拽/键盘
├── ImageInfoPanel.tsx     # 子组件 (~130行): 右侧面板
├── ImageBottomBar.tsx     # 子组件 (~60行): 底部信息栏
├── ImageToolbar.tsx       # 子组件 (~55行): 工具栏
├── SampleViewerPlaceholder.tsx  # 占位符 (~52行)
└── utils.ts               # formatFileSize, parseExifData
```

### useImageZoom Hook 签名（通用复用）
```typescript
interface UseImageZoomReturn {
  scale: number
  position: { x: number; y: number }
  isDragging: boolean
  handlers: {
    onWheel: (e: React.WheelEvent) => void
    onMouseDown: (e: React.MouseEvent) => void
    onMouseMove: (e: React.MouseEvent) => void
    onMouseUp: () => void
  }
  zoomIn: (step?: number) => void
  zoomOut: (step?: number) => void
  reset: () => void
}
```

### 风险点与缓解
| 风险 | 严重度 | 缓解措施 |
|------|--------|---------|
| 键盘 listener 生命周期 | 中 | 移入 hook，回调 ref 解决闭包 |
| scale/position 跨组件同步 | 中 | hook 返回 single source of truth |

---

## 三、KnowledgeGraphView 拆分方案 (P1)

### 当前问题
- 力导向物理引擎 (~200 行算法) 与 Canvas 渲染混合
- 完全自包含孤岛（零 store、零 props、14 个 hooks）
- 类型定义和常量内联在组件中

### 目标结构
```
components/gallery/KnowledgeGraphView/
├── index.tsx              # 组合层 (~80行)
├── useForceSimulation.ts  # 自定义 Hook (~180行): 物理引擎核心
├── GraphCanvas.tsx        # 子组件 (~120行): Canvas 渲染
├── GraphSidebar.tsx       # 子组件 (~100行): 侧边栏
├── GraphHeader.tsx        # 子组件 (~40行): 顶栏
├── types.ts               # ForceNode, ForceEdge 接口
└── constants.ts           # NODE_COLORS, EDGE_COLORS
```

### 关键发现：完全自包含
- **零 Zustand store 依赖**
- **零 props**（无父组件传参）
- **7 个 API 调用点**全部在 loadGraphData 内
- → 拆分后可考虑提升为全局状态或保持自包含

### 高风险点
| 风险 | 严重度 | 说明 |
|------|--------|------|
| forceNodesRef/edgesRef 同步 | **高** | 可变 ref 在模拟和绘制间共享，拆分后生命周期必须绑定 hook |
| requestAnimationFrame 清理 | 中 | unmount 时确保旧 rAF 取消 |
| draw() 闭包陈旧 | 低 | 状态变化时重新绑定 |

---

## 四、GalleryPage 拆分方案 (P2)

### 当前问题
- Page Controller 职责过重（协调 store + API + props + 8 个事件处理）
- `handleFilesSelected` 是最复杂函数（~40 行，4 步串行业务逻辑）
- GalleryEmptyState (~90 行) 内联在 JSX 中

### 目标结构
```
pages/GalleryPage/
├── index.tsx              # Page Controller (~120行)
├── GalleryActionBar.tsx   # 子组件 (~80行): 选择模式工具栏
├── GalleryEmptyState.tsx  # 子组件 (~90行): 空状态引导
├── GallerySearchResults.tsx # 子组件 (~50行): 搜索三态
└── useGalleryActions.ts   # 自定义 Hook (~100行): 业务操作封装
```

### useGalleryActions 封装范围
```typescript
interface UseGalleryActionsReturn {
  handleFilesSelected: (paths: string[]) => Promise<void>
  handleBatchDelete: () => Promise<void>
  handleClearFailedImages: () => Promise<void>
  handleSelectFiles: () => Promise<void>
  handleSelectFolder: () => Promise<void>
  deleting: boolean
  clearingFailed: boolean
}
```

---

## 五、CSP unsafe-inline 分析结论

> **判定：不建议当前阶段移除**

### 阻塞因素
1. **motion (framer-motion)** — 46 处动画强制依赖 inline style（库的设计基础）
2. **虚拟滚动** — ImageGrid 绝对定位布局无法用 CSS class 表达
3. **数据可视化** — 图表动态尺寸无法预编译为 Tailwind class

### 统计数据
| 类别 | 数量 |
|------|------|
| 显式内联 style | 12 处（10 处动态计算值 + 1 处可替代） |
| motion 动画注入 | 46 处（6 个组件） |
| `<style>` 标签 | 0 处 |
| dangerouslySetInnerHTML | 0 处 |

### 低成本改进（P1 可做）
- [AIConfig.tsx:313-320](frontend/src/components/settings/AIConfig.tsx#L313-L320) 条件样式改用 Tailwind className（5 分钟）

### 前置条件（如未来必须移除）
1. motion 库迁移到 CSS Animation / Web Animations API
2. 虚拟滚动重构为 CSS Grid 布局
3. Tauri 2.x CSP nonce 支持确认

---

## 六、执行计划

### 第一轮：ImageViewer (预计 3h)
1. 创建 `useImageZoom.ts` — 提取缩放/拖拽/键盘逻辑
2. 创建 `utils.ts` — 提取工具函数
3. 创建 `ImageInfoPanel.tsx` — 提取右侧面板
4. 创建 `ImageBottomBar.tsx` — 提取底部信息栏
5. 创建 `ImageToolbar.tsx` — 提取工具栏
6. 重构 `index.tsx` — 组合所有子组件
7. 运行测试验证

### 第二轮：KnowledgeGraphView (预计 4h)
1. 创建 `types.ts` + `constants.ts`
2. 创建 `useForceSimulation.ts` — 核心物理引擎
3. 创建 `GraphCanvas.tsx` / `GraphSidebar.tsx` / `GraphHeader.tsx`
4. 重构 `index.tsx`

### 第三轮：GalleryPage (预计 3h)
1. 创建 `useGalleryActions.ts`
2. 创建 `GalleryEmptyState.tsx` / `GalleryActionBar.tsx` / `GallerySearchResults.tsx`
3. 重构 `index.tsx`

### 每轮验收标准
- [ ] `npx tsc --noEmit` 零错误
- [ ] `npm run lint` 零错误
- [ ] `npx vitest run` 223 用例全过
- [ ] 手动验证功能无回归

---

> **文档维护者**: 虫群意志（Swarm Will）
> **下一步**: 用户确认后从 ImageViewer (P0) 开始执行
