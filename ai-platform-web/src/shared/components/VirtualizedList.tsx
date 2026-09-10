import { useCallback, useRef, useState, type UIEvent } from 'react'
import { Box } from '@mui/material'

const ROW_HEIGHT = 40

interface VirtualizedListProps<T> {
  items: T[]
  rowHeight?: number
  height: number
  renderRow: (item: T, index: number) => React.ReactNode
  overscan?: number
}

export function VirtualizedList<T>({
  items,
  rowHeight = ROW_HEIGHT,
  height,
  renderRow,
  overscan = 5,
}: VirtualizedListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0)
  const containerRef = useRef<HTMLDivElement>(null)

  const onScroll = useCallback((e: UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop)
  }, [])

  const totalHeight = items.length * rowHeight
  const startIndex = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan)
  const visibleCount = Math.ceil(height / rowHeight) + overscan * 2
  const endIndex = Math.min(items.length, startIndex + visibleCount)
  const visibleItems = items.slice(startIndex, endIndex)

  return (
    <Box
      ref={containerRef}
      onScroll={onScroll}
      sx={{ height, overflow: 'auto', position: 'relative' }}
    >
      <Box sx={{ height: totalHeight, position: 'relative' }}>
        {visibleItems.map((item, i) => {
          const index = startIndex + i
          return (
            <Box
              key={index}
              sx={{
                position: 'absolute',
                top: index * rowHeight,
                left: 0,
                right: 0,
                height: rowHeight,
              }}
            >
              {renderRow(item, index)}
            </Box>
          )
        })}
      </Box>
    </Box>
  )
}
