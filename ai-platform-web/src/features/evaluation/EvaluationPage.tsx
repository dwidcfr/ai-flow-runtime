import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import PlayArrowIcon from '@mui/icons-material/PlayArrow'
import {
  Alert,
  Box,
  Button,
  Chip,
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
import { adminApi } from '@/shared/api/client'
import { LoadingError, PageHeader, StatusChip } from '@/shared/components'
import type { EvalReportDto } from '@/shared/types'

function ResultsTable({ report }: { report: EvalReportDto }) {
  return (
    <Box sx={{ mt: 3 }}>
      <Stack direction="row" spacing={1} sx={{ mb: 2 }}>
        <Chip label={`Total: ${report.metrics.total}`} />
        <Chip label={`Passed: ${report.metrics.passed}`} color="success" />
        <Chip label={`Failed: ${report.metrics.failed}`} color="error" />
        <Chip label={`Skipped: ${report.metrics.skipped}`} />
        <Chip label={`${report.metrics.duration_ms}ms`} variant="outlined" />
      </Stack>
      <TableContainer component={Paper} variant="outlined">
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell>Scenario</TableCell>
              <TableCell>Domain</TableCell>
              <TableCell>Status</TableCell>
              <TableCell>Failures</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {report.results.map((r) => (
              <TableRow key={r.name}>
                <TableCell>{r.name}</TableCell>
                <TableCell>{r.domain ?? '—'}</TableCell>
                <TableCell><StatusChip status={r.status} /></TableCell>
                <TableCell>
                  {r.failures.map((f, i) => (
                    <Typography key={i} variant="caption" display="block">
                      {f.field}: expected {f.expected}, got {f.actual}
                    </Typography>
                  ))}
                  {r.skip_reason && (
                    <Typography variant="caption" color="text.secondary">{r.skip_reason}</Typography>
                  )}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  )
}

export function EvaluationPage() {
  const [report, setReport] = useState<EvalReportDto | null>(null)

  const scenariosQuery = useQuery({
    queryKey: ['eval-scenarios'],
    queryFn: adminApi.listEvalScenarios,
  })

  const runAllMutation = useMutation({
    mutationFn: () => adminApi.runEvalSync({}),
    onSuccess: (data) => {
      if (data.report) setReport(data.report)
    },
  })

  const runOneMutation = useMutation({
    mutationFn: (scenario: string) => adminApi.runEvalSync({ scenario }),
    onSuccess: (data) => {
      if (data.report) setReport(data.report)
    },
  })

  return (
    <Box>
      <PageHeader
        title="Evaluation"
        subtitle="Run eval scenarios"
        actions={
          <Button
            variant="contained"
            startIcon={<PlayArrowIcon />}
            onClick={() => runAllMutation.mutate()}
            disabled={runAllMutation.isPending}
          >
            Run All
          </Button>
        }
      />
      {(runAllMutation.isError || runOneMutation.isError) && (
        <Alert severity="error" sx={{ mb: 2 }}>Eval run failed</Alert>
      )}
      <LoadingError loading={scenariosQuery.isLoading} error={scenariosQuery.error}>
        <TableContainer component={Paper} variant="outlined">
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell>Domain</TableCell>
                <TableCell>LLM</TableCell>
                <TableCell>Source</TableCell>
                <TableCell width={100} />
              </TableRow>
            </TableHead>
            <TableBody>
              {(scenariosQuery.data ?? []).map((s) => (
                <TableRow key={s.name}>
                  <TableCell>{s.name}</TableCell>
                  <TableCell>{s.domain ?? '—'}</TableCell>
                  <TableCell>{s.llm ?? '—'}</TableCell>
                  <TableCell>{s.source_path}</TableCell>
                  <TableCell>
                    <Button
                      size="small"
                      onClick={() => runOneMutation.mutate(s.name)}
                      disabled={runOneMutation.isPending}
                    >
                      Run
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </LoadingError>
      {report && <ResultsTable report={report} />}
    </Box>
  )
}
