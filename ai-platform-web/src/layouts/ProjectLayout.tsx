import { useMemo } from 'react'
import { Outlet, useNavigate, useParams, useLocation } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { Box, Tab, Tabs, Typography } from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { CommandPalette, type CommandItem, LoadingError, StatusChip } from '@/shared/components'

const PROJECT_TABS = [
  { label: 'Company', segment: 'company' },
  { label: 'Knowledge', segment: 'knowledge' },
  { label: 'Search', segment: 'search' },
  { label: 'Flows', segment: 'flows' },
  { label: 'Prompts', segment: 'prompts' },
  { label: 'Clients', segment: 'clients' },
  { label: 'Publish', segment: 'publish' },
  { label: 'Assets', segment: 'assets' },
]

export function ProjectLayout() {
  const { projectId = '' } = useParams()
  const navigate = useNavigate()
  const location = useLocation()

  const projectQuery = useQuery({
    queryKey: ['project', projectId],
    queryFn: () => studioApi.getProject(projectId),
    enabled: !!projectId,
  })

  const activeTab = useMemo(() => {
    const path = location.pathname
    if (path.includes('/publish')) return 'publish'
    if (path.includes('/assets')) return 'assets'
    if (path.includes('/studio/flows')) return 'flows'
    const match = PROJECT_TABS.find((t) => path.includes(`/studio/${t.segment}`))
    return match?.segment ?? 'company'
  }, [location.pathname])

  const handleTabChange = (_: unknown, value: string) => {
    if (value === 'publish') {
      navigate(`/projects/${projectId}/publish`)
    } else if (value === 'assets') {
      navigate(`/projects/${projectId}/assets`)
    } else {
      navigate(`/projects/${projectId}/studio/${value}`)
    }
  }

  const projectCommands: CommandItem[] = PROJECT_TABS.map((t) => ({
    id: `project-${t.segment}`,
    label: `${projectQuery.data?.name ?? projectId} — ${t.label}`,
    path:
      t.segment === 'publish'
        ? `/projects/${projectId}/publish`
        : t.segment === 'assets'
          ? `/projects/${projectId}/assets`
          : `/projects/${projectId}/studio/${t.segment}`,
    group: 'Project',
  }))

  return (
    <Box>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
        <Typography variant="h5" fontWeight={600}>
          {projectQuery.data?.name ?? projectId}
        </Typography>
        {projectQuery.data?.dirty && <StatusChip status="dirty" label="Unpublished changes" />}
        {projectQuery.data && (
          <StatusChip
            status="ok"
            label={`v${projectQuery.data.published_version}`}
          />
        )}
      </Box>
      <Tabs value={activeTab} onChange={handleTabChange} variant="scrollable" sx={{ mb: 3 }}>
        {PROJECT_TABS.map((tab) => (
          <Tab key={tab.segment} label={tab.label} value={tab.segment} />
        ))}
      </Tabs>
      <LoadingError
        loading={projectQuery.isLoading}
        error={projectQuery.error}
        onRetry={() => projectQuery.refetch()}
      >
        <Outlet />
      </LoadingError>
      <CommandPalette projectCommands={projectCommands} />
    </Box>
  )
}
