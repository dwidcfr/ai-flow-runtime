import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import SendIcon from '@mui/icons-material/Send'
import {
  Alert,
  Box,
  Button,
  Chip,
  Divider,
  Paper,
  Stack,
  TextField,
  Typography,
} from '@mui/material'
import ReactMarkdown from 'react-markdown'
import { runtimeApi, studioApi } from '@/shared/api/client'
import { CopyBlock, PageHeader } from '@/shared/components'
import { useSettingsStore } from '@/shared/stores/settingsStore'
import type { HistoryEntryDto, MessageDebugDto } from '@/shared/types'

function DebugPipeline({ debug }: { debug: MessageDebugDto }) {
  return (
    <Stack spacing={2} sx={{ mt: 2 }}>
      <Typography variant="h6">Debug Pipeline</Typography>

      <Typography variant="subtitle2">Search Candidates</Typography>
      {debug.search_candidates.map((c, i) => (
        <CopyBlock
          key={i}
          title={`${c.title} (${c.score.toFixed(3)})`}
          content={`module: ${c.module_id}\nsection: ${c.section_id ?? '—'}\n${c.snippet}`}
          maxHeight={120}
        />
      ))}

      <Typography variant="subtitle2">Flow</Typography>
      <CopyBlock content={JSON.stringify(debug.flow, null, 2)} />

      <Typography variant="subtitle2">Modules</Typography>
      <CopyBlock content={JSON.stringify(debug.modules, null, 2)} />

      <Typography variant="subtitle2">Prompts</Typography>
      <CopyBlock title="Router Prompt" content={debug.prompts.router_prompt} maxHeight={160} />
      <CopyBlock title="Response Prompt" content={debug.prompts.response_prompt} maxHeight={160} />

      <Typography variant="subtitle2">Runtime</Typography>
      <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
        <Chip label={`Search: ${debug.runtime.search_ms}ms`} size="small" />
        <Chip label={`Router: ${debug.runtime.router_ms}ms`} size="small" />
        <Chip label={`Response: ${debug.runtime.response_ms}ms`} size="small" />
        <Chip label={`Total: ${debug.runtime.total_ms}ms`} size="small" />
      </Stack>
      <CopyBlock content={JSON.stringify(debug.runtime.timeline, null, 2)} maxHeight={120} />

      <Typography variant="subtitle2">JSON</Typography>
      <CopyBlock title="Router Raw" content={debug.json.router_raw_json} maxHeight={160} />
      <CopyBlock title="Response Raw" content={debug.json.response_raw_json} maxHeight={160} />
      <CopyBlock title="Router Decision" content={JSON.stringify(debug.json.router_decision, null, 2)} />
    </Stack>
  )
}

export function PlaygroundPage() {
  const debugMode = useSettingsStore((s) => s.debugMode)
  const [projectId, setProjectId] = useState('insurance_demo')
  const [flowId, setFlowId] = useState('payment_flow')
  const [clientPath, setClientPath] = useState('clients/sample.json')
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [message, setMessage] = useState('')
  const [chat, setChat] = useState<{ role: string; content: string }[]>([])
  const [lastDebug, setLastDebug] = useState<MessageDebugDto | null>(null)

  const projectsQuery = useQuery({
    queryKey: ['projects'],
    queryFn: async () => (await studioApi.listProjects()).projects,
  })

  const createSession = useMutation({
    mutationFn: () =>
      runtimeApi.createSession({
        flow_id: flowId,
        client_source_path: clientPath,
      }),
    onSuccess: (data) => {
      setSessionId(data.session_id)
      setChat([])
      if (data.opening_message) {
        setChat([{ role: 'assistant', content: data.opening_message }])
      }
    },
  })

  const sendMessage = useMutation({
    mutationFn: (text: string) =>
      runtimeApi.sendMessage(sessionId!, { message: text }, debugMode),
    onSuccess: (data) => {
      setChat((prev) => [
        ...prev,
        { role: 'assistant', content: data.assistant_message },
      ])
      if (data.debug) setLastDebug(data.debug)
    },
  })

  const historyQuery = useQuery({
    queryKey: ['history', sessionId],
    queryFn: () => runtimeApi.getHistory(sessionId!),
    enabled: !!sessionId,
  })

  const handleSend = () => {
    if (!message.trim() || !sessionId) return
    setChat((prev) => [...prev, { role: 'user', content: message }])
    sendMessage.mutate(message)
    setMessage('')
  }

  return (
    <Box>
      <PageHeader title="Playground" subtitle="Interactive session testing" />
      <Paper variant="outlined" sx={{ p: 2, mb: 3 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextField
            label="Project"
            select
            value={projectId}
            onChange={(e) => setProjectId(e.target.value)}
            SelectProps={{ native: true }}
            sx={{ minWidth: 160 }}
          >
            {(projectsQuery.data ?? []).map((p) => (
              <option key={p.id} value={p.id}>{p.name}</option>
            ))}
          </TextField>
          <TextField label="Flow ID" value={flowId} onChange={(e) => setFlowId(e.target.value)} />
          <TextField label="Client Path" value={clientPath} onChange={(e) => setClientPath(e.target.value)} />
          <Button variant="contained" onClick={() => createSession.mutate()} disabled={createSession.isPending}>
            {sessionId ? 'New Session' : 'Start Session'}
          </Button>
        </Stack>
        {sessionId && (
          <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>
            Session: {sessionId} {debugMode && '(debug on)'}
          </Typography>
        )}
      </Paper>

      <Stack direction={{ xs: 'column', lg: 'row' }} spacing={2}>
        <Paper variant="outlined" sx={{ flex: 1, p: 2, minHeight: 400, display: 'flex', flexDirection: 'column' }}>
          <Box sx={{ flex: 1, overflow: 'auto', mb: 2 }}>
            {chat.map((msg, i) => (
              <Box key={i} sx={{ mb: 2, textAlign: msg.role === 'user' ? 'right' : 'left' }}>
                <Chip label={msg.role} size="small" sx={{ mb: 0.5 }} />
                <Paper sx={{ p: 1.5, display: 'inline-block', maxWidth: '85%', bgcolor: msg.role === 'user' ? 'primary.dark' : 'background.default' }}>
                  <ReactMarkdown>{msg.content}</ReactMarkdown>
                </Paper>
              </Box>
            ))}
            {sendMessage.isPending && <Alert severity="info">Thinking…</Alert>}
          </Box>
          <Divider sx={{ mb: 2 }} />
          <Stack direction="row" spacing={1}>
            <TextField
              fullWidth
              placeholder="Type a message…"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
              disabled={!sessionId}
            />
            <Button variant="contained" endIcon={<SendIcon />} onClick={handleSend} disabled={!sessionId || !message}>
              Send
            </Button>
          </Stack>
        </Paper>

        <Paper variant="outlined" sx={{ width: { lg: 400 }, p: 2, maxHeight: 600, overflow: 'auto' }}>
          <Typography variant="subtitle2" gutterBottom>History</Typography>
          {(historyQuery.data?.entries ?? []).map((e: HistoryEntryDto, i) => (
            <Typography key={i} variant="caption" display="block" sx={{ mb: 0.5 }}>
              [{e.role}] {e.content.slice(0, 80)}
            </Typography>
          ))}
        </Paper>
      </Stack>

      {lastDebug && debugMode && <DebugPipeline debug={lastDebug} />}
    </Box>
  )
}
