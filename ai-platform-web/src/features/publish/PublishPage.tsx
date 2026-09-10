import { useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery } from '@tanstack/react-query'
import PublishIcon from '@mui/icons-material/Publish'
import {
  Alert,
  Box,
  Button,
  Paper,
  Stack,
  Step,
  StepLabel,
  Stepper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Typography,
} from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { CopyBlock, LoadingError, PageHeader } from '@/shared/components'
import { PUBLISH_STAGES, type PublishStage } from '@/shared/types'

function stageIndex(stage?: string): number {
  if (!stage) return -1
  return PUBLISH_STAGES.indexOf(stage as PublishStage)
}

export function PublishPage() {
  const { projectId = '' } = useParams()
  const [activeStage, setActiveStage] = useState(-1)
  const [publishError, setPublishError] = useState<string | null>(null)
  const [publishedVersion, setPublishedVersion] = useState<number | null>(null)

  const versionsQuery = useQuery({
    queryKey: ['versions', projectId],
    queryFn: () => studioApi.listVersions(projectId),
    enabled: !!projectId,
  })

  const publishMutation = useMutation({
    mutationFn: () => studioApi.publishProject(projectId),
    onSuccess: (data) => {
      const idx = stageIndex(data.stage)
      setActiveStage(idx >= 0 ? idx : PUBLISH_STAGES.length - 1)
      if (data.error) setPublishError(data.error)
      if (data.version) setPublishedVersion(data.version)
      if (data.status === 'ok' || data.stage === 'done') {
        versionsQuery.refetch()
      }
    },
    onError: (err) => setPublishError(err.message),
  })

  return (
    <Box>
      <PageHeader
        title="Publish"
        subtitle="Deploy project draft to runtime"
        actions={
          <Button
            variant="contained"
            startIcon={<PublishIcon />}
            onClick={() => publishMutation.mutate()}
            disabled={publishMutation.isPending}
          >
            Publish
          </Button>
        }
      />

      <Paper variant="outlined" sx={{ p: 3, mb: 3 }}>
        <Stepper activeStep={activeStage} alternativeLabel>
          {PUBLISH_STAGES.map((label) => (
            <Step key={label}>
              <StepLabel>{label.replace(/_/g, ' ')}</StepLabel>
            </Step>
          ))}
        </Stepper>
        {publishMutation.isPending && (
          <Typography variant="body2" color="text.secondary" sx={{ mt: 2, textAlign: 'center' }}>
            Publishing…
          </Typography>
        )}
        {publishedVersion !== null && (
          <Alert severity="success" sx={{ mt: 2 }}>
            Published version {publishedVersion}
          </Alert>
        )}
        {publishError && <Alert severity="error" sx={{ mt: 2 }}>{publishError}</Alert>}
        {publishMutation.data?.report && (
          <Box sx={{ mt: 2 }}>
            <CopyBlock content={JSON.stringify(publishMutation.data.report, null, 2)} />
          </Box>
        )}
      </Paper>

      <Typography variant="h6" gutterBottom>Versions</Typography>
      <LoadingError loading={versionsQuery.isLoading} error={versionsQuery.error}>
        <TableContainer component={Paper} variant="outlined">
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Version</TableCell>
                <TableCell>Published At</TableCell>
                <TableCell>Files</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {(versionsQuery.data ?? []).map((v) => (
                <TableRow key={v.version}>
                  <TableCell>v{v.version}</TableCell>
                  <TableCell>{v.published_at}</TableCell>
                  <TableCell>
                    <Stack direction="row" spacing={0.5} flexWrap="wrap" useFlexGap>
                      {v.files.map((f) => (
                        <Typography key={f} variant="caption" component="span">{f}</Typography>
                      ))}
                    </Stack>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </LoadingError>
    </Box>
  )
}
