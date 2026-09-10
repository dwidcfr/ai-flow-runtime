import { useParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import {
  Box,
  Chip,
  Grid,
  Paper,
  Stack,
  Typography,
} from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { LoadingError, PageHeader } from '@/shared/components'

export function AssetsPage() {
  const { projectId = '' } = useParams()

  const assetsQuery = useQuery({
    queryKey: ['assets', projectId],
    queryFn: () => studioApi.getProjectAssets(projectId),
    enabled: !!projectId,
  })

  const assets = assetsQuery.data

  return (
    <Box>
      <PageHeader title="Assets" subtitle="Project resource inventory" />
      <LoadingError loading={assetsQuery.isLoading} error={assetsQuery.error}>
        {assets && (
          <Grid container spacing={2}>
            <Grid size={{ xs: 12, md: 6 }}>
              <Paper variant="outlined" sx={{ p: 2 }}>
                <Typography variant="subtitle2" gutterBottom>Company</Typography>
                <Typography>{assets.company.name} ({assets.company.id})</Typography>
                <Typography variant="caption" color="text.secondary">
                  Prompt: {assets.company.prompt_set} · Knowledge: {assets.company.knowledge_source}
                </Typography>
              </Paper>
            </Grid>
            <Grid size={{ xs: 12, md: 6 }}>
              <Paper variant="outlined" sx={{ p: 2 }}>
                <Typography variant="subtitle2" gutterBottom>Knowledge</Typography>
                <Typography>{assets.knowledge_documents} documents · {assets.knowledge_categories} categories</Typography>
              </Paper>
            </Grid>
            {[
              { title: 'Flows', items: assets.flows },
              { title: 'Prompt Sets', items: assets.prompt_sets },
              { title: 'Clients', items: assets.clients },
              { title: 'Modules', items: assets.modules },
            ].map((section) => (
              <Grid key={section.title} size={{ xs: 12, sm: 6 }}>
                <Paper variant="outlined" sx={{ p: 2 }}>
                  <Typography variant="subtitle2" gutterBottom>{section.title}</Typography>
                  <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                    {section.items.map((item) => (
                      <Chip key={item} label={item} size="small" variant="outlined" />
                    ))}
                    {section.items.length === 0 && (
                      <Typography variant="body2" color="text.secondary">None</Typography>
                    )}
                  </Stack>
                </Paper>
              </Grid>
            ))}
          </Grid>
        )}
      </LoadingError>
    </Box>
  )
}
