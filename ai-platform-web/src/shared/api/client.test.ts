import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setApiBaseUrl, getApiBaseUrl, studioApi, adminApi } from '@/shared/api/client'

vi.mock('axios', () => {
  const instance = {
    defaults: { baseURL: 'http://127.0.0.1:8080' },
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    patch: vi.fn(),
    delete: vi.fn(),
  }
  return {
    default: {
      create: vi.fn(() => instance),
    },
  }
})

import axios from 'axios'

describe('API client', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setApiBaseUrl('http://127.0.0.1:8080')
  })

  it('returns default base url', () => {
    expect(getApiBaseUrl()).toBe('http://127.0.0.1:8080')
  })

  it('updates base url', () => {
    setApiBaseUrl('http://localhost:9000/')
    expect(getApiBaseUrl()).toBe('http://localhost:9000')
  })

  it('studioApi.listProjects calls correct endpoint', async () => {
    const mockGet = vi.mocked(axios.create().get)
    mockGet.mockResolvedValue({ data: { projects: [] } })
    await studioApi.listProjects()
    expect(mockGet).toHaveBeenCalledWith('/studio/projects')
  })

  it('adminApi.health calls correct endpoint', async () => {
    const mockGet = vi.mocked(axios.create().get)
    mockGet.mockResolvedValue({ data: { status: 'ok' } })
    const result = await adminApi.health()
    expect(mockGet).toHaveBeenCalledWith('/health')
    expect(result.status).toBe('ok')
  })
})
