import { describe, it, expect } from 'vitest'
import { useSettingsStore } from '@/shared/stores/settingsStore'

describe('settingsStore', () => {
  it('has default api url', () => {
    const state = useSettingsStore.getState()
    expect(state.apiBaseUrl).toBeTruthy()
  })

  it('updates debug mode', () => {
    useSettingsStore.getState().setDebugMode(true)
    expect(useSettingsStore.getState().debugMode).toBe(true)
    useSettingsStore.getState().setDebugMode(false)
  })

  it('updates api base url', () => {
    useSettingsStore.getState().setApiBaseUrl('http://test:9999')
    expect(useSettingsStore.getState().apiBaseUrl).toBe('http://test:9999')
  })
})
