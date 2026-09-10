import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { StatusChip } from '@/shared/components/StatusChip'

describe('StatusChip', () => {
  it('renders status label', () => {
    render(<StatusChip status="ok" />)
    expect(screen.getByText('ok')).toBeInTheDocument()
  })

  it('uses custom label', () => {
    render(<StatusChip status="error" label="Failed" />)
    expect(screen.getByText('Failed')).toBeInTheDocument()
  })

  it('maps passed to success color', () => {
    const { container } = render(<StatusChip status="passed" />)
    expect(container.querySelector('.MuiChip-colorSuccess')).toBeTruthy()
  })
})
