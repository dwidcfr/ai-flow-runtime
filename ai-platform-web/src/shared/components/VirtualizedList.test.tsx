import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { VirtualizedList } from '@/shared/components/VirtualizedList'

describe('VirtualizedList', () => {
  it('renders visible items', () => {
    const items = Array.from({ length: 100 }, (_, i) => `item-${i}`)
    render(
      <VirtualizedList
        items={items}
        height={200}
        renderRow={(item) => <div>{item}</div>}
      />,
    )
    expect(screen.getByText('item-0')).toBeInTheDocument()
  })

  it('does not render all items at once', () => {
    const items = Array.from({ length: 200 }, (_, i) => `row-${i}`)
    render(
      <VirtualizedList
        items={items}
        height={100}
        rowHeight={40}
        renderRow={(item) => <div>{item}</div>}
      />,
    )
    expect(screen.queryByText('row-199')).not.toBeInTheDocument()
  })
})
