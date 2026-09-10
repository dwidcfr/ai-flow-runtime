import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { LoadingError } from '@/shared/components/LoadingError'

describe('LoadingError', () => {
  it('shows loading spinner', () => {
    const { container } = render(<LoadingError loading />)
    expect(container.querySelector('.MuiCircularProgress-root')).toBeTruthy()
  })

  it('shows error message', () => {
    render(<LoadingError error="Something failed" />)
    expect(screen.getByText('Something failed')).toBeInTheDocument()
  })

  it('renders children when no error', () => {
    render(<LoadingError><span>Content</span></LoadingError>)
    expect(screen.getByText('Content')).toBeInTheDocument()
  })
})
