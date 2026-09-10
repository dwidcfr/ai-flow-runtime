import { useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import SearchIcon from '@mui/icons-material/Search'
import {
  Dialog,
  DialogContent,
  InputAdornment,
  List,
  ListItemButton,
  ListItemText,
  TextField,
  Typography,
} from '@mui/material'
import { useLayoutStore } from '@/shared/stores/layoutStore'

export interface CommandItem {
  id: string
  label: string
  path: string
  group: string
}

const STATIC_COMMANDS: CommandItem[] = [
  { id: 'projects', label: 'Projects', path: '/', group: 'Navigation' },
  { id: 'playground', label: 'Playground', path: '/playground', group: 'Navigation' },
  { id: 'evaluation', label: 'Evaluation', path: '/evaluation', group: 'Navigation' },
  { id: 'monitoring', label: 'Monitoring', path: '/monitoring', group: 'Navigation' },
  { id: 'settings', label: 'Settings', path: '/settings', group: 'Navigation' },
]

interface CommandPaletteProps {
  projectCommands?: CommandItem[]
}

export function CommandPalette({ projectCommands = [] }: CommandPaletteProps) {
  const open = useLayoutStore((s) => s.commandPaletteOpen)
  const setOpen = useLayoutStore((s) => s.setCommandPaletteOpen)
  const navigate = useNavigate()
  const [query, setQuery] = useState('')

  const commands = useMemo(
    () => [...STATIC_COMMANDS, ...projectCommands],
    [projectCommands],
  )

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return commands
    return commands.filter(
      (c) => c.label.toLowerCase().includes(q) || c.group.toLowerCase().includes(q),
    )
  }, [commands, query])

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault()
        const current = useLayoutStore.getState().commandPaletteOpen
        setOpen(!current)
      }
      if (e.key === 'Escape') setOpen(false)
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [setOpen])

  const handleSelect = (path: string) => {
    navigate(path)
    setOpen(false)
    setQuery('')
  }

  return (
    <Dialog open={open} onClose={() => setOpen(false)} maxWidth="sm" fullWidth>
      <DialogContent sx={{ p: 2 }}>
        <TextField
          autoFocus
          fullWidth
          placeholder="Search pages… (Ctrl+K)"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon />
                </InputAdornment>
              ),
            },
          }}
        />
        <List dense sx={{ mt: 1, maxHeight: 360, overflow: 'auto' }}>
          {filtered.length === 0 && (
            <Typography variant="body2" color="text.secondary" sx={{ py: 2, textAlign: 'center' }}>
              No results
            </Typography>
          )}
          {filtered.map((cmd) => (
            <ListItemButton key={cmd.id} onClick={() => handleSelect(cmd.path)}>
              <ListItemText primary={cmd.label} secondary={cmd.group} />
            </ListItemButton>
          ))}
        </List>
      </DialogContent>
    </Dialog>
  )
}
