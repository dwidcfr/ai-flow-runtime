import { useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm, Controller } from 'react-hook-form'
import { z } from 'zod'
import { Alert, Box, Button, Chip, Stack, TextField } from '@mui/material'
import { studioApi } from '@/shared/api/client'
import { PageHeader } from '@/shared/components'

const schema = z.object({
  id: z.string().min(1),
  name: z.string().min(1),
  description: z.string().optional(),
  prompt_set: z.string().min(1),
  knowledge_source: z.string().min(1),
  flows: z.array(z.string()),
})

type FormValues = z.infer<typeof schema>

export function CompanyEditor() {
  const { projectId = '' } = useParams()
  const queryClient = useQueryClient()

  const companyQuery = useQuery({
    queryKey: ['company', projectId],
    queryFn: () => studioApi.getCompany(projectId),
    enabled: !!projectId,
  })

  const { control, handleSubmit, reset, formState: { isDirty } } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      id: '',
      name: '',
      description: '',
      prompt_set: '',
      knowledge_source: '',
      flows: [],
    },
  })

  useEffect(() => {
    if (companyQuery.data) {
      reset({
        id: companyQuery.data.id,
        name: companyQuery.data.name,
        description: companyQuery.data.description ?? '',
        prompt_set: companyQuery.data.prompt_set,
        knowledge_source: companyQuery.data.knowledge_source,
        flows: companyQuery.data.flows,
      })
    }
  }, [companyQuery.data, reset])

  const saveMutation = useMutation({
    mutationFn: (values: FormValues) =>
      studioApi.updateCompany(projectId, {
        ...values,
        description: values.description || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['company', projectId] })
      queryClient.invalidateQueries({ queryKey: ['project', projectId] })
    },
  })

  const onSubmit = handleSubmit((values) => saveMutation.mutate(values))

  return (
    <Box>
      <PageHeader
        title="Company"
        subtitle="Project registry configuration"
        actions={
          <Button variant="contained" onClick={onSubmit} disabled={!isDirty || saveMutation.isPending}>
            Save
          </Button>
        }
      />
      {saveMutation.isSuccess && <Alert severity="success" sx={{ mb: 2 }}>Saved successfully</Alert>}
      {saveMutation.isError && <Alert severity="error" sx={{ mb: 2 }}>Failed to save</Alert>}
      <Stack spacing={2} component="form" onSubmit={onSubmit} maxWidth={640}>
        <Controller name="id" control={control} render={({ field }) => (
          <TextField {...field} label="ID" disabled fullWidth />
        )} />
        <Controller name="name" control={control} render={({ field }) => (
          <TextField {...field} label="Name" fullWidth />
        )} />
        <Controller name="description" control={control} render={({ field }) => (
          <TextField {...field} label="Description" fullWidth multiline rows={2} />
        )} />
        <Controller name="prompt_set" control={control} render={({ field }) => (
          <TextField {...field} label="Prompt Set" fullWidth />
        )} />
        <Controller name="knowledge_source" control={control} render={({ field }) => (
          <TextField {...field} label="Knowledge Source" fullWidth />
        )} />
        <Controller name="flows" control={control} render={({ field }) => (
          <Box>
            <TextField
              label="Flows (comma-separated)"
              fullWidth
              value={field.value.join(', ')}
              onChange={(e) => field.onChange(e.target.value.split(',').map((s) => s.trim()).filter(Boolean))}
            />
            <Stack direction="row" spacing={1} sx={{ mt: 1 }} flexWrap="wrap" useFlexGap>
              {field.value.map((f) => <Chip key={f} label={f} size="small" />)}
            </Stack>
          </Box>
        )} />
      </Stack>
    </Box>
  )
}
