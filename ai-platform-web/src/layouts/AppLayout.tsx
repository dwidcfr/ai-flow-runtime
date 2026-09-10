import { useMemo } from 'react'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import DashboardIcon from '@mui/icons-material/Dashboard'
import ScienceIcon from '@mui/icons-material/Science'
import SettingsIcon from '@mui/icons-material/Settings'
import SmartToyIcon from '@mui/icons-material/SmartToy'
import MonitorHeartIcon from '@mui/icons-material/MonitorHeart'
import MenuIcon from '@mui/icons-material/Menu'
import {
  Box,
  Drawer,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Toolbar,
  Typography,
  useMediaQuery,
  useTheme,
} from '@mui/material'
import { adminApi } from '@/shared/api/client'
import { CommandPalette } from '@/shared/components'
import { StatusChip } from '@/shared/components/StatusChip'
import { useLayoutStore } from '@/shared/stores/layoutStore'

const DRAWER_WIDTH = 260

const NAV_ITEMS = [
  { label: 'Projects', path: '/', icon: <DashboardIcon /> },
  { label: 'Playground', path: '/playground', icon: <SmartToyIcon /> },
  { label: 'Evaluation', path: '/evaluation', icon: <ScienceIcon /> },
  { label: 'Monitoring', path: '/monitoring', icon: <MonitorHeartIcon /> },
  { label: 'Settings', path: '/settings', icon: <SettingsIcon /> },
]

export function AppLayout() {
  const theme = useTheme()
  const isMobile = useMediaQuery(theme.breakpoints.down('md'))
  const collapsed = useLayoutStore((s) => s.navCollapsed)
  const toggleNav = useLayoutStore((s) => s.toggleNav)
  const navigate = useNavigate()
  const location = useLocation()

  const healthQuery = useQuery({
    queryKey: ['health'],
    queryFn: adminApi.health,
    refetchInterval: 30_000,
  })

  const drawerWidth = collapsed && !isMobile ? 72 : DRAWER_WIDTH

  const drawer = useMemo(
    () => (
      <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        <Toolbar sx={{ px: 2, gap: 1 }}>
          {!collapsed && (
            <Typography variant="h6" noWrap fontWeight={700}>
              AI Platform
            </Typography>
          )}
        </Toolbar>
        <List sx={{ flex: 1, px: 1 }}>
          {NAV_ITEMS.map((item) => {
            const selected =
              item.path === '/'
                ? location.pathname === '/'
                : location.pathname.startsWith(item.path)
            return (
              <ListItemButton
                key={item.path}
                selected={selected}
                onClick={() => navigate(item.path)}
                sx={{ borderRadius: 2, mb: 0.5 }}
              >
                <ListItemIcon sx={{ minWidth: 40 }}>{item.icon}</ListItemIcon>
                {!collapsed && <ListItemText primary={item.label} />}
              </ListItemButton>
            )
          })}
        </List>
        <Box sx={{ p: 2 }}>
          <StatusChip
            status={healthQuery.data?.status ?? (healthQuery.isError ? 'error' : 'loading')}
            label={`API: ${healthQuery.data?.status ?? '…'}`}
          />
        </Box>
      </Box>
    ),
    [collapsed, healthQuery.data?.status, healthQuery.isError, location.pathname, navigate],
  )

  return (
    <Box sx={{ display: 'flex', minHeight: '100vh' }}>
      <Drawer
        variant={isMobile ? 'temporary' : 'permanent'}
        open={isMobile ? !collapsed : true}
        onClose={toggleNav}
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': { width: drawerWidth, boxSizing: 'border-box' },
        }}
      >
        {drawer}
      </Drawer>
      <Box component="main" sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        <Toolbar sx={{ borderBottom: 1, borderColor: 'divider', gap: 1 }}>
          <IconButton onClick={toggleNav} edge="start">
            <MenuIcon />
          </IconButton>
          <Typography
            variant="body2"
            color="text.secondary"
            sx={{ flex: 1, cursor: 'pointer' }}
            onClick={() => useLayoutStore.getState().setCommandPaletteOpen(true)}
          >
            Ctrl+K to navigate
          </Typography>
        </Toolbar>
        <Box sx={{ flex: 1, p: { xs: 2, md: 3 }, overflow: 'auto' }}>
          <Outlet />
        </Box>
      </Box>
      <CommandPalette />
    </Box>
  )
}
