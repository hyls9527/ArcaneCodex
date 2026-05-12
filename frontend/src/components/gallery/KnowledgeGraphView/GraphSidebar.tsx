import type { KgCommunity, KgNeighbor } from '../../../lib/api'
import type { ForceNode } from './types'
import { EDGE_FILTER_OPTIONS } from './constants'

interface GraphSidebarProps {
  edgeFilter: Set<string>
  communities: KgCommunity[]
  selectedNode: string | null
  neighbors: KgNeighbor[]
  forceNodesRef: React.RefObject<Map<string, ForceNode>>
  onToggleEdgeFilter: (type: string) => void
}

export function GraphSidebar({
  edgeFilter,
  communities,
  selectedNode,
  neighbors,
  forceNodesRef,
  onToggleEdgeFilter,
}: GraphSidebarProps) {
  const selectedForceNode = selectedNode ? forceNodesRef.current.get(selectedNode) : undefined

  return (
    <aside className="w-56 border-r border-slate-700/50 p-3 overflow-y-auto space-y-4">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2">Edge Types</h3>
        <div className="space-y-1">
          {EDGE_FILTER_OPTIONS.map(([key, label, color]) => (
            <label key={key} className="flex items-center gap-2 cursor-pointer group">
              <input
                type="checkbox"
                checked={edgeFilter.has(key)}
                onChange={() => onToggleEdgeFilter(key)}
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
            <p className="text-sm font-medium text-white truncate">{selectedForceNode?.data.label}</p>
            <p className="text-[10px] text-slate-400">
              Type: {selectedForceNode?.data.node_type}
            </p>
            <p className="text-[10px] text-slate-400">
              Degree: {selectedForceNode?.data.degree}
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
  )
}
