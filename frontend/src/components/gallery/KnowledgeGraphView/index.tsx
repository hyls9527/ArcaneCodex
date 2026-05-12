import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  kgBuildGraph,
  kgGetAllEdges,
  kgGetAllNodes,
  kgGetCommunities,
  kgGetNeighbors,
  kgGetStats,
  kgLoadFromDb,
  type KgCommunity,
  type KgEdge,
  type KgNeighbor,
  type KgNode,
  type KgStats,
} from '../../../lib/api'
import { EDGE_COLORS, NODE_COLORS } from './constants'
import { GraphCanvas } from './GraphCanvas'
import { GraphHeader } from './GraphHeader'
import { GraphSidebar } from './GraphSidebar'
import type { ForceEdge, ForceNode } from './types'
import { useForceSimulation } from './useForceSimulation'

export default function KnowledgeGraphView() {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)

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

  // Ref-based draw function to avoid stale closures in the simulation loop.
  // The simulation calls onDraw via this ref, so it always gets the latest
  // selectedNode / hoveredNode / neighbors values without restarting.
  const selectedNodeRef = useRef(selectedNode)
  const hoveredNodeRef = useRef(hoveredNode)
  const neighborsRef = useRef(neighbors)

  useEffect(() => { selectedNodeRef.current = selectedNode }, [selectedNode])
  useEffect(() => { hoveredNodeRef.current = hoveredNode }, [hoveredNode])
  useEffect(() => { neighborsRef.current = neighbors }, [neighbors])

  const draw = useCallback(
    (nodeMap: Map<string, ForceNode>, forceEdges: ForceEdge[]) => {
      const canvas = canvasRef.current
      const ctx = canvas?.getContext('2d')
      if (!canvas || !ctx) return

      const width = canvas.width
      const height = canvas.height

      ctx.clearRect(0, 0, width, height)
      ctx.fillStyle = '#0f172a'
      ctx.fillRect(0, 0, width, height)

      // Draw edges
      forceEdges.forEach(edge => {
        const source = nodeMap.get(edge.source)
        const target = nodeMap.get(edge.target)
        if (!source || !target) return

        const isSelected = selectedNodeRef.current === source.id || selectedNodeRef.current === target.id
        const isHovered = hoveredNodeRef.current === source.id || hoveredNodeRef.current === target.id

        ctx.beginPath()
        ctx.moveTo(source.x, source.y)
        ctx.lineTo(target.x, target.y)
        ctx.strokeStyle = EDGE_COLORS[edge.edgeType] ?? 'rgba(148, 163, 184, 0.2)'
        ctx.lineWidth = isSelected ? 2 : isHovered ? 1.5 : 0.8
        ctx.globalAlpha = isSelected || isHovered ? 1 : 0.6
        ctx.stroke()
        ctx.globalAlpha = 1
      })

      // Draw neighbor highlight edges
      if (selectedNodeRef.current && neighborsRef.current.length > 0) {
        neighborsRef.current.forEach(n => {
          const node = nodeMap.get(n.node.id)
          if (!node) return
          const selected = nodeMap.get(selectedNodeRef.current!)
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

      // Draw nodes
      nodeMap.forEach(node => {
        const isSelected = node.id === selectedNodeRef.current
        const isHovered = node.id === hoveredNodeRef.current
        const radius = node.data.node_type === 'image' ? (isSelected ? 10 : isHovered ? 8 : 6) : (isSelected ? 7 : isHovered ? 5 : 4)

        // Glow ring
        ctx.beginPath()
        ctx.arc(node.x, node.y, radius + 2, 0, Math.PI * 2)
        ctx.fillStyle = NODE_COLORS[node.data.node_type] ?? '#94a3b8'
        ctx.globalAlpha = 0.15
        ctx.fill()
        ctx.globalAlpha = 1

        // Node circle
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

        // Label
        if (radius >= 6 && node.data.label) {
          ctx.font = `${isSelected ? 12 : 10}px system-ui`
          ctx.fillStyle = '#e2e8f0'
          ctx.textAlign = 'center'
          ctx.fillText(
            node.data.label.length > 15 ? node.data.label.slice(0, 13) + '..' : node.data.label,
            node.x,
            node.y + radius + 14,
          )
        }
      })
    },
    [], // intentionally empty — reads from refs to avoid stale closures
  )

  const { forceNodesRef, initForceLayout, findNodeAt, cancelSimulation } = useForceSimulation({
    containerRef,
    onDraw: draw,
  })

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
      initForceLayout(nodesData, edgesData, edgeFilter)
    }
  }, [initForceLayout, edgeFilter])

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
      cancelSimulation()
    }
  }, [loadGraph, cancelSimulation])

  const filteredNodes = useMemo(() => {
    if (!searchQuery.trim()) return nodes
    const q = searchQuery.toLowerCase()
    return nodes.filter(
      n =>
        n.label.toLowerCase().includes(q) ||
        n.properties.description?.toString().toLowerCase().includes(q) ||
        n.properties.category?.toString().toLowerCase().includes(q),
    )
  }, [nodes, searchQuery])

  useEffect(() => {
    if (edges.length > 0) {
      initForceLayout(filteredNodes, edges, edgeFilter)
    }
  }, [filteredNodes, edges, edgeFilter, initForceLayout])

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
  }, [edgeFilter, findNodeAt])

  const handleCanvasMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current
    if (!canvas) return

    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top

    const found = findNodeAt(x, y)
    setHoveredNode(found?.id ?? null)
    canvas.style.cursor = found ? 'pointer' : 'default'
  }, [findNodeAt])

  const toggleEdgeFilter = useCallback((type: string) => {
    setEdgeFilter(prev => {
      const next = new Set(prev)
      if (next.has(type)) next.delete(type)
      else next.add(type)
      return next
    })
  }, [])

  return (
    <div className="flex flex-col h-full bg-slate-900 text-slate-100">
      <GraphHeader
        stats={stats}
        isBuilding={isBuilding}
        searchQuery={searchQuery}
        onRebuild={loadGraph}
        onSearchChange={setSearchQuery}
      />

      <div className="flex flex-1 overflow-hidden">
        <GraphSidebar
          edgeFilter={edgeFilter}
          communities={communities}
          selectedNode={selectedNode}
          neighbors={neighbors}
          forceNodesRef={forceNodesRef}
          onToggleEdgeFilter={toggleEdgeFilter}
        />

        <GraphCanvas
          nodes={nodes}
          isBuilding={isBuilding}
          canvasRef={canvasRef}
          containerRef={containerRef}
          onCanvasClick={handleCanvasClick}
          onCanvasMouseMove={handleCanvasMouseMove}
          onCanvasMouseLeave={() => setHoveredNode(null)}
        />
      </div>
    </div>
  )
}
