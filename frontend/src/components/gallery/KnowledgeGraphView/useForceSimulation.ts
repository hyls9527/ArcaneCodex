import { useCallback, useRef } from 'react'
import type { KgEdge, KgNode } from '../../../lib/api'
import type { ForceEdge, ForceNode } from './types'
import {
  SIMULATION_ALPHA_THRESHOLD,
  SIMULATION_BOUNDARY_PADDING,
  SIMULATION_CENTER_GRAVITY_OTHER,
  SIMULATION_CENTER_GRAVITY_TAG,
  SIMULATION_INITIAL_ALPHA,
  SIMULATION_MAX_ITERATIONS,
  SIMULATION_REPULSION_DISTANCE,
  SIMULATION_REPULSION_STRENGTH,
  SIMULATION_SPRING_REST_LENGTH,
  SIMULATION_SPRING_STRENGTH,
  SIMULATION_VELOCITY_DECAY,
} from './constants'

interface UseForceSimulationOptions {
  containerRef: React.RefObject<HTMLDivElement | null>
  onDraw: (nodeMap: Map<string, ForceNode>, edges: ForceEdge[]) => void
}

interface UseForceSimulationReturn {
  forceNodesRef: React.RefObject<Map<string, ForceNode>>
  edgesRef: React.RefObject<ForceEdge[]>
  initForceLayout: (nodeData: KgNode[], edgeData: KgEdge[], edgeFilter: Set<string>) => void
  findNodeAt: (x: number, y: number) => ForceNode | undefined
  cancelSimulation: () => void
}

export function useForceSimulation({
  containerRef,
  onDraw,
}: UseForceSimulationOptions): UseForceSimulationReturn {
  const forceNodesRef = useRef<Map<string, ForceNode>>(new Map())
  const edgesRef = useRef<ForceEdge[]>([])
  const animationRef = useRef<number>(0)

  const cancelSimulation = useCallback(() => {
    if (animationRef.current) {
      cancelAnimationFrame(animationRef.current)
      animationRef.current = 0
    }
  }, [])

  const startSimulation = useCallback(
    (width: number, height: number) => {
      let iteration = 0
      const alphaDecay = 1 - SIMULATION_INITIAL_ALPHA / SIMULATION_MAX_ITERATIONS
      let currentAlpha = SIMULATION_INITIAL_ALPHA

      const simulate = () => {
        if (iteration >= SIMULATION_MAX_ITERATIONS || currentAlpha < SIMULATION_ALPHA_THRESHOLD) {
          animationRef.current = 0
          return
        }

        const nodeMap = forceNodesRef.current
        const centerX = width / 2
        const centerY = height / 2

        // Center gravity
        nodeMap.forEach(node => {
          if (node.data.node_type === 'tag') {
            node.vx += (centerX - node.x) * SIMULATION_CENTER_GRAVITY_TAG * currentAlpha
            node.vy += (centerY - node.y) * SIMULATION_CENTER_GRAVITY_TAG * currentAlpha
            return
          }

          node.vx += (centerX - node.x) * SIMULATION_CENTER_GRAVITY_OTHER * currentAlpha
          node.vy += (centerY - node.y) * SIMULATION_CENTER_GRAVITY_OTHER * currentAlpha
        })

        // Spring forces along edges
        edgesRef.current.forEach(edge => {
          const source = nodeMap.get(edge.source)
          const target = nodeMap.get(edge.target)
          if (!source || !target) return

          const dx = target.x - source.x
          const dy = target.y - source.y
          const dist = Math.sqrt(dx * dx + dy * dy) || 1
          const force = (dist - SIMULATION_SPRING_REST_LENGTH) * SIMULATION_SPRING_STRENGTH * edge.weight * currentAlpha
          const fx = (dx / dist) * force
          const fy = (dy / dist) * force

          source.vx += fx
          source.vy += fy
          target.vx -= fx
          target.vy -= fy
        })

        // Repulsion between all node pairs
        const nodeArray = Array.from(nodeMap.values())
        for (let i = 0; i < nodeArray.length; i++) {
          for (let j = i + 1; j < nodeArray.length; j++) {
            const a = nodeArray[i]
            const b = nodeArray[j]
            const dx = b.x - a.x
            const dy = b.y - a.y
            const dist = Math.sqrt(dx * dx + dy * dy) || 1
            if (dist < SIMULATION_REPULSION_DISTANCE) {
              const force = ((SIMULATION_REPULSION_DISTANCE - dist) / dist) * SIMULATION_REPULSION_STRENGTH * currentAlpha
              const fx = dx * force
              const fy = dy * force
              a.vx -= fx
              a.vy -= fy
              b.vx += fx
              b.vy += fy
            }
          }
        }

        // Velocity integration + boundary constraint
        nodeMap.forEach(node => {
          node.vx *= SIMULATION_VELOCITY_DECAY
          node.vy *= SIMULATION_VELOCITY_DECAY
          node.x += node.vx
          node.y += node.vy
          node.x = Math.max(SIMULATION_BOUNDARY_PADDING, Math.min(width - SIMULATION_BOUNDARY_PADDING, node.x))
          node.y = Math.max(SIMULATION_BOUNDARY_PADDING, Math.min(height - SIMULATION_BOUNDARY_PADDING, node.y))
        })

        onDraw(nodeMap, edgesRef.current)
        iteration++
        currentAlpha *= alphaDecay
        animationRef.current = requestAnimationFrame(simulate)
      }

      simulate()
    },
    [onDraw],
  )

  const initForceLayout = useCallback(
    (nodeData: KgNode[], edgeData: KgEdge[], edgeFilter: Set<string>) => {
      // Cancel any running simulation before reinitializing
      cancelSimulation()

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
    },
    [containerRef, cancelSimulation, startSimulation],
  )

  const findNodeAt = useCallback((x: number, y: number): ForceNode | undefined => {
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
  }, [])

  return {
    forceNodesRef,
    edgesRef,
    initForceLayout,
    findNodeAt,
    cancelSimulation,
  }
}
