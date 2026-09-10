import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { PageHeader } from '@/shared/components/PageHeader'

describe('PageHeader', () => {
  it('renders title', () => {
    render(<PageHeader title="Test Page" />)
    expect(screen.getByRole('heading', { name: 'Test Page' })).toBeInTheDocument()
  })

  it('renders subtitle', () => {
    render(<PageHeader title="Title" subtitle="Subtitle text" />)
    expect(screen.getByText('Subtitle text')).toBeInTheDocument()
  })

  it('renders actions', () => {
    render(<PageHeader title="Title" actions={<button>Action</button>} />)
    expect(screen.getByRole('button', { name: 'Action' })).toBeInTheDocument()
  })
})
