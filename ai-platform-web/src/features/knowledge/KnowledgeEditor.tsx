import { useMemo, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import Editor from '@monaco-editor/react'
import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import {
  Alert,
  Box,
  Button,
  IconButton,
  ListItemButton,
  ListItemText,
  Paper,
  Stack,
  TextField,
  Typography,
} from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { LoadingError, PageHeader } from '@/shared/components'
import { VirtualizedList } from '@/shared/components/VirtualizedList'
import type { KnowledgeDocumentDto } from '@/shared/types'

export function KnowledgeEditor() {
  const { projectId = '' } = useParams()
  const queryClient = useQueryClient()
  const [selectedDocId, setSelectedDocId] = useState<string | null>(null)
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null)
  const [editorContent, setEditorContent] = useState('')
  const [title, setTitle] = useState('')
  const [section, setSection] = useState('')

  const knowledgeQuery = useQuery({
    queryKey: ['knowledge', projectId],
    queryFn: () => studioApi.getKnowledge(projectId),
    enabled: !!projectId,
  })

  const filteredDocs = useMemo(() => {
    const docs = knowledgeQuery.data?.documents ?? []
    if (!selectedCategoryId) return docs
    return docs.filter((d) => d.category_id === selectedCategoryId)
  }, [knowledgeQuery.data?.documents, selectedCategoryId])

  const selectedDoc = useMemo(
    () => knowledgeQuery.data?.documents.find((d) => d.id === selectedDocId),
    [knowledgeQuery.data?.documents, selectedDocId],
  )

  const selectDoc = (doc: KnowledgeDocumentDto) => {
    setSelectedDocId(doc.id)
    setSelectedCategoryId(doc.category_id)
    setTitle(doc.title)
    setSection(doc.section)
    setEditorContent(doc.content)
  }

  const saveMutation = useMutation({
    mutationFn: () =>
      studioApi.upsertDocument(projectId, {
        id: selectedDocId ?? undefined,
        category_id: selectedCategoryId ?? knowledgeQuery.data?.categories[0]?.id ?? 'default',
        title,
        section,
        content: editorContent,
      }),
    onSuccess: (data) => {
      queryClient.setQueryData(['knowledge', projectId], data)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (docId: string) => studioApi.deleteDocument(projectId, docId),
    onSuccess: (data) => {
      queryClient.setQueryData(['knowledge', projectId], data)
      setSelectedDocId(null)
    },
  })

  const reindexMutation = useMutation({
    mutationFn: () => studioApi.reindexKnowledge(projectId),
  })

  const handleNewDoc = () => {
    const catId = selectedCategoryId ?? knowledgeQuery.data?.categories[0]?.id
    if (!catId) return
    setSelectedDocId(null)
    setSelectedCategoryId(catId)
    setTitle('New Document')
    setSection('')
    setEditorContent('# New Document\n')
  }

  return (
    <Box>
      <PageHeader
        title="Knowledge"
        subtitle="Categories and markdown documents"
        actions={
          <Stack direction="row" spacing={1}>
            <Button startIcon={<AddIcon />} onClick={handleNewDoc}>New Document</Button>
            <Button onClick={() => reindexMutation.mutate()} disabled={reindexMutation.isPending}>
              Reindex
            </Button>
            <Button variant="contained" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
              Save Document
            </Button>
          </Stack>
        }
      />
      {reindexMutation.data && (
        <Alert severity="info" sx={{ mb: 2 }}>
          Reindexed {reindexMutation.data.document_count} documents
        </Alert>
      )}
      <LoadingError loading={knowledgeQuery.isLoading} error={knowledgeQuery.error}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ height: 'calc(100vh - 280px)' }}>
          <Paper variant="outlined" sx={{ width: { md: 220 }, p: 1, flexShrink: 0 }}>
            <Typography variant="caption" color="text.secondary" sx={{ px: 1 }}>
              Categories
            </Typography>
            <ListItemButton
              selected={!selectedCategoryId}
              onClick={() => setSelectedCategoryId(null)}
            >
              <ListItemText primary="All" />
            </ListItemButton>
            {(knowledgeQuery.data?.categories ?? []).map((cat) => (
              <ListItemButton
                key={cat.id}
                selected={selectedCategoryId === cat.id}
                onClick={() => setSelectedCategoryId(cat.id)}
              >
                <ListItemText primary={cat.name} secondary={cat.id} />
              </ListItemButton>
            ))}
          </Paper>
          <Paper variant="outlined" sx={{ width: { md: 280 }, flexShrink: 0 }}>
            <VirtualizedList
              items={filteredDocs}
              height={400}
              renderRow={(doc) => (
                <ListItemButton
                  selected={selectedDocId === doc.id}
                  onClick={() => selectDoc(doc)}
                  sx={{ py: 0.5 }}
                >
                  <ListItemText primary={doc.title} secondary={doc.section} />
                  <IconButton
                    size="small"
                    onClick={(e) => { e.stopPropagation(); deleteMutation.mutate(doc.id) }}
                  >
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </ListItemButton>
              )}
            />
          </Paper>
          <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 1 }}>
            <Stack direction="row" spacing={1}>
              <TextField label="Title" value={title} onChange={(e) => setTitle(e.target.value)} fullWidth size="small" />
              <TextField label="Section" value={section} onChange={(e) => setSection(e.target.value)} fullWidth size="small" />
            </Stack>
            <Paper variant="outlined" sx={{ flex: 1, overflow: 'hidden' }}>
              <Editor
                height="100%"
                defaultLanguage="markdown"
                value={editorContent}
                onChange={(v) => setEditorContent(v ?? '')}
                theme="vs-dark"
                options={{ minimap: { enabled: false }, wordWrap: 'on' }}
              />
            </Paper>
            {selectedDoc && (
              <Typography variant="caption" color="text.secondary">
                Editing: {selectedDoc.id}
              </Typography>
            )}
          </Box>
        </Stack>
      </LoadingError>
    </Box>
  )
}
