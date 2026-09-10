import { describe, it, expect } from 'vitest'
import { PUBLISH_STAGES } from '@/shared/types'

describe('PUBLISH_STAGES', () => {
  it('includes all publish stages', () => {
    expect(PUBLISH_STAGES).toContain('validate')
    expect(PUBLISH_STAGES).toContain('done')
    expect(PUBLISH_STAGES.length).toBe(9)
  })

  it('ends with done', () => {
    expect(PUBLISH_STAGES[PUBLISH_STAGES.length - 1]).toBe('done')
  })
})
