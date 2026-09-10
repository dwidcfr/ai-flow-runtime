import { describe, it, expect } from 'vitest'
import { darkTheme } from '@/app/theme'

describe('darkTheme', () => {
  it('uses dark palette mode', () => {
    expect(darkTheme.palette.mode).toBe('dark')
  })

  it('has rounded shape', () => {
    expect(darkTheme.shape.borderRadius).toBe(12)
  })
})
