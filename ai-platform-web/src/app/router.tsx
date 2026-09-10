import { lazy, Suspense } from 'react'
import { Navigate, Route, Routes } from 'react-router-dom'
import { CircularProgress, Box } from '@mui/material'
import { AppLayout } from '@/layouts/AppLayout'
import { ProjectLayout } from '@/layouts/ProjectLayout'

const ProjectsPage = lazy(() =>
  import('@/features/projects/ProjectsPage').then((m) => ({ default: m.ProjectsPage })),
)
const CompanyEditor = lazy(() =>
  import('@/features/company/CompanyEditor').then((m) => ({ default: m.CompanyEditor })),
)
const KnowledgeEditor = lazy(() =>
  import('@/features/knowledge/KnowledgeEditor').then((m) => ({ default: m.KnowledgeEditor })),
)
const SearchPreviewPage = lazy(() =>
  import('@/features/search/SearchPreviewPage').then((m) => ({ default: m.SearchPreviewPage })),
)
const FlowsPage = lazy(() =>
  import('@/features/flows/FlowsPage').then((m) => ({ default: m.FlowsPage })),
)
const FlowBuilderPage = lazy(() =>
  import('@/features/flows/FlowBuilderPage').then((m) => ({ default: m.FlowBuilderPage })),
)
const PromptsPage = lazy(() =>
  import('@/features/prompts/PromptsPage').then((m) => ({ default: m.PromptsPage })),
)
const ClientsPage = lazy(() =>
  import('@/features/clients/ClientsPage').then((m) => ({ default: m.ClientsPage })),
)
const PublishPage = lazy(() =>
  import('@/features/publish/PublishPage').then((m) => ({ default: m.PublishPage })),
)
const AssetsPage = lazy(() =>
  import('@/features/assets/AssetsPage').then((m) => ({ default: m.AssetsPage })),
)
const PlaygroundPage = lazy(() =>
  import('@/features/playground/PlaygroundPage').then((m) => ({ default: m.PlaygroundPage })),
)
const EvaluationPage = lazy(() =>
  import('@/features/evaluation/EvaluationPage').then((m) => ({ default: m.EvaluationPage })),
)
const MonitoringPage = lazy(() =>
  import('@/features/monitoring/MonitoringPage').then((m) => ({ default: m.MonitoringPage })),
)
const SettingsPage = lazy(() =>
  import('@/features/settings/SettingsPage').then((m) => ({ default: m.SettingsPage })),
)

function PageLoader() {
  return (
    <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}>
      <CircularProgress />
    </Box>
  )
}

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<PageLoader />}>{children}</Suspense>
}

export function AppRouter() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route
          index
          element={
            <Lazy>
              <ProjectsPage />
            </Lazy>
          }
        />
        <Route
          path="playground"
          element={
            <Lazy>
              <PlaygroundPage />
            </Lazy>
          }
        />
        <Route
          path="evaluation"
          element={
            <Lazy>
              <EvaluationPage />
            </Lazy>
          }
        />
        <Route
          path="monitoring"
          element={
            <Lazy>
              <MonitoringPage />
            </Lazy>
          }
        />
        <Route
          path="settings"
          element={
            <Lazy>
              <SettingsPage />
            </Lazy>
          }
        />
        <Route path="projects/:projectId" element={<ProjectLayout />}>
          <Route
            path="studio/company"
            element={
              <Lazy>
                <CompanyEditor />
              </Lazy>
            }
          />
          <Route
            path="studio/knowledge"
            element={
              <Lazy>
                <KnowledgeEditor />
              </Lazy>
            }
          />
          <Route
            path="studio/search"
            element={
              <Lazy>
                <SearchPreviewPage />
              </Lazy>
            }
          />
          <Route
            path="studio/flows"
            element={
              <Lazy>
                <FlowsPage />
              </Lazy>
            }
          />
          <Route
            path="studio/flows/:flowId"
            element={
              <Lazy>
                <FlowBuilderPage />
              </Lazy>
            }
          />
          <Route
            path="studio/prompts"
            element={
              <Lazy>
                <PromptsPage />
              </Lazy>
            }
          />
          <Route
            path="studio/clients"
            element={
              <Lazy>
                <ClientsPage />
              </Lazy>
            }
          />
          <Route
            path="publish"
            element={
              <Lazy>
                <PublishPage />
              </Lazy>
            }
          />
          <Route
            path="assets"
            element={
              <Lazy>
                <AssetsPage />
              </Lazy>
            }
          />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  )
}
