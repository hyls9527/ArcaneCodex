import type { KgStats } from '../../../lib/api'

interface GraphHeaderProps {
  stats: KgStats | null
  isBuilding: boolean
  searchQuery: string
  onRebuild: () => void
  onSearchChange: (query: string) => void
}

export function GraphHeader({
  stats,
  isBuilding,
  searchQuery,
  onRebuild,
  onSearchChange,
}: GraphHeaderProps) {
  return (
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
          onClick={onRebuild}
          disabled={isBuilding}
          className="px-3 py-1.5 text-xs font-medium rounded-lg bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white transition-colors"
        >
          {isBuilding ? 'Building...' : 'Rebuild Graph'}
        </button>

        <input
          type="text"
          value={searchQuery}
          onChange={e => onSearchChange(e.target.value)}
          placeholder="Search nodes..."
          className="px-3 py-1.5 text-xs rounded-lg bg-slate-800 border border-slate-600 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:outline-none w-48"
        />
      </div>
    </div>
  )
}
