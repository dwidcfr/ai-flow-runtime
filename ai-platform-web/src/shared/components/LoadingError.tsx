import { Alert, Box, Button, CircularProgress } from '@mui/material'
import type { ReactNode } from 'react'

interface LoadingErrorProps {
  loading?: boolean
  error?: Error | string | null
  onRetry?: () => void
  children?: ReactNode
}

export function LoadingError({ loading, error, onRetry, children }: LoadingErrorProps) {
  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}>
        <CircularProgress />
      </Box>
    )
  }

  if (error) {
    const message = typeof error === 'string' ? error : error.message
    return (
      <Alert
        severity="error"
        action={
          onRetry ? (
            <Button color="inherit" size="small" onClick={onRetry}>
              Retry
            </Button>
          ) : undefined
        }
      >
        {message}
      </Alert>
    )
  }

  return <>{children}</>
}
