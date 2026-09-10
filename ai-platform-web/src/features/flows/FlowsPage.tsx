import { useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import AddIcon from '@mui/icons-material/Add'
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  TextField,
} from '@mui/material'
import DeleteIcon from '@mui/icons-material/Delete'
import { studioApi } from '@/shared/api/client'
import { LoadingError, PageHeader } from '@/shared/components'

export function FlowsPage() {
  const { projectId = '' } = useParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [createOpen, setCreateOpen] = useState(false)
  const [flowId, setFlowId] = useState('')
  const [flowName, setFlowName] = useState('')

  const flowsQuery = useQuery({
    queryKey: ['flows', projectId],
    queryFn: () => studioApi.listFlows(projectId),
    enabled: !!projectId,
  })

  const createMutation = useMutation({
    mutationFn: () => studioApi.createFlow(projectId, { id: flowId, name: flowName || undefined }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['flows', projectId] })
      setCreateOpen(false)
      navigate(`/projects/${projectId}/studio/flows/${flowId}`)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => studioApi.deleteFlow(projectId, id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['flows', projectId] }),
  })

  return (
    <Box>
      <PageHeader
        title="Flows"
        subtitle="Conversation flow definitions"
        actions={
          <Button variant="contained" startIcon={<AddIcon />} onClick={() => setCreateOpen(true)}>
            New Flow
          </Button>
        }
      />
      <LoadingError loading={flowsQuery.isLoading} error={flowsQuery.error}>
        <List>
          {(flowsQuery.data ?? []).map((id) => (
            <ListItem
              key={id}
              secondaryAction={
                <IconButton edge="end" onClick={() => deleteMutation.mutate(id)}>
                  <DeleteIcon />
                </IconButton>
              }
              disablePadding
            >
              <ListItemButton onClick={() => navigate(`/projects/${projectId}/studio/flows/${id}`)}>
                <ListItemText primary={id} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </LoadingError>

      <Dialog open={createOpen} onClose={() => setCreateOpen(false)}>
        <DialogTitle>Create Flow</DialogTitle>
        <DialogContent>
          <TextField label="ID" value={flowId} onChange={(e) => setFlowId(e.target.value)} fullWidth sx={{ mt: 1 }} />
          <TextField label="Name" value={flowName} onChange={(e) => setFlowName(e.target.value)} fullWidth sx={{ mt: 2 }} />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={() => createMutation.mutate()} disabled={!flowId}>
            Create
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
