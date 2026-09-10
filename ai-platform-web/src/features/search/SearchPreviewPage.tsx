import { useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import SearchIcon from '@mui/icons-material/Search'
import {
  Box,
  Button,
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
import { PageHeader } from '@/shared/components'
import type { SearchCandidateDto } from '@/shared/types'

export function SearchPreviewPage() {
  const { projectId = '' } = useParams()
  const [query, setQuery] = useState('')
  const [topK, setTopK] = useState(5)
  const [results, setResults] = useState<SearchCandidateDto[]>([])

  const searchMutation = useMutation({
    mutationFn: () => studioApi.searchKnowledge(projectId, { query, top_k: topK }),
    onSuccess: (data) => setResults(data.results),
  })

  return (
    <Box>
      <PageHeader title="Search Preview" subtitle="Test knowledge retrieval" />
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} sx={{ mb: 3 }}>
        <TextField
          label="Query"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          fullWidth
          onKeyDown={(e) => e.key === 'Enter' && searchMutation.mutate()}
        />
        <TextField
          label="Top K"
          type="number"
          value={topK}
          onChange={(e) => setTopK(Number(e.target.value))}
          sx={{ width: 120 }}
        />
        <Button
          variant="contained"
          startIcon={<SearchIcon />}
          onClick={() => searchMutation.mutate()}
          disabled={!query || searchMutation.isPending}
        >
          Search
        </Button>
      </Stack>
      <TableContainer component={Paper} variant="outlined">
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell>Score</TableCell>
              <TableCell>Module</TableCell>
              <TableCell>Section</TableCell>
              <TableCell>Title</TableCell>
              <TableCell>Snippet</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {results.map((r, i) => (
              <TableRow key={`${r.module_id}-${i}`}>
                <TableCell>{r.score.toFixed(3)}</TableCell>
                <TableCell>{r.module_id}</TableCell>
                <TableCell>{r.section_id ?? '—'}</TableCell>
                <TableCell>{r.title}</TableCell>
                <TableCell sx={{ maxWidth: 400, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                  {r.snippet}
                </TableCell>
              </TableRow>
            ))}
            {results.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} align="center">
                  No results yet
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  )
}
