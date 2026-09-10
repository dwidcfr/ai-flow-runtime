import { useState } from 'react'
import {
  Alert,
  Box,
  Button,
  FormControlLabel,
  Paper,
  Stack,
  Switch,
  TextField,
  Typography,
} from '@mui/material'
import { useSettingsStore } from '@/shared/stores/settingsStore'
import { PageHeader } from '@/shared/components'
import { adminApi } from '@/shared/api/client'

export function SettingsPage() {
  const apiBaseUrl = useSettingsStore((s) => s.apiBaseUrl)
  const debugMode = useSettingsStore((s) => s.debugMode)
  const setApiBaseUrl = useSettingsStore((s) => s.setApiBaseUrl)
  const setDebugMode = useSettingsStore((s) => s.setDebugMode)

  const [url, setUrl] = useState(apiBaseUrl)
  const [testResult, setTestResult] = useState<string | null>(null)

  const handleSave = () => {
    setApiBaseUrl(url)
    setTestResult('Settings saved')
  }

  const handleTest = async () => {
    try {
      const health = await adminApi.health()
      setTestResult(`Connected: ${health.status}`)
    } catch {
      setTestResult('Connection failed')
    }
  }

  return (
    <Box>
      <PageHeader title="Settings" subtitle="API and debug configuration" />
      <Paper variant="outlined" sx={{ p: 3, maxWidth: 560 }}>
        <Stack spacing={3}>
          <TextField
            label="API Base URL"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            fullWidth
            helperText="Stored in localStorage"
          />
          <FormControlLabel
            control={
              <Switch
                checked={debugMode}
                onChange={(e) => setDebugMode(e.target.checked)}
              />
            }
            label="Debug mode (include pipeline in playground messages)"
          />
          <Stack direction="row" spacing={1}>
            <Button variant="contained" onClick={handleSave}>Save</Button>
            <Button variant="outlined" onClick={handleTest}>Test Connection</Button>
          </Stack>
          {testResult && (
            <Alert severity={testResult.includes('failed') ? 'error' : 'success'}>
              {testResult}
            </Alert>
          )}
          <Typography variant="caption" color="text.secondary">
            Current API: {apiBaseUrl}
          </Typography>
        </Stack>
      </Paper>
    </Box>
  )
}
