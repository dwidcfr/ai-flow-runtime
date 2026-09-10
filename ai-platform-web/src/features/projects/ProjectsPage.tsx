import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import EditIcon from '@mui/icons-material/Edit'
import {
  Box,
  Button,
  Card,
  CardActionArea,
  CardContent,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Grid,
  IconButton,
  Stack,
  TextField,
  Typography,
} from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { LoadingError, PageHeader } from '@/shared/components'
import type { CreateProjectRequest } from '@/shared/types'

function ProjectCard({ projectId, name, dirty, onOpen, onRename, onDelete }: {
  projectId: string
  name: string
  dirty: boolean
  onOpen: () => void
  onRename: () => void
  onDelete: () => void
}) {
  const assetsQuery = useQuery({
    queryKey: ['assets', projectId],
    queryFn: () => studioApi.getProjectAssets(projectId),
  })

  return (
    <Card variant="outlined">
      <CardActionArea onClick={onOpen}>
        <CardContent>
          <Stack direction="row" justifyContent="space-between" alignItems="flex-start">
            <Box>
              <Typography variant="h6">{name}</Typography>
              <Typography variant="caption" color="text.secondary">
                {projectId}
              </Typography>
            </Box>
            <Stack direction="row" spacing={0.5}>
              {dirty && <Chip label="Dirty" color="warning" size="small" />}
              <IconButton
                size="small"
                onClick={(e) => { e.stopPropagation(); onRename() }}
              >
                <EditIcon fontSize="small" />
              </IconButton>
              <IconButton
                size="small"
                color="error"
                onClick={(e) => { e.stopPropagation(); onDelete() }}
              >
                <DeleteIcon fontSize="small" />
              </IconButton>
            </Stack>
          </Stack>
          {assetsQuery.data && (
            <Stack direction="row" spacing={1} sx={{ mt: 2 }} flexWrap="wrap" useFlexGap>
              <Chip label={`${assetsQuery.data.flows.length} flows`} size="small" variant="outlined" />
              <Chip label={`${assetsQuery.data.knowledge_documents} docs`} size="small" variant="outlined" />
              <Chip label={`${assetsQuery.data.clients.length} clients`} size="small" variant="outlined" />
            </Stack>
          )}
        </CardContent>
      </CardActionArea>
    </Card>
  )
}

export function ProjectsPage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [createOpen, setCreateOpen] = useState(false)
  const [renameId, setRenameId] = useState<string | null>(null)
  const [deleteId, setDeleteId] = useState<string | null>(null)
  const [formId, setFormId] = useState('')
  const [formName, setFormName] = useState('')
  const [formDesc, setFormDesc] = useState('')

  const projectsQuery = useQuery({
    queryKey: ['projects'],
    queryFn: async () => (await studioApi.listProjects()).projects,
  })

  const createMutation = useMutation({
    mutationFn: (body: CreateProjectRequest) => studioApi.createProject(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      setCreateOpen(false)
      resetForm()
    },
  })

  const renameMutation = useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      studioApi.updateProject(id, { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      setRenameId(null)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => studioApi.deleteProject(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      setDeleteId(null)
    },
  })

  const resetForm = () => {
    setFormId('')
    setFormName('')
    setFormDesc('')
  }

  const handleCreate = () => {
    createMutation.mutate({
      id: formId,
      name: formName,
      description: formDesc || undefined,
    })
  }

  return (
    <Box>
      <PageHeader
        title="Projects"
        subtitle="Manage AI studio projects"
        actions={
          <Button variant="contained" startIcon={<AddIcon />} onClick={() => setCreateOpen(true)}>
            New Project
          </Button>
        }
      />
      <LoadingError
        loading={projectsQuery.isLoading}
        error={projectsQuery.error}
        onRetry={() => projectsQuery.refetch()}
      >
        <Grid container spacing={2}>
          {(projectsQuery.data ?? []).map((p) => (
            <Grid key={p.id} size={{ xs: 12, sm: 6, md: 4 }}>
              <ProjectCard
                projectId={p.id}
                name={p.name}
                dirty={p.dirty}
                onOpen={() => navigate(`/projects/${p.id}/studio/company`)}
                onRename={() => {
                  setRenameId(p.id)
                  setFormName(p.name)
                }}
                onDelete={() => setDeleteId(p.id)}
              />
            </Grid>
          ))}
        </Grid>
      </LoadingError>

      <Dialog open={createOpen} onClose={() => setCreateOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Project</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <TextField label="ID" value={formId} onChange={(e) => setFormId(e.target.value)} fullWidth />
            <TextField label="Name" value={formName} onChange={(e) => setFormName(e.target.value)} fullWidth />
            <TextField label="Description" value={formDesc} onChange={(e) => setFormDesc(e.target.value)} fullWidth multiline />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={handleCreate} disabled={!formId || !formName}>
            Create
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={!!renameId} onClose={() => setRenameId(null)} maxWidth="sm" fullWidth>
        <DialogTitle>Rename Project</DialogTitle>
        <DialogContent>
          <TextField
            label="Name"
            value={formName}
            onChange={(e) => setFormName(e.target.value)}
            fullWidth
            sx={{ mt: 1 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRenameId(null)}>Cancel</Button>
          <Button
            variant="contained"
            onClick={() => renameId && renameMutation.mutate({ id: renameId, name: formName })}
          >
            Save
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={!!deleteId} onClose={() => setDeleteId(null)}>
        <DialogTitle>Delete Project?</DialogTitle>
        <DialogContent>
          <Typography>This action cannot be undone.</Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteId(null)}>Cancel</Button>
          <Button color="error" variant="contained" onClick={() => deleteId && deleteMutation.mutate(deleteId)}>
            Delete
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
