import { describe, it, expect } from 'vitest'
import { useLayoutStore } from '@/shared/stores/layoutStore'

describe('layoutStore', () => {
  it('toggles nav collapsed', () => {
    const initial = useLayoutStore.getState().navCollapsed
    useLayoutStore.getState().toggleNav()
    expect(useLayoutStore.getState().navCollapsed).toBe(!initial)
  })

  it('sets command palette open', () => {
    useLayoutStore.getState().setCommandPaletteOpen(true)
    expect(useLayoutStore.getState().commandPaletteOpen).toBe(true)
  })
})
