import { Chip, type ChipProps } from '@mui/material'

const STATUS_COLORS: Record<string, ChipProps['color']> = {
  ok: 'success',
  healthy: 'success',
  passed: 'success',
  done: 'success',
  success: 'success',
  active: 'info',
  running: 'info',
  dirty: 'warning',
  warning: 'warning',
  skipped: 'warning',
  error: 'error',
  failed: 'error',
  unhealthy: 'error',
  offline: 'error',
}

interface StatusChipProps {
  status: string
  label?: string
  size?: ChipProps['size']
}

export function StatusChip({ status, label, size = 'small' }: StatusChipProps) {
  const normalized = status.toLowerCase()
  const color = STATUS_COLORS[normalized] ?? 'default'
  return <Chip label={label ?? status} color={color} size={size} variant="outlined" />
}
