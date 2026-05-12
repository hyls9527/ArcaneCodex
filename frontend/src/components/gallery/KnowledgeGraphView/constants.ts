export const NODE_COLORS: Record<string, string> = {
  image: '#3b82f6',
  entity: '#10b981',
  tag: '#f59e0b',
  concept: '#8b5cf6',
}

export const EDGE_COLORS: Record<string, string> = {
  semantic: 'rgba(59, 130, 246, 0.3)',
  tag_overlap: 'rgba(245, 158, 11, 0.4)',
  temporal: 'rgba(16, 185, 129, 0.3)',
  location: 'rgba(139, 92, 246, 0.3)',
  face_match: 'rgba(239, 68, 68, 0.4)',
}

export const EDGE_FILTER_OPTIONS: [string, string, string][] = [
  ['semantic', 'Semantic', 'blue'],
  ['tag_overlap', 'Tag Overlap', 'amber'],
  ['temporal', 'Temporal', 'emerald'],
  ['location', 'Location', 'violet'],
  ['face_match', 'Face Match', 'rose'],
]

export const SIMULATION_MAX_ITERATIONS = 300
export const SIMULATION_INITIAL_ALPHA = 0.3
export const SIMULATION_ALPHA_THRESHOLD = 0.001
export const SIMULATION_VELOCITY_DECAY = 0.9
export const SIMULATION_REPULSION_DISTANCE = 80
export const SIMULATION_SPRING_REST_LENGTH = 120
export const SIMULATION_CENTER_GRAVITY_TAG = 0.01
export const SIMULATION_CENTER_GRAVITY_OTHER = 0.005
export const SIMULATION_SPRING_STRENGTH = 0.01
export const SIMULATION_REPULSION_STRENGTH = 0.5
export const SIMULATION_BOUNDARY_PADDING = 20
