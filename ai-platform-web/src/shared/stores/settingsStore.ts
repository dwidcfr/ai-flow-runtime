import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { setApiBaseUrl } from '@/shared/api/client'

const STORAGE_KEY = 'ai-platform-settings'

export interface SettingsState {
  apiBaseUrl: string
  debugMode: boolean
  setApiBaseUrl: (url: string) => void
  setDebugMode: (enabled: boolean) => void
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      apiBaseUrl: 'http://127.0.0.1:8080',
      debugMode: false,
      setApiBaseUrl: (url) => {
        setApiBaseUrl(url)
        set({ apiBaseUrl: url })
      },
      setDebugMode: (enabled) => set({ debugMode: enabled }),
    }),
    {
      name: STORAGE_KEY,
      onRehydrateStorage: () => (state) => {
        if (state?.apiBaseUrl) {
          setApiBaseUrl(state.apiBaseUrl)
        }
      },
    },
  ),
)

export function initSettingsFromStorage(): void {
  const raw = localStorage.getItem(STORAGE_KEY)
  if (!raw) return
  try {
    const parsed = JSON.parse(raw) as { state?: { apiBaseUrl?: string } }
    if (parsed.state?.apiBaseUrl) {
      setApiBaseUrl(parsed.state.apiBaseUrl)
    }
  } catch {
    // ignore corrupt storage
  }
}
