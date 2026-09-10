import { useCallback, useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  Alert,
  Box,
  Button,
  Paper,
  Stack,
  TextField,
  Typography,
  List,
  ListItem,
  ListItemText,
} from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { LoadingError, PageHeader } from '@/shared/components'
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  addEdge,
  useNodesState,
  useEdgesState,
  nodeTypes,
  flowToReactFlow,
  reactFlowToDraft,
  autoLayout,
  type Connection,
  type FlowNode,
  type Edge,
} from './flowUtils'

export function FlowBuilderPage() {
  const { projectId = '', flowId = '' } = useParams()
  const queryClient = useQueryClient()
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
  const [validationIssues, setValidationIssues] = useState<string[]>([])

  const flowQuery = useQuery({
    queryKey: ['flow', projectId, flowId],
    queryFn: () => studioApi.getFlow(projectId, flowId),
    enabled: !!projectId && !!flowId,
  })

  const [nodes, setNodes, onNodesChange] = useNodesState<FlowNode>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([])

  useEffect(() => {
    if (flowQuery.data) {
      const { nodes: n, edges: e } = flowToReactFlow(flowQuery.data)
      setNodes(n)
      setEdges(e)
    }
  }, [flowQuery.data, setNodes, setEdges])

  const onConnect = useCallback(
    (connection: Connection) => setEdges((eds) => addEdge({ ...connection, label: 'next', animated: true }, eds)),
    [setEdges],
  )

  const selectedNode = nodes.find((n) => n.id === selectedNodeId)
  const flowNode = flowQuery.data?.nodes.find((n) => n.id === selectedNodeId)

  const saveMutation = useMutation({
    mutationFn: () => {
      if (!flowQuery.data) throw new Error('No flow')
      const draft = reactFlowToDraft(flowQuery.data, nodes, edges)
      return studioApi.putFlow(projectId, draft)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['flow', projectId, flowId] })
    },
  })

  const validateMutation = useMutation({
    mutationFn: () => {
      if (!flowQuery.data) throw new Error('No flow')
      const draft = reactFlowToDraft(flowQuery.data, nodes, edges)
      return studioApi.validateFlow(projectId, draft)
    },
    onSuccess: (result) => {
      setValidationIssues(result.issues.map((i) => `${i.rule}: ${i.message}`))
    },
  })

  const handleAutoLayout = () => {
    setNodes((nds) => autoLayout(nds, edges))
  }

  const handleAddNode = () => {
    const id = `node_${nodes.length + 1}`
    setNodes((nds) => [
      ...nds,
      {
        id,
        type: 'flowNode',
        position: { x: 100, y: 100 + nodes.length * 80 },
        data: { label: id, nodeType: 'message' },
      },
    ])
  }

  const updateNodeType = (nodeType: string) => {
    if (!selectedNodeId || !flowQuery.data) return
    setNodes((nds) =>
      nds.map((n) =>
        n.id === selectedNodeId ? { ...n, data: { ...n.data, nodeType } } : n,
      ),
    )
  }

  return (
    <Box>
      <PageHeader
        title={`Flow: ${flowId}`}
        subtitle="Visual flow builder"
        actions={
          <Stack direction="row" spacing={1}>
            <Button onClick={handleAddNode}>Add Node</Button>
            <Button onClick={handleAutoLayout}>Auto Layout</Button>
            <Button onClick={() => validateMutation.mutate()} disabled={validateMutation.isPending}>
              Validate
            </Button>
            <Button variant="contained" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
              Save
            </Button>
          </Stack>
        }
      />
      {saveMutation.isSuccess && <Alert severity="success" sx={{ mb: 1 }}>Flow saved</Alert>}
      {validateMutation.data && (
        <Alert severity={validateMutation.data.valid ? 'success' : 'warning'} sx={{ mb: 1 }}>
          {validateMutation.data.valid ? 'Flow is valid' : `${validationIssues.length} issue(s)`}
        </Alert>
      )}
      <LoadingError loading={flowQuery.isLoading} error={flowQuery.error}>
        <Stack direction={{ xs: 'column', lg: 'row' }} spacing={2} sx={{ height: 'calc(100vh - 260px)' }}>
          <Paper variant="outlined" sx={{ flex: 1, minHeight: 400 }}>
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              onNodeClick={(_, node) => setSelectedNodeId(node.id)}
              nodeTypes={nodeTypes}
              fitView
            >
              <Background />
              <Controls />
              <MiniMap />
            </ReactFlow>
          </Paper>
          <Paper variant="outlined" sx={{ width: { lg: 300 }, p: 2 }}>
            <Typography variant="subtitle2" gutterBottom>Node Inspector</Typography>
            {selectedNode ? (
              <Stack spacing={2}>
                <TextField label="ID" value={selectedNode.id} disabled fullWidth size="small" />
                <TextField
                  label="Node Type"
                  value={String(selectedNode.data.nodeType ?? '')}
                  onChange={(e) => updateNodeType(e.target.value)}
                  fullWidth
                  size="small"
                />
                {flowNode && (
                  <TextField
                    label="Payload (JSON)"
                    value={JSON.stringify(flowNode.payload, null, 2)}
                    multiline
                    rows={6}
                    fullWidth
                    size="small"
                    InputProps={{ readOnly: true }}
                  />
                )}
              </Stack>
            ) : (
              <Typography variant="body2" color="text.secondary">
                Select a node to inspect
              </Typography>
            )}
            {validationIssues.length > 0 && (
              <List dense sx={{ mt: 2 }}>
                {validationIssues.map((issue, i) => (
                  <ListItem key={i} disablePadding>
                    <ListItemText primary={issue} />
                  </ListItem>
                ))}
              </List>
            )}
          </Paper>
        </Stack>
      </LoadingError>
    </Box>
  )
}
