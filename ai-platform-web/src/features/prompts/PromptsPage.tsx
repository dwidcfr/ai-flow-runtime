import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import Editor from '@monaco-editor/react'
import {
  Alert,
  Box,
  Button,
  Paper,
  Stack,
  Tab,
  Tabs,
  TextField,
  Typography,
} from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { CopyBlock, LoadingError, PageHeader } from '@/shared/components'
import type { DraftPromptResponse, PromptTemplateDto } from '@/shared/types'

type TabKey = 'identity' | 'router' | 'response'

function TemplateFields({
  template,
  onChange,
}: {
  template: PromptTemplateDto
  onChange: (t: PromptTemplateDto) => void
}) {
  return (
    <Stack spacing={2}>
      {(['system', 'developer', 'assembly'] as const).map((key) => (
        <TextField
          key={key}
          label={key}
          value={template[key]}
          onChange={(e) => onChange({ ...template, [key]: e.target.value })}
          multiline
          rows={4}
          fullWidth
        />
      ))}
    </Stack>
  )
}

export function PromptsPage() {
  const { projectId = '' } = useParams()
  const queryClient = useQueryClient()
  const [tab, setTab] = useState<TabKey>('identity')
  const [prompt, setPrompt] = useState<DraftPromptResponse | null>(null)
  const [preview, setPreview] = useState('')

  const projectQuery = useQuery({
    queryKey: ['project', projectId],
    queryFn: () => studioApi.getProject(projectId),
    enabled: !!projectId,
  })

  const setId = projectQuery.data?.prompt_set ?? ''

  const promptQuery = useQuery({
    queryKey: ['prompt', projectId, setId],
    queryFn: () => studioApi.getPrompt(projectId, setId),
    enabled: !!projectId && !!setId,
  })

  useEffect(() => {
    if (promptQuery.data) setPrompt(promptQuery.data)
  }, [promptQuery.data])

  const saveMutation = useMutation({
    mutationFn: () => {
      if (!prompt) throw new Error('No prompt')
      return studioApi.putPrompt(projectId, prompt)
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['prompt', projectId, setId] }),
  })

  const previewMutation = useMutation({
    mutationFn: (template: string) =>
      studioApi.previewPrompt(projectId, setId, { template }),
    onSuccess: (data) => setPreview(data.rendered),
  })

  const handleReload = () => promptQuery.refetch()

  if (!prompt) {
    return <LoadingError loading={promptQuery.isLoading} error={promptQuery.error} />
  }

  const activeTemplate = tab === 'router' ? prompt.router : tab === 'response' ? prompt.response : null

  return (
    <Box>
      <PageHeader
        title="Prompts"
        subtitle={`Set: ${setId}`}
        actions={
          <Stack direction="row" spacing={1}>
            <Button onClick={handleReload}>Reload</Button>
            {activeTemplate && (
              <Button onClick={() => previewMutation.mutate(activeTemplate.assembly)}>
                Preview
              </Button>
            )}
            <Button variant="contained" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
              Save
            </Button>
          </Stack>
        }
      />
      {saveMutation.isSuccess && <Alert severity="success" sx={{ mb: 2 }}>Saved</Alert>}
      <Tabs value={tab} onChange={(_, v) => setTab(v)} sx={{ mb: 2 }}>
        <Tab label="Identity" value="identity" />
        <Tab label="Router" value="router" />
        <Tab label="Response" value="response" />
      </Tabs>
      {tab === 'identity' && (
        <Paper variant="outlined" sx={{ p: 0, overflow: 'hidden', height: 400 }}>
          <Editor
            height="400px"
            defaultLanguage="json"
            value={JSON.stringify(prompt.identity, null, 2)}
            onChange={(v) => {
              try {
                const identity = JSON.parse(v ?? '{}') as Record<string, unknown>
                setPrompt({ ...prompt, identity })
              } catch { /* ignore parse errors while typing */ }
            }}
            theme="vs-dark"
          />
        </Paper>
      )}
      {tab === 'router' && (
        <TemplateFields
          template={prompt.router}
          onChange={(router) => setPrompt({ ...prompt, router })}
        />
      )}
      {tab === 'response' && (
        <TemplateFields
          template={prompt.response}
          onChange={(response) => setPrompt({ ...prompt, response })}
        />
      )}
      {preview && (
        <Box sx={{ mt: 3 }}>
          <Typography variant="subtitle2" gutterBottom>Preview</Typography>
          <CopyBlock content={preview} />
        </Box>
      )}
    </Box>
  )
}
