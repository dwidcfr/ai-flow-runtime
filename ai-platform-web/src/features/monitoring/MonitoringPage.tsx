import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Box,
  Card,
  CardContent,
  Grid,
  Paper,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Typography,
} from '@mui/material'
import { adminApi, runtimeApi } from '@/shared/api/client'
import { CopyBlock, LoadingError, PageHeader, StatusChip } from '@/shared/components'

export function MonitoringPage() {
  const [selectedSession, setSelectedSession] = useState<string | null>(null)

  const overviewQuery = useQuery({
    queryKey: ['overview'],
    queryFn: adminApi.overview,
    refetchInterval: 15_000,
  })

  const platformQuery = useQuery({
    queryKey: ['platform-status'],
    queryFn: adminApi.platformStatus,
    refetchInterval: 15_000,
  })

  const sessionsQuery = useQuery({
    queryKey: ['sessions'],
    queryFn: runtimeApi.listSessions,
    refetchInterval: 10_000,
  })

  const inspectQuery = useQuery({
    queryKey: ['session-inspect', selectedSession],
    queryFn: () => runtimeApi.inspectSession(selectedSession!),
    enabled: !!selectedSession,
  })

  const counts = overviewQuery.data?.counts

  return (
    <Box>
      <PageHeader title="Monitoring" subtitle="Platform health and active sessions" />
      <LoadingError loading={overviewQuery.isLoading} error={overviewQuery.error}>
        <Grid container spacing={2} sx={{ mb: 3 }}>
          {[
            { label: 'Active Sessions', value: counts?.active_sessions ?? 0 },
            { label: 'Flows', value: counts?.flows ?? 0 },
            { label: 'Prompt Sets', value: counts?.prompt_sets ?? 0 },
            { label: 'Modules', value: counts?.modules ?? 0 },
            { label: 'Indexed Docs', value: counts?.search_index_documents ?? 0 },
          ].map((card) => (
            <Grid key={card.label} size={{ xs: 6, sm: 4, md: 2.4 }}>
              <Card variant="outlined">
                <CardContent>
                  <Typography variant="h4">{card.value}</Typography>
                  <Typography variant="body2" color="text.secondary">{card.label}</Typography>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>
        {overviewQuery.data && (
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            API {overviewQuery.data.api_version} · Runtime {overviewQuery.data.runtime_version} · Uptime {overviewQuery.data.uptime_secs}s
            · Gemini {overviewQuery.data.gemini.configured ? 'configured' : 'mock'}
          </Typography>
        )}
      </LoadingError>

      <Typography variant="h6" sx={{ mb: 1 }}>Platform Status</Typography>
      <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{ mb: 3 }}>
        {(platformQuery.data?.components ?? []).map((c) => (
          <StatusChip key={c.id} status={c.status} label={`${c.name}: ${c.status}`} />
        ))}
      </Stack>

      <Typography variant="h6" sx={{ mb: 1 }}>Sessions</Typography>
      <Grid container spacing={2}>
        <Grid size={{ xs: 12, md: 7 }}>
          <TableContainer component={Paper} variant="outlined">
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Session</TableCell>
                  <TableCell>Flow</TableCell>
                  <TableCell>State</TableCell>
                  <TableCell>Node</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {(sessionsQuery.data ?? []).map((s) => (
                  <TableRow
                    key={s.session_id}
                    hover
                    selected={selectedSession === s.session_id}
                    onClick={() => setSelectedSession(s.session_id)}
                    sx={{ cursor: 'pointer' }}
                  >
                    <TableCell>{s.session_id.slice(0, 8)}…</TableCell>
                    <TableCell>{s.flow_id ?? '—'}</TableCell>
                    <TableCell><StatusChip status={s.runtime_state} /></TableCell>
                    <TableCell>{s.current_node ?? '—'}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </Grid>
        <Grid size={{ xs: 12, md: 5 }}>
          {selectedSession && inspectQuery.data && (
            <CopyBlock
              title={`Session ${selectedSession.slice(0, 8)}`}
              content={JSON.stringify(inspectQuery.data, null, 2)}
              maxHeight={400}
            />
          )}
        </Grid>
      </Grid>
    </Box>
  )
}
