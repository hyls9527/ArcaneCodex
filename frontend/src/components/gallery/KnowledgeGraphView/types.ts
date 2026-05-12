import type { KgNode } from '../../../lib/api'

export interface ForceNode {
  id: string
  x: number
  y: number
  vx: number
  vy: number
  data: KgNode
}

export interface ForceEdge {
  source: string
  target: string
  weight: number
  edgeType: string
}
