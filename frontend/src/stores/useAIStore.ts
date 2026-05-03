import { create } from 'zustand'

export interface AIState {
  status: 'idle' | 'processing' | 'paused' | 'completed' | 'failed'
  total: number
  completed: number
  failed: number
  retrying: number
  eta_seconds?: number
  
  // Actions
  setStatus: (status: AIState['status']) => void
  setTotal: (total: number) => void
  setCompleted: (completed: number) => void
  setFailed: (failed: number) => void
  setRetrying: (retrying: number) => void
  setETA: (eta: number | undefined) => void
  updateStatus: (status: Partial<AIState>) => void
  reset: () => void
}

export const useAIStore = create<AIState>((set) => ({
  status: 'idle',
  total: 0,
  completed: 0,
  failed: 0,
  retrying: 0,
  eta_seconds: undefined,
  
  setStatus: (status) => set({ status }),
  setTotal: (total) => set({ total }),
  setCompleted: (completed) => set({ completed }),
  setFailed: (failed) => set({ failed }),
  setRetrying: (retrying) => set({ retrying }),
  setETA: (eta_seconds) => set({ eta_seconds }),
  updateStatus: (status) => set((state) => ({ ...state, ...status })),
  reset: () => set({
    status: 'idle',
    total: 0,
    completed: 0,
    failed: 0,
    retrying: 0,
    eta_seconds: undefined,
  }),
}))
