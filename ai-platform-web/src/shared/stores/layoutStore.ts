import { create } from 'zustand'

export interface LayoutState {
  navCollapsed: boolean
  commandPaletteOpen: boolean
  toggleNav: () => void
  setCommandPaletteOpen: (open: boolean) => void
}

export const useLayoutStore = create<LayoutState>((set) => ({
  navCollapsed: false,
  commandPaletteOpen: false,
  toggleNav: () => set((s) => ({ navCollapsed: !s.navCollapsed })),
  setCommandPaletteOpen: (open) => set({ commandPaletteOpen: open }),
}))
