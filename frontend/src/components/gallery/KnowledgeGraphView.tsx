import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { kgBuildGraph, kgGetAllEdges, kgGetAllNodes, kgGetCommunities, kgGetNeighbors, kgGetStats, kgLoadFromDb, type KgCommunity, type KgEdge, type KgNode, type KgNeighbor, type KgStats } from '../../lib/api'

interface ForceNode {
  id: string
  x: number
  y: number
  vx: number
  vy: number
  data: KgNode
}

interface ForceEdge {
  source: string
  target: string
  weight: number
  edgeType: string
}

const NODE_COLORS: Record<string, string> = {
  image: '#3b82f6',
  entity: '#10b981',
  tag: '#f59e0b',
  concept: '#8b5cf6',
}

const EDGE_COLORS: Record<string, string> = {
  semantic: 'rgba(59, 130, 246, 0.3)',
  tag_overlap: 'rgba(245, 158, 11, 0.4)',
  temporal: 'rgba(16, 185, 129, 0.3)',
  location: 'rgba(139, 92, 246, 0.3)',
  face_match: 'rgba(239, 68, 68, 0.4)',
}

export default function KnowledgeGraphView() {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const animationRef = useRef<number>(0)
  const [nodes, setNodes] = useState<KgNode[]>([])
  const [edges, setEdges] = useState<KgEdge[]>([])
  const [communities, setCommunities] = useState<KgCommunity[]>([])
  const [stats, setStats] = useState<KgStats | null>(null)
  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [neighbors, setNeighbors] = useState<KgNeighbor[]>([])
  const [isBuilding, setIsBuilding] = useState(false)
  const [hoveredNode, setHoveredNode] = useState<string | null>(null)
  const [edgeFilter, setEdgeFilter] = useState<Set<string>>(new Set(['semantic', 'tag_overlap']))
  const [searchQuery, setSearchQuery] = useState('')

  const forceNodesRef = useRef<Map<string, ForceNode>>(new Map())
  const edgesRef = useRef<ForceEdge[]>([])

  const loadGraphData = useCallback(async () => {
    const [nodesData, edgesData, communitiesData, statsData] = await Promise.all([
      kgGetAllNodes(),
      kgGetAllEdges(),
      kgGetCommunities(),
      kgGetStats(),
    ])

    setNodes(nodesData)
    setEdges(edgesData)
    setCommunities(communitiesData)
    setStats(statsData)

    if (nodesData.length > 0) {
      initForceLayout(nodesData, edgesData)
    }
  }, [])

  const loadGraph = useCallback(async () => {
    setIsBuilding(true)

    try {
      const loadResult = await kgLoadFromDb()
      if (loadResult.success && loadResult.nodes_added > 0) {
        await loadGraphData()
        return
      }

      const buildResult = await kgBuildGraph()
      if (!buildResult.success) {
        console.error('Build failed:', buildResult.error)
        return
      }

      await loadGraphData()
    } finally {
      setIsBuilding(false)
    }
  }, [loadGraphData])

  useEffect(() => {
    loadGraph()
    return () => {
      if (animationRef.current) cancelAnimationFrame(animationRef.current)
    }
  }, [loadGraph])

  const initForceLayout = (nodeData: KgNode[], edgeData: KgEdge[]) => {
    const width = containerRef.current?.clientWidth ?? 800
    const height = containerRef.current?.clientHeight ?? 600

    const nodeMap = new Map<string, ForceNode>()

    nodeData.forEach((node, i) => {
      const angle = (i / nodeData.length) * Math.PI * 2
      const radius = Math.min(width, height) * 0.35
      nodeMap.set(node.id, {
        id: node.id,
        x: width / 2 + Math.cos(angle) * radius + (Math.random() - 0.5) * 50,
        y: height / 2 + Math.sin(angle) * radius + (Math.random() - 0.5) * 50,
        vx: 0,
        vy: 0,
        data: node,
      })
    })

    const filteredEdges = edgeData.filter(e => edgeFilter.has(e.edge_type))
    const forceEdges: ForceEdge[] = filteredEdges.map(e => ({
      source: e.source_id,
      target: e.target_id,
      weight: e.weight,
      edgeType: e.edge_type,
    }))

    forceNodesRef.current = nodeMap
    edgesRef.current = forceEdges

    startSimulation(width, height)
  }

  const startSimulation = (width: number, height: number) => {
    let iteration = 0
    const maxIterations = 300
    const alpha = 0.3
    const alphaDecay = 1 - alpha / maxIterations
    let currentAlpha = alpha

    const simulate = () => {
      if (iteration >= maxIterations || currentAlpha < 0.001) return

      const nodeMap = forceNodesRef.current
      const centerX = width / 2
      const centerY = height / 2

      nodeMap.forEach((node) => {
        if (node.data.node_type === 'tag') {
          node.vx += (centerX - node.x) * 0.01 * currentAlpha
          node.vy += (centerY - node.y) * 0.01 * currentAlpha
          return
        }

        node.vx += (centerX - node.x) * 0.005 * currentAlpha
        node.vy += (centerY - node.y) * 0.005 * currentAlpha
      })

      edgesRef.current.forEach(edge => {
        const source = nodeMap.get(edge.source)
        const target = nodeMap.get(edge.target)
        if (!source || !target) return

        const dx = target.x - source.x
        const dy = target.y - source.y
        const dist = Math.sqrt(dx * dx + dy * dy) || 1
        const force = (dist - 120) * 0.01 * edge.weight * currentAlpha
        const fx = (dx / dist) * force
        const fy = (dy / dist) * force

        source.vx += fx
        source.vy += fy
        target.vx -= fx
        target.vy -= fy
      })

      const nodeArray = Array.from(nodeMap.values())
      for (let i = 0; i < nodeArray.length; i++) {
        for (let j = i + 1; j < nodeArray.length; j++) {
          const a = nodeArray[i]
          const b = nodeArray[j]
          const dx = b.x - a.x
          const dy = b.y - a.y
          const dist = Math.sqrt(dx * dx + dy * dy) || 1
          if (dist < 80) {
            const force = ((80 - dist) / dist) * 0.5 * currentAlpha
            const fx = dx * force
            const fy = dy * force
            a.vx -= fx
            a.vy -= fy
            b.vx += fx
            b.vy += fy
          }
        }
      }

      nodeMap.forEach((node) => {
        node.vx *= 0.9
        node.vy *= 0.9
        node.x += node.vx
        node.y += node.vy
        node.x = Math.max(20, Math.min(width - 20, node.x))
        node.y = Math.max(20, Math.min(height - 20, node.y))
      })

      draw()
      iteration++
      currentAlpha *= alphaDecay
      animationRef.current = requestAnimationFrame(simulate)
    }

    simulate()
  }

  const draw = () => {
    const canvas = canvasRef.current
    const ctx = canvas?.getContext('2d')
    if (!canvas || !ctx) return

    const width = canvas.width
    const height = canvas.height

    ctx.clearRect(0, 0, width, height)

    ctx.fillStyle = '#0f172a'
    ctx.fillRect(0, 0, width, height)

    const nodeMap = forceNodesRef.current

    edgesRef.current.forEach(edge => {
      const source = nodeMap.get(edge.source)
      const target = nodeMap.get(edge.target)
      if (!source || !target) return

      const isSelected = selectedNode === source.id || selectedNode === target.id
      const isHovered = hoveredNode === source.id || hoveredNode === target.id

      ctx.beginPath()
      ctx.moveTo(source.x, source.y)
      ctx.lineTo(target.x, target.y)
      ctx.strokeStyle = EDGE_COLORS[edge.edgeType] ?? 'rgba(148, 163, 184, 0.2)'
      ctx.lineWidth = isSelected ? 2 : isHovered ? 1.5 : 0.8
      ctx.globalAlpha = isSelected || isHovered ? 1 : 0.6
      ctx.stroke()
      ctx.globalAlpha = 1
    })

    if (selectedNode && neighbors.length > 0) {
      neighbors.forEach(n => {
        const node = nodeMap.get(n.node.id)
        if (!node) return
        const selected = nodeMap.get(selectedNode)
        if (!selected) return

        ctx.beginPath()
        ctx.moveTo(selected.x, selected.y)
        ctx.lineTo(node.x, node.y)
        ctx.strokeStyle = 'rgba(251, 191, 36, 0.5)'
        ctx.lineWidth = 2
        ctx.setLineDash([5, 5])
        ctx.stroke()
        ctx.setLineDash([])
      })
    }

    nodeMap.forEach((node) => {
      const isSelected = node.id === selectedNode
      const isHovered = node.id === hoveredNode
      const radius = node.data.node_type === 'image' ? (isSelected ? 10 : isHovered ? 8 : 6) : (isSelected ? 7 : isHovered ? 5 : 4)

      ctx.beginPath()
      ctx.arc(node.x, node.y, radius + 2, 0, Math.PI * 2)
      ctx.fillStyle = NODE_COLORS[node.data.node_type] ?? '#94a3b8'
      ctx.globalAlpha = 0.15
      ctx.fill()
      ctx.globalAlpha = 1

      ctx.beginPath()
      ctx.arc(node.x, node.y, radius, 0, Math.PI * 2)
      ctx.fillStyle = NODE_COLORS[node.data.node_type] ?? '#94a3b8'

      if (isSelected) {
        ctx.shadowColor = NODE_COLORS[node.data.node_type] ?? '#fff'
        ctx.shadowBlur = 15
      } else if (isHovered) {
        ctx.shadowColor = '#fff'
        ctx.shadowBlur = 8
      }

      ctx.fill()
      ctx.shadowBlur = 0

      if (radius >= 6 && node.data.label) {
        ctx.font = `${isSelected ? 12 : 10}px system-ui`
        ctx.fillStyle = '#e2e8f0'
        ctx.textAlign = 'center'
        ctx.fillText(
          node.data.label.length > 15 ? node.data.label.slice(0, 13) + '..' : node.data.label,
          node.x,
          node.y + radius + 14
        )
      }
    })
  }

  const handleCanvasClick = useCallback(async (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current
    if (!canvas) return

    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top

    const clickedNode = findNodeAt(x, y)

    if (clickedNode) {
      setSelectedNode(clickedNode.id)
      try {
        const neighborData = await kgGetNeighbors(clickedNode.id, Array.from(edgeFilter), 30)
        setNeighbors(neighborData)
      } catch {
        setNeighbors([])
      }
    } else {
      setSelectedNode(null)
      setNeighbors([])
    }
  }, [edgeFilter])

  const handleCanvasMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current
    if (!canvas) return

    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top

    const found = findNodeAt(x, y)
    setHoveredNode(found?.id ?? null)
    canvas.style.cursor = found ? 'pointer' : 'default'
  }, [])

  const findNodeAt = (x: number, y: number): ForceNode | undefined => {
    let closest: ForceNode | undefined
    let minDist = Infinity

    forceNodesRef.current.forEach(node => {
      const dx = node.x - x
      const dy = node.y - y
      const dist = Math.sqrt(dx * dx + dy * dy)
      const threshold = node.data.node_type === 'image' ? 12 : 8

      if (dist < threshold && dist < minDist) {
        minDist = dist
        closest = node
      }
    })

    return closest
  }

  const toggleEdgeFilter = (type: string) => {
    setEdgeFilter(prev => {
      const next = new Set(prev)
      if (next.has(type)) next.delete(type)
      else next.add(type)
      return next
    })
  }

  const filteredNodes = useMemo(() => {
    if (!searchQuery.trim()) return nodes
    const q = searchQuery.toLowerCase()
    return nodes.filter(
      n =>
        n.label.toLowerCase().includes(q) ||
        n.properties.description?.toString().toLowerCase().includes(q) ||
        n.properties.category?.toString().toLowerCase().includes(q)
    )
  }, [nodes, searchQuery])

  useEffect(() => {
    if (edges.length > 0) {
      initForceLayout(filteredNodes, edges)
    }
  }, [filteredNodes, edges, edgeFilter])

  return (
    <div className="flex flex-col h-full bg-slate-900 text-slate-100">
      <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700/50">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold text-white">Knowledge Graph</h2>
          {stats && (
            <span className="text-xs text-slate-400">
              {stats.total_nodes} nodes · {stats.total_edges} edges · {stats.communities} communities
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={loadGraph}
            disabled={isBuilding}
            className="px-3 py-1.5 text-xs font-medium rounded-lg bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white transition-colors"
          >
            {isBuilding ? 'Building...' : 'Rebuild Graph'}
          </button>

          <input
            type="text"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            placeholder="Search nodes..."
            className="px-3 py-1.5 text-xs rounded-lg bg-slate-800 border border-slate-600 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:outline-none w-48"
          />
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <aside className="w-56 border-r border-slate-700/50 p-3 overflow-y-auto space-y-4">
          <div>
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2">Edge Types</h3>
            <div className="space-y-1">
              {[
                ['semantic', 'Semantic', 'blue'],
                ['tag_overlap', 'Tag Overlap', 'amber'],
                ['temporal', 'Temporal', 'emerald'],
                ['location', 'Location', 'violet'],
                ['face_match', 'Face Match', 'rose'],
              ].map(([key, label, color]) => (
                <label key={key} className="flex items-center gap-2 cursor-pointer group">
                  <input
                    type="checkbox"
                    checked={edgeFilter.has(key)}
                    onChange={() => toggleEdgeFilter(key)}
                    className="rounded border-slate-500 text-blue-500 focus:ring-blue-500"
                  />
                  <span className={`w-2.5 h-2.5 rounded-full bg-${color}-400`} />
                  <span className="text-xs text-slate-300 group-hover:text-white transition-colors">{label}</span>
                </label>
              ))}
            </div>
          </div>

          {communities.length > 0 && (
            <div>
              <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2">Communities ({communities.length})</h3>
              <div className="space-y-1">
                {communities.slice(0, 10).map(c => (
                  <div key={c.id} className="p-2 rounded bg-slate-800/50 hover:bg-slate-800 transition-colors cursor-pointer" title={`Density: ${(c.density * 100).toFixed(1)}%`}>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-xs font-medium text-white">C{c.id}</span>
                      <span className="text-[10px] text-slate-400">{c.size} nodes</span>
                    </div>
                    <div className="flex flex-wrap gap-1">
                      {c.tags.slice(0, 3).map(t => (
                        <span key={t} className="text-[10px] px-1.5 py-0.5 rounded bg-slate-700 text-slate-300">{t}</span>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {selectedNode && (
            <div>
              <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2">Selected Node</h3>
              <div className="p-2 rounded bg-slate-800/50 space-y-1.5">
                <p className="text-sm font-medium text-white truncate">{forceNodesRef.current.get(selectedNode)?.data.label}</p>
                <p className="text-[10px] text-slate-400">
                  Type: {forceNodesRef.current.get(selectedNode)?.data.node_type}
                </p>
                <p className="text-[10px] text-slate-400">
                  Degree: {forceNodesRef.current.get(selectedNode)?.data.degree}
                </p>
                <p className="text-[10px] text-slate-400">
                  Neighbors: {neighbors.length}
                </p>
              </div>
              {neighbors.length > 0 && (
                <div className="mt-2 space-y-1">
                  <h4 className="text-[10px] font-semibold uppercase tracking-wider text-slate-500">Top Neighbors</h4>
                  {neighbors.slice(0, 8).map(n => (
                    <div key={n.node.id} className="flex items-center justify-between text-[10px] px-1.5 py-1 rounded hover:bg-slate-700/50 cursor-default">
                      <span className="text-slate-300 truncate max-w-[100px]">{n.node.label}</span>
                      <span className="text-amber-400">{(n.edge.weight * 100).toFixed(0)}%</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </aside>

        <main ref={containerRef} className="flex-1 relative">
          <canvas
            ref={canvasRef}
            onClick={handleCanvasClick}
            onMouseMove={handleCanvasMouseMove}
            onMouseLeave={() => setHoveredNode(null)}
            className="w-full h-full"
            width={1200}
            height={800}
          />

          {nodes.length === 0 && !isBuilding && (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="text-center space-y-3">
                <div className="w-16 h-16 mx-auto rounded-full bg-slate-800 flex items-center justify-center">
                  <svg className="w-8 h-8 text-slate-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13.19 8.688a4.5 4.5 0 011.24 7.109a5 5 0 01-6.283 3.03A4.5 4.5 0 018.38 7.048M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <p className="text-sm text-slate-400">No images analyzed yet</p>
                <p className="text-xs text-slate-500">Run AI analysis on your images first, then rebuild the graph</p>
              </div>
            </div>
          )}

          {isBuilding && (
            <div className="absolute inset-0 flex items-center justify-center bg-slate-900/50 backdrop-blur-sm">
              <div className="text-center space-y-2">
                <div className="w-8 h-8 mx-auto border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
                <p className="text-sm text-blue-400">Building Knowledge Graph...</p>
                <p className="text-xs text-slate-500">Analyzing semantic relationships</p>
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  )
}
