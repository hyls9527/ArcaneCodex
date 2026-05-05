# ArcaneGallery v2 架构方案 — 深度重构

**文档版本**: v2.0  
**创建日期**: 2026-05-04  
**状态**: 深度研究后重构  
**基于**: 16项联网搜索 + 官方文档 + 最佳实践

---

## 0. 研究发现摘要 — 颠覆性技术突破

### 突破1: `xmpkit` — 纯Rust XMP实现（无需C依赖）

**来源**: [crates.io/xmpkit](https://crates.io/crates/xmpkit) + [github.com/cavivie/xmpkit](https://github.com/cavivie/xmpkit)

之前方案认为XMP写入需要`rexiv2`（依赖gexiv2 C库，Windows编译困难），或调用`exiftool` CLI。**现在发现`xmpkit`**——纯Rust实现的Adobe XMP Toolkit：

- ✅ 纯Rust，零C依赖，跨平台编译无障碍
- ✅ 支持JPEG/PNG/TIFF/MP4/GIF读写XMP
- ✅ 兼容Adobe XMP标准
- ✅ 支持iOS/Android/macOS/Windows/Linux/WASM
- ✅ v0.1.2，2025年12月发布

**同时发现**:
- `xmp-writer` v0.3 (typst团队出品，MIT+Apache双许可，零unsafe，零依赖)
- `exif-oxide` (PhotoStructure出品，Rust版ExifTool，10x faster batch processing)
- `exif-ai` v0.2 (AI驱动EXIF写入，支持本地BLIP模型+sidecar XMP)

**架构影响**: XMP互操作从"P2高风险"升级为"P0低风险"，`xmpkit`直接替代`rexiv2`。

### 突破2: `ort` crate — ONNX Runtime Rust绑定成熟

**来源**: [github.com/microsoft/onnxruntime/rust](https://github.com/microsoft/onnxruntime/blob/main/rust/README.md) + [qiita.com/lamela](https://qiita.com/lamela/items/a5e9c07d6921c3ff0819)

之前方案担心CLIP模型太大/太慢。现在发现：

- `ort` crate v2.0.0-rc.9，支持`download-binaries`特性自动下载预编译ONNX Runtime
- CLIP ViT-B/32 ONNX模型可通过INT8量化将推理速度提升5-10倍
- Rust+ONNX Runtime比Python等价物快3-5x，内存少60-80%
- 支持CPU/GPU自动切换

**架构影响**: 本地CLIP推理从"V2可选"升级为"V1核心"，INT8量化后CPU推理可达<2s/张。

### 突破3: `DistX` — 纯Rust向量数据库（Qdrant API兼容）

**来源**: [crates.io/distx](https://crates.io/crates/distx) + [Rust Forum](https://users.rust-lang.org/t/distx-high-performance-vector-database-in-rust-6x-faster-search-than-qdrant/137032)

之前方案用SQLite存储向量或集成外部Qdrant。现在发现：

- DistX: 纯Rust内存向量数据库，Qdrant API兼容
- 搜索比Qdrant快6.3x，插入比Redis快10x
- SIMD优化(AVX2+NEON)，HNSW索引
- 单二进制，Redis式简洁部署
- 支持Schema-Driven Similarity（无需嵌入模型的结构化相似搜索）

**架构影响**: 向量搜索从"SQLite BLOB+自建HNSW"升级为"DistX嵌入式向量数据库"。

### 突破4: Motion (原Framer Motion) — 混合引擎120fps

**来源**: [motion.dev/docs/react](https://motion.dev/docs/react?fob=FDdVG4xVVnMKXtOw)

之前方案使用framer-motion。现在发现Motion已升级为混合引擎：

- 原生Web Animations API + ScrollTimeline实现120fps
- 弹簧物理、可中断关键帧、手势追踪时无缝回退JS
- Figma和Framer信任的动画引擎
- `layout` prop自动处理FLIP计算
- `LazyMotion`减少初始加载
- 自动尊重`prefers-reduced-motion`

**架构影响**: 动效系统从"framer-motion基础用法"升级为"Motion混合引擎+FLIP+弹簧物理"。

### 突破5: InsightFace ONNX — 本地人脸识别可行

**来源**: [github.com/deepinsight/insightface](https://github.com/deepinsight/insightface) + [CSDN InsightFace+WASM实战](https://blog.csdn.net/gitblog_00908/article/details/152112288)

- buffalo_s模型仅~100MB，CPU上1400fps
- ONNX格式可直接用`ort` crate在Rust中运行
- 512维嵌入向量，cosine距离≤0.35判定同一人
- WASM版本可在浏览器中42ms/帧

**架构影响**: 人脸识别从"V3可选"升级为"V2核心功能"。

### 突破6: IPTC知识图谱标准 — 媒体行业已有本体

**来源**: [iptc.org/themes/knowledge-graphs-semantic-web](https://www.iptc.org/themes/knowledge-graphs-semantic-web/) + [ImageSnippets LIO](https://imagesnippets.com/ArtSpeak/help/ontologies.html)

- IPTC已有rNews(RDF新闻内容模型)→被schema.org采纳为NewsArticle/ImageObject
- Dublin Core + IPTC/XMP + LIO(轻量图像本体)构成图片知识图谱标准
- JSON-LD作为W3C推荐格式，可序列化知识图谱
- Schema.org的ImageObject/CreativeWork提供标准实体类型

**架构影响**: 知识图谱从"自建Schema"升级为"基于IPTC/Dublin Core/schema.org标准"。

---

## 1. 重新定义的架构

### 1.1 架构全景图

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ArcaneGallery v2                              │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    前端 (React + Motion)                     │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │    │
│  │  │ Gallery  │ │ Graph    │ │ Workflow  │ │ Task         │   │    │
│  │  │ +Motion  │ │ Explorer │ │ Engine    │ │ Dashboard    │   │    │
│  │  │ +FLIP    │ │ +D3.js   │ │ +Templates│ │ +Progress    │   │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                              ↕ Tauri IPC                            │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                   后端 (Rust + Tauri 2.x)                    │    │
│  │                                                             │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │              AI推理层 (ort + ONNX)                   │    │    │
│  │  │  ┌──────────┐ ┌──────────┐ ┌────────────────────┐   │    │    │
│  │  │  │ CLIP     │ │ MobileNet│ │ InsightFace        │   │    │    │
│  │  │  │ ViT-B/32 │ │ V3      │ │ buffalo_s          │   │    │    │
│  │  │  │ INT8量化 │ │ 标签分类 │ │ 人脸检测+识别      │   │    │    │
│  │  │  └──────────┘ └──────────┘ └────────────────────┘   │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  │                                                             │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │              知识图谱层 (SQLite + JSON-LD)           │    │    │
│  │  │  ┌──────────┐ ┌──────────┐ ┌────────────────────┐   │    │    │
│  │  │  │ kg_nodes │ │ kg_edges │ │ schema.org/        │   │    │    │
│  │  │  │ 实体节点 │ │ 关系边   │ │ Dublin Core/IPTC   │   │    │    │
│  │  │  └──────────┘ └──────────┘ └────────────────────┘   │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  │                                                             │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │              向量搜索层 (DistX)                      │    │    │
│  │  │  ┌──────────┐ ┌──────────┐ ┌────────────────────┐   │    │    │
│  │  │  │ CLIP嵌入 │ │ 人脸嵌入 │ │ HNSW索引           │   │    │    │
│  │  │  │ 语义搜索 │ │ 人脸搜索 │ │ SIMD加速           │   │    │    │
│  │  │  └──────────┘ └──────────┘ └────────────────────┘   │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  │                                                             │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │              互操作层 (xmpkit + notify)              │    │    │
│  │  │  ┌──────────┐ ┌──────────┐ ┌────────────────────┐   │    │    │
│  │  │  │ XMP读写  │ │ 文件监控 │ │ JSON-LD导出        │   │    │    │
│  │  │  │ xmpkit   │ │ notify   │ │ 开放数据格式       │   │    │    │
│  │  │  └──────────┘ └──────────┘ └────────────────────┘   │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                              ↕                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │              存储层 (SQLite + DistX + 文件系统)              │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │    │
│  │  │ SQLite   │ │ DistX    │ │ 原始文件 │ │ XMP Sidecar  │   │    │
│  │  │ 元数据   │ │ 向量索引 │ │ 保持原样 │ │ 互操作桥梁   │   │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 新增Rust依赖

| 依赖 | 版本 | 用途 | 替代 |
|------|------|------|------|
| `ort` | 2.0.0-rc.9 | ONNX Runtime绑定 | 新增 |
| `ndarray` | 0.16 | 张量计算 | 新增 |
| `xmpkit` | 0.1.2 | XMP读写(纯Rust) | 替代rexiv2 |
| `xmp-writer` | 0.3 | XMP写入(零依赖) | 备选 |
| `notify` | 7 | 跨平台文件监控 | 新增 |
| `distx` | 0.2 | 嵌入式向量数据库 | 替代自建HNSW |

### 1.3 新增前端依赖

| 依赖 | 版本 | 用途 | 替代 |
|------|------|------|------|
| `motion` | latest | 动效混合引擎 | 替代framer-motion |
| `d3` | 7 | 知识图谱可视化 | 新增 |

---

## 2. 五项功能 — 深度重构

### 功能1: 图片知识图谱 — 基于IPTC/schema.org标准

**v1方案**: 自建节点/边Schema  
**v2方案**: 基于IPTC/Dublin Core/schema.org国际标准

#### 标准本体映射

```
schema.org ImageObject ←→ kg_nodes (type=Image)
  ├── schema:caption    ←→ ai_description
  ├── schema:keywords   ←→ ai_tags
  ├── schema:contentLocation ←→ GPS聚类地点
  ├── schema:dateCreated ←→ EXIF拍摄时间
  └── schema:author     ←→ EXIF/IPTC创作者

schema.org Person ←→ kg_nodes (type=Person)
  └── InsightFace 512-D嵌入 → DistX向量搜索

schema.org Place ←→ kg_nodes (type=Place)
  └── GPS聚类(DBSCAN) → 地点实体

schema.org Event ←→ kg_nodes (type=Event)
  └── 时间+地点聚类 → 事件实体

Dublin Core ←→ XMP Sidecar (xmpkit写入)
  ├── dc:title    ←→ 用户标题
  ├── dc:subject  ←→ 标签
  ├── dc:creator  ←→ 创作者
  └── dc:rights   ←→ 版权

IPTC ←→ XMP Sidecar
  ├── iptc4xmpcore:sceneCode ←→ 场景代码
  └── xmpRights:usageTerms   ←→ 使用条款
```

#### 关系边定义（基于schema.org属性）

| 边类型 | schema.org来源 | 含义 |
|--------|---------------|------|
| `DEPICTS` | schema:image | 图片描绘了某人物/物体 |
| `LOCATION` | schema:contentLocation | 拍摄于某地点 |
| `SAME_EVENT` | 推理 | 属于同一事件 |
| `SIMILAR_SCENE` | CLIP余弦相似度 | 场景语义相似 |
| `BEFORE/AFTER` | schema:dateCreated | 时间先后 |
| `SAME_PERSON` | InsightFace嵌入 | 包含同一人物 |

#### JSON-LD导出

```json
{
  "@context": "https://schema.org",
  "@type": "ImageObject",
  "@id": "img_12345",
  "name": "东京塔日落",
  "contentUrl": "file:///photos/tokyo_sunset.jpg",
  "dateCreated": "2025-03-15T18:30:00",
  "contentLocation": {
    "@type": "Place",
    "name": "东京塔",
    "geo": { "@type": "GeoCoordinates", "latitude": 35.6586, "longitude": 139.7454 }
  },
  "about": [
    { "@type": "Person", "name": "张三" },
    { "@type": "Thing", "name": "sunset" }
  ]
}
```

#### 向量搜索架构（DistX）

```
DistX Collection: arcane_images
├── CLIP嵌入 (512维) → 语义搜索
├── InsightFace嵌入 (512维) → 人脸搜索
├── Payload: { tags, date, location, category }
└── HNSW索引 + SIMD加速
```

---

### 功能2: 开放互操作架构 — xmpkit核心

**v1方案**: exiftool CLI或rexiv2  
**v2方案**: xmpkit纯Rust + xmp-writer零依赖

#### XMP Sidecar工作流

```
图片导入 → kamadak-exif读取EXIF
         → xmpkit解析已有XMP
         → AI处理结果写入XMP Sidecar
         → Lightroom/digiKam可直接读取

外部编辑 → notify检测文件变更
         → xmpkit读取更新后的XMP
         → 同步到SQLite数据库

导出 → xmpkit确保XMP嵌入/Sidecar完整
     → JSON-LD导出知识图谱
     → 原始文件保持不变
```

#### 关键技术决策更新

| 决策项 | v1方案 | v2方案 | 理由 |
|--------|--------|--------|------|
| XMP写入 | exiftool CLI | xmpkit纯Rust | 零外部依赖，跨平台 |
| XMP生成 | — | xmp-writer备选 | 零unsafe，零依赖 |
| EXIF读取 | kamadak-exif(已有) | +exif-oxide(未来) | 10x faster batch |
| 文件监控 | notify crate | notify crate(不变) | 成熟跨平台 |

---

### 功能3: 场景化智能工作流 — 知识图谱驱动

**v1方案**: 简单条件匹配  
**v2方案**: 知识图谱图模式匹配驱动

#### 工作流触发机制升级

```
知识图谱图模式匹配 → 场景识别 → 工作流建议

例: 旅行整理
  图模式: (Image)-[:LOCATION]->(Place A)-[:BEFORE]->(Image)-[:LOCATION]->(Place A)
  匹配: 连续多天在同一地点的照片
  触发: "检测到东京旅行(3天, 247张照片)"
```

#### 5个内置工作流（全部可实现）

| 工作流 | 触发图模式 | 依赖 | V1/V2 |
|--------|-----------|------|-------|
| 重复清理 | pHash相似+时间近 | BK-Tree(已有) | V1 |
| 隐私清理 | EXIF含GPS | kamadak-exif(已有) | V1 |
| 旅行整理 | GPS聚类+时间连续 | DBSCAN+知识图谱 | V2 |
| 活动精选 | 时间密集+人脸聚类 | InsightFace+知识图谱 | V2 |
| 素材归档 | AI分类=截图/图标 | AI标签(已有) | V1 |

---

### 功能4: 精致物理动效系统 — Motion混合引擎

**v1方案**: framer-motion基础用法  
**v2方案**: Motion混合引擎+FLIP+弹簧物理+120fps

#### Motion混合引擎架构

```
┌─────────────────────────────────────────────┐
│              Motion 混合引擎                  │
│                                             │
│  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Web Animations  │  │ JavaScript      │  │
│  │ API (WAAPI)     │  │ Fallback        │  │
│  │                 │  │                  │  │
│  │ • 120fps        │  │ • 弹簧物理      │  │
│  │ • GPU加速       │  │ • 可中断关键帧  │  │
│  │ • ScrollTimeline│  │ • 手势追踪      │  │
│  │ • 零JS开销      │  │ • FLIP计算      │  │
│  └─────────────────┘  └─────────────────┘  │
│                                             │
│  自动切换: WAAPI优先 → 需要时回退JS          │
│  prefers-reduced-motion → 自动降级           │
└─────────────────────────────────────────────┘
```

#### 动效规范（修订版）

**弹簧物理参数**:
```typescript
const SPRING = {
  gentle: { type: "spring", stiffness: 120, damping: 14, mass: 1 },
  snappy: { type: "spring", stiffness: 300, damping: 25, mass: 0.8 },
  bouncy: { type: "spring", stiffness: 180, damping: 12, mass: 0.6 },
}
```

**FLIP共享元素过渡** (Motion `layout` prop):
```tsx
<motion.div layout layoutId={`image-${id}`}>
  {/* 缩略图 → 全屏预览，自动FLIP动画 */}
</motion.div>
```

**交错动画** (Variants + staggerChildren):
```tsx
const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.03, when: "beforeChildren" }
  }
}
const item = {
  hidden: { opacity: 0, y: 16, scale: 0.95 },
  show: { opacity: 1, y: 0, scale: 1, transition: SPRING.snappy }
}
```

**AnimatePresence 退出动画**:
```tsx
<AnimatePresence mode="popLayout">
  {images.map(img => (
    <motion.div
      key={img.id}
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8, y: -20 }}
      transition={SPRING.gentle}
    />
  ))}
</AnimatePresence>
```

#### 动效分层（修订版）

| 层级 | 动效 | 技术实现 | 性能 |
|------|------|---------|------|
| L1 微交互 | 按钮弹性、标签弹入、选中描边 | whileHover/whileTap + spring | WAAPI 120fps |
| L2 过渡 | 页面切换、侧边栏、模态框 | AnimatePresence + layout | WAAPI+FLIP |
| L3 搜索 | 结果集交错增删 | Variants + staggerChildren | WAAPI |
| L4 场景 | 导入飞入、删除消散 | layout + AnimatePresence | JS fallback |
| L5 图谱 | 力导向弹性稳定 | D3.js + Motion spring | JS |

---

### 功能5: 透明后台引擎 — 增量处理+检查点

**v1方案**: 基本检查点  
**v2方案**: DistX增量索引 + 检查点持久化 + 资源感知

#### 增量AI处理流程

```
新图片导入
  → 检查DistX是否已有该图片的嵌入
  → 无嵌入 → ONNX推理(CLIP+MobileNet+InsightFace)
  → 有嵌入 → 跳过
  → 写入DistX向量索引
  → 更新知识图谱节点/边
  → 保存检查点到SQLite

恢复处理
  → 读取最后检查点
  → 从断点继续处理
  → 不重复已完成的工作
```

#### ONNX推理管线

```
┌─────────────────────────────────────────────┐
│           ONNX 推理管线 (ort crate)          │
│                                             │
│  图片 → 预处理(224x224)                     │
│       ├→ MobileNetV3 → 标签+分类            │
│       ├→ CLIP ViT-B/32 → 512-D嵌入          │
│       └→ InsightFace buffalo_s → 人脸嵌入   │
│                                             │
│  优化:                                       │
│  • INT8量化 (5-10x加速)                     │
│  • Graph Optimization Level 3               │
│  • intra_threads=4 并行                     │
│  • 批量推理 (batch_size=4)                  │
│                                             │
│  性能预期 (CPU, INT8):                       │
│  • MobileNetV3: ~50ms/张                    │
│  • CLIP: ~1.5s/张                           │
│  • InsightFace: ~100ms/张                   │
│  • 总计: ~2s/张 (三模型串行)                │
└─────────────────────────────────────────────┘
```

---

## 3. 修订后的实施计划

### 批次1: 基础升级 (2-3周)

| 任务 | 新依赖 | 工作量 | 降级方案 |
|------|--------|--------|---------|
| Motion混合引擎迁移 | motion替换framer-motion | 3天 | 保留framer-motion |
| 微交互动效L1 | 无 | 3天 | 减少动效种类 |
| 过渡动效L2 | 无 | 3天 | 简化过渡 |
| xmpkit XMP读写 | xmpkit | 3天 | 仅读取不写入 |
| 文件监控 | notify | 2天 | 定时扫描 |
| **缓冲** | — | 3天 | — |

### 批次2: AI核心 (3-4周)

| 任务 | 新依赖 | 工作量 | 降级方案 |
|------|--------|--------|---------|
| ort ONNX Runtime集成 | ort+ndarray | 4天 | 保留现有API调用方式 |
| MobileNetV3标签分类 | ONNX模型 | 3天 | 保留现有AI Provider |
| CLIP嵌入+语义搜索 | ONNX模型+DistX | 5天 | 仅用AI标签搜索 |
| InsightFace人脸检测 | ONNX模型 | 4天 | V3再实现 |
| DistX向量数据库集成 | distx | 3天 | SQLite BLOB |
| **缓冲** | — | 5天 | — |

### 批次3: 知识图谱+工作流 (3-4周)

| 任务 | 新依赖 | 工作量 | 降级方案 |
|------|--------|--------|---------|
| 知识图谱节点/边构建 | 无 | 4天 | 仅SAME_TAG关系 |
| schema.org/JSON-LD导出 | 无 | 2天 | 仅JSON导出 |
| 图谱可视化(D3.js) | d3 | 4天 | 列表视图替代 |
| 工作流V1(重复+隐私) | 无 | 3天 | 仅重复清理 |
| 工作流V2(旅行+活动) | 知识图谱 | 4天 | V3再实现 |
| 后台引擎升级 | 无 | 3天 | 保留现有 |
| **缓冲** | — | 4天 | — |

---

## 4. 风险评估（修订版）

| 风险 | v1概率 | v2概率 | 变化 | 原因 |
|------|--------|--------|------|------|
| XMP编译问题 | 高 | **极低** | ⬇️⬇️⬇️ | xmpkit纯Rust |
| CLIP太慢 | 中 | **低** | ⬇️⬇️ | INT8量化+ort优化 |
| 向量搜索性能 | 中 | **极低** | ⬇️⬇️⬇️ | DistX SIMD+HNSW |
| 人脸识别不可行 | 中 | **低** | ⬇️⬇️ | InsightFace ONNX成熟 |
| 知识图谱可视化卡顿 | 中 | **中** | → | 仍需限制节点数 |
| Motion迁移风险 | — | **低** | 新增 | API高度兼容framer-motion |
| ort编译问题 | — | **中** | 新增 | download-binaries特性缓解 |

---

## 5. 关键技术验证清单

在正式开发前，需验证以下技术可行性：

- [ ] `xmpkit` 在Windows上编译通过，能读写JPEG XMP
- [ ] `ort` + `download-binaries` 在Windows上正常工作
- [ ] CLIP ViT-B/32 ONNX INT8模型CPU推理<2s
- [ ] MobileNetV3 ONNX模型CPU推理<100ms
- [ ] `distx` 嵌入式模式在Tauri中正常工作
- [ ] `motion` 替换`framer-motion`后现有动画正常
- [ ] `notify` 在Windows/macOS/Linux上文件监控正常
- [ ] InsightFace buffalo_s ONNX模型CPU推理<200ms

---

**文档状态**: ✅ v2架构方案完成（基于深度研究重构）  
**核心变化**: 6项颠覆性技术发现 → 架构全面升级  
**下一步**: 技术验证 → 用户确认 → 进入实施
