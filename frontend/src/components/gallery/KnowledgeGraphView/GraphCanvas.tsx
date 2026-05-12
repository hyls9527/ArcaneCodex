import type { KgNode } from '../../../lib/api'

interface GraphCanvasProps {
  nodes: KgNode[]
  isBuilding: boolean
  canvasRef: React.RefObject<HTMLCanvasElement | null>
  containerRef: React.RefObject<HTMLDivElement | null>
  onCanvasClick: (e: React.MouseEvent<HTMLCanvasElement>) => void
  onCanvasMouseMove: (e: React.MouseEvent<HTMLCanvasElement>) => void
  onCanvasMouseLeave: () => void
}

export function GraphCanvas({
  nodes,
  isBuilding,
  canvasRef,
  containerRef,
  onCanvasClick,
  onCanvasMouseMove,
  onCanvasMouseLeave,
}: GraphCanvasProps) {
  return (
    <main ref={containerRef} className="flex-1 relative">
      <canvas
        ref={canvasRef}
        onClick={onCanvasClick}
        onMouseMove={onCanvasMouseMove}
        onMouseLeave={onCanvasMouseLeave}
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
  )
}
