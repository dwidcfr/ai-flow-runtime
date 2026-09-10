import { useState } from 'react'
import ContentCopyIcon from '@mui/icons-material/ContentCopy'
import CheckIcon from '@mui/icons-material/Check'
import { Box, IconButton, Paper, Typography } from '@mui/material'

interface CopyBlockProps {
  title?: string
  content: string
  maxHeight?: number
}

export function CopyBlock({ title, content, maxHeight = 240 }: CopyBlockProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(content)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Paper variant="outlined" sx={{ p: 1.5, bgcolor: 'background.default' }}>
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
        {title && (
          <Typography variant="caption" color="text.secondary" fontWeight={600}>
            {title}
          </Typography>
        )}
        <IconButton size="small" onClick={handleCopy} aria-label="Copy">
          {copied ? <CheckIcon fontSize="small" /> : <ContentCopyIcon fontSize="small" />}
        </IconButton>
      </Box>
      <Box
        component="pre"
        sx={{
          m: 0,
          p: 1,
          overflow: 'auto',
          maxHeight,
          fontSize: '0.75rem',
          fontFamily: 'monospace',
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
        }}
      >
        {content}
      </Box>
    </Paper>
  )
}
