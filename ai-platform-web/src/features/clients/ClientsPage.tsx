import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import Editor from '@monaco-editor/react'
import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  Paper,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TextField,
} from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { LoadingError, PageHeader } from '@/shared/components'

export function ClientsPage() {
  const { projectId = '' } = useParams()
  const queryClient = useQueryClient()
  const [selected, setSelected] = useState<string | null>(null)
  const [json, setJson] = useState('{}')
  const [createOpen, setCreateOpen] = useState(false)
  const [newName, setNewName] = useState('')

  const clientsQuery = useQuery({
    queryKey: ['clients', projectId],
    queryFn: () => studioApi.listClients(projectId),
    enabled: !!projectId,
  })

  const clientQuery = useQuery({
    queryKey: ['client', projectId, selected],
    queryFn: () => studioApi.getClient(projectId, selected!),
    enabled: !!projectId && !!selected,
  })

  useEffect(() => {
    if (clientQuery.data) {
      setJson(JSON.stringify(clientQuery.data.data, null, 2))
    }
  }, [clientQuery.data])

  const saveMutation = useMutation({
    mutationFn: () => {
      const data = JSON.parse(json) as Record<string, unknown>
      return studioApi.upsertClientNamed(projectId, selected!, { name: selected!, data })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['clients', projectId] })
      queryClient.invalidateQueries({ queryKey: ['client', projectId, selected] })
    },
  })

  const createMutation = useMutation({
    mutationFn: () =>
      studioApi.upsertClient(projectId, { name: newName, data: {} }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['clients', projectId] })
      setSelected(newName)
      setCreateOpen(false)
      setNewName('')
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (name: string) => studioApi.deleteClient(projectId, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['clients', projectId] })
      setSelected(null)
    },
  })

  return (
    <Box>
      <PageHeader
        title="Clients"
        subtitle="Client template JSON configurations"
        actions={
          <Stack direction="row" spacing={1}>
            <Button startIcon={<AddIcon />} onClick={() => setCreateOpen(true)}>New Client</Button>
            {selected && (
              <>
                <Button variant="contained" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
                  Save
                </Button>
              </>
            )}
          </Stack>
        }
      />
      <LoadingError loading={clientsQuery.isLoading} error={clientsQuery.error}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TableContainer component={Paper} variant="outlined" sx={{ maxWidth: 360 }}>
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Name</TableCell>
                  <TableCell width={48} />
                </TableRow>
              </TableHead>
              <TableBody>
                {(clientsQuery.data ?? []).map((name) => (
                  <TableRow
                    key={name}
                    hover
                    selected={selected === name}
                    onClick={() => setSelected(name)}
                    sx={{ cursor: 'pointer' }}
                  >
                    <TableCell>{name}</TableCell>
                    <TableCell>
                      <IconButton size="small" onClick={(e) => { e.stopPropagation(); deleteMutation.mutate(name) }}>
                        <DeleteIcon fontSize="small" />
                      </IconButton>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
          {selected && (
            <Paper variant="outlined" sx={{ flex: 1, overflow: 'hidden' }}>
              <Editor
                height="400px"
                defaultLanguage="json"
                value={json}
                onChange={(v) => setJson(v ?? '{}')}
                theme="vs-dark"
              />
            </Paper>
          )}
        </Stack>
      </LoadingError>

      <Dialog open={createOpen} onClose={() => setCreateOpen(false)}>
        <DialogTitle>New Client</DialogTitle>
        <DialogContent>
          <TextField label="Name" value={newName} onChange={(e) => setNewName(e.target.value)} fullWidth sx={{ mt: 1 }} />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={() => createMutation.mutate()} disabled={!newName}>Create</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
