import { create } from 'zustand'
import { type DuplicateGroup } from '../lib/api'

export interface DedupState {
  groups: DuplicateGroup[]
  loading: boolean
  threshold: number

  setGroups: (groups: DuplicateGroup[]) => void
  setLoading: (loading: boolean) => void
  setThreshold: (threshold: number) => void
  removeGroups: (ids: string[]) => void
  reset: () => void
}

export const useDedupStore = create<DedupState>((set) => ({
  groups: [],
  loading: false,
  threshold: 95,

  setGroups: (groups) => set({ groups }),
  setLoading: (loading) => set({ loading }),
  setThreshold: (threshold) => set({ threshold }),
  removeGroups: (ids) => set((state) => ({
    groups: state.groups.filter(g => !ids.includes(g.id)),
  })),
  reset: () => set({ groups: [], loading: false, threshold: 95 }),
}))
