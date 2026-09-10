import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { CopyBlock } from '@/shared/components/CopyBlock'

describe('CopyBlock', () => {
  it('renders content', () => {
    render(<CopyBlock content="hello world" />)
    expect(screen.getByText('hello world')).toBeInTheDocument()
  })

  it('renders title when provided', () => {
    render(<CopyBlock title="Debug" content="data" />)
    expect(screen.getByText('Debug')).toBeInTheDocument()
  })

  it('copies to clipboard on click', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.assign(navigator, { clipboard: { writeText } })
    render(<CopyBlock content="copy me" />)
    fireEvent.click(screen.getByLabelText('Copy'))
    await waitFor(() => expect(writeText).toHaveBeenCalledWith('copy me'))
  })
})
