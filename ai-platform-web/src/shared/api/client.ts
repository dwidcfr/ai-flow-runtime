import axios, { type AxiosInstance } from 'axios'
import type {
  ClientTemplateResponse,
  CreateFlowRequest,
  CreateProjectRequest,
  CreateSessionRequest,
  CreateSessionResponse,
  DraftCompanyResponse,
  DraftFlowDto,
  DraftKnowledgeResponse,
  DraftPromptResponse,
  EvalAsyncRunResponse,
  EvalRunRequest,
  EvalRunResponse,
  EvalRunStatusResponse,
  EvalScenariosResponse,
  FlowInspectResponse,
  FlowValidationResponse,
  HealthResponse,
  HistoryResponse,
  KnowledgeReindexResponse,
  KnowledgeSearchRequest,
  KnowledgeSearchResponse,
  MessageRequest,
  MessageResponse,
  OverviewResponse,
  PlatformConfigResponse,
  PlatformStatusResponse,
  ProjectAssetsResponse,
  ProjectStatusResponse,
  PromptPreviewRequest,
  PromptPreviewResponse,
  PublishResponse,
  SessionInspectResponse,
  SessionResponse,
  SessionsListResponse,
  StudioProjectDetailResponse,
  StudioProjectsResponse,
  UpdateCompanyRequest,
  UpdateProjectRequest,
  UpdatePromptRequest,
  UpsertClientRequest,
  UpsertDocumentRequest,
  VersionDetailResponse,
  VersionsResponse,
} from '@/shared/types'

const DEFAULT_BASE_URL = 'http://127.0.0.1:8080'

let apiInstance: AxiosInstance | null = null
let currentBaseUrl = DEFAULT_BASE_URL

export function getApiBaseUrl(): string {
  return currentBaseUrl
}

export function setApiBaseUrl(url: string): void {
  currentBaseUrl = url.replace(/\/$/, '')
  if (apiInstance) {
    apiInstance.defaults.baseURL = currentBaseUrl
  }
}

function getClient(): AxiosInstance {
  if (!apiInstance) {
    apiInstance = axios.create({
      baseURL: currentBaseUrl,
      headers: { 'Content-Type': 'application/json' },
      timeout: 300_000,
    })
  }
  return apiInstance
}

export const studioApi = {
  listProjects: () =>
    getClient().get<StudioProjectsResponse>('/studio/projects').then((r) => r.data),

  getProject: (projectId: string) =>
    getClient()
      .get<StudioProjectDetailResponse>(`/studio/projects/${projectId}`)
      .then((r) => r.data),

  createProject: (body: CreateProjectRequest) =>
    getClient()
      .post<StudioProjectDetailResponse>('/studio/projects', body)
      .then((r) => r.data),

  updateProject: (projectId: string, body: UpdateProjectRequest) =>
    getClient()
      .patch<StudioProjectDetailResponse>(`/studio/projects/${projectId}`, body)
      .then((r) => r.data),

  deleteProject: (projectId: string) =>
    getClient().delete(`/studio/projects/${projectId}`),

  getProjectStatus: (projectId: string) =>
    getClient()
      .get<ProjectStatusResponse>(`/studio/projects/${projectId}/status`)
      .then((r) => r.data),

  getProjectAssets: (projectId: string) =>
    getClient()
      .get<ProjectAssetsResponse>(`/studio/projects/${projectId}/assets`)
      .then((r) => r.data),

  getCompany: (projectId: string) =>
    getClient()
      .get<DraftCompanyResponse>(`/studio/projects/${projectId}/company`)
      .then((r) => r.data),

  updateCompany: (projectId: string, body: UpdateCompanyRequest) =>
    getClient()
      .put<DraftCompanyResponse>(`/studio/projects/${projectId}/company`, body)
      .then((r) => r.data),

  getKnowledge: (projectId: string) =>
    getClient()
      .get<DraftKnowledgeResponse>(`/studio/projects/${projectId}/knowledge`)
      .then((r) => r.data),

  putKnowledge: (projectId: string, body: DraftKnowledgeResponse) =>
    getClient()
      .put<DraftKnowledgeResponse>(`/studio/projects/${projectId}/knowledge`, body)
      .then((r) => r.data),

  upsertDocument: (projectId: string, body: UpsertDocumentRequest) =>
    getClient()
      .post<DraftKnowledgeResponse>(`/studio/projects/${projectId}/knowledge/documents`, body)
      .then((r) => r.data),

  deleteDocument: (projectId: string, docId: string) =>
    getClient()
      .delete<DraftKnowledgeResponse>(`/studio/projects/${projectId}/knowledge/documents/${docId}`)
      .then((r) => r.data),

  reindexKnowledge: (projectId: string) =>
    getClient()
      .post<KnowledgeReindexResponse>(`/studio/projects/${projectId}/knowledge/reindex`)
      .then((r) => r.data),

  searchKnowledge: (projectId: string, body: KnowledgeSearchRequest) =>
    getClient()
      .post<KnowledgeSearchResponse>(`/studio/projects/${projectId}/knowledge/search`, body)
      .then((r) => r.data),

  listFlows: (projectId: string) =>
    getClient()
      .get<{ flows: string[] }>(`/studio/projects/${projectId}/flows`)
      .then((r) => r.data.flows),

  getFlow: (projectId: string, flowId: string) =>
    getClient()
      .get<DraftFlowDto>(`/studio/projects/${projectId}/flows/${flowId}`)
      .then((r) => r.data),

  createFlow: (projectId: string, body: CreateFlowRequest) =>
    getClient()
      .post<DraftFlowDto>(`/studio/projects/${projectId}/flows`, body)
      .then((r) => r.data),

  putFlow: (projectId: string, flow: DraftFlowDto) =>
    getClient()
      .put<DraftFlowDto>(`/studio/projects/${projectId}/flows/${flow.id}`, flow)
      .then((r) => r.data),

  deleteFlow: (projectId: string, flowId: string) =>
    getClient().delete(`/studio/projects/${projectId}/flows/${flowId}`),

  validateFlow: (projectId: string, flow: DraftFlowDto) =>
    getClient()
      .post<FlowValidationResponse>(`/studio/projects/${projectId}/flows/validate`, flow)
      .then((r) => r.data),

  getPrompt: (projectId: string, setId: string) =>
    getClient()
      .get<DraftPromptResponse>(`/studio/projects/${projectId}/prompts/${setId}`)
      .then((r) => r.data),

  putPrompt: (projectId: string, body: UpdatePromptRequest) =>
    getClient()
      .put<DraftPromptResponse>(`/studio/projects/${projectId}/prompts/${body.id}`, body)
      .then((r) => r.data),

  previewPrompt: (projectId: string, setId: string, body: PromptPreviewRequest) =>
    getClient()
      .post<PromptPreviewResponse>(`/studio/projects/${projectId}/prompts/${setId}/preview`, body)
      .then((r) => r.data),

  listClients: (projectId: string) =>
    getClient()
      .get<{ clients: string[] }>(`/studio/projects/${projectId}/clients`)
      .then((r) => r.data.clients),

  getClient: (projectId: string, name: string) =>
    getClient()
      .get<ClientTemplateResponse>(`/studio/projects/${projectId}/clients/${name}`)
      .then((r) => r.data),

  upsertClient: (projectId: string, body: UpsertClientRequest) =>
    getClient()
      .post<ClientTemplateResponse>(`/studio/projects/${projectId}/clients`, body)
      .then((r) => r.data),

  upsertClientNamed: (projectId: string, name: string, body: UpsertClientRequest) =>
    getClient()
      .put<ClientTemplateResponse>(`/studio/projects/${projectId}/clients/${name}`, body)
      .then((r) => r.data),

  deleteClient: (projectId: string, name: string) =>
    getClient().delete(`/studio/projects/${projectId}/clients/${name}`),

  publishProject: (projectId: string) =>
    getClient()
      .post<PublishResponse>(`/studio/projects/${projectId}/publish`)
      .then((r) => r.data),

  listVersions: (projectId: string) =>
    getClient()
      .get<VersionsResponse>(`/studio/projects/${projectId}/versions`)
      .then((r) => r.data.versions),

  getVersion: (projectId: string, version: number) =>
    getClient()
      .get<VersionDetailResponse>(`/studio/projects/${projectId}/versions/${version}`)
      .then((r) => r.data),
}

export const runtimeApi = {
  createSession: (body: CreateSessionRequest) =>
    getClient().post<CreateSessionResponse>('/sessions', body).then((r) => r.data),

  listSessions: () =>
    getClient().get<SessionsListResponse>('/sessions').then((r) => r.data.sessions),

  getSession: (sessionId: string) =>
    getClient().get<SessionResponse>(`/sessions/${sessionId}`).then((r) => r.data),

  deleteSession: (sessionId: string) =>
    getClient().delete(`/sessions/${sessionId}`),

  sendMessage: (sessionId: string, body: MessageRequest, debug = false) =>
    getClient()
      .post<MessageResponse>(`/sessions/${sessionId}/messages`, body, {
        params: debug ? { debug: true } : undefined,
      })
      .then((r) => r.data),

  getHistory: (sessionId: string) =>
    getClient().get<HistoryResponse>(`/sessions/${sessionId}/history`).then((r) => r.data),

  inspectSession: (sessionId: string) =>
    getClient()
      .get<SessionInspectResponse>(`/sessions/${sessionId}/inspect`)
      .then((r) => r.data),

  inspectFlow: (sessionId: string) =>
    getClient()
      .get<FlowInspectResponse>(`/sessions/${sessionId}/flow`)
      .then((r) => r.data),
}

export const adminApi = {
  health: () => getClient().get<HealthResponse>('/health').then((r) => r.data),

  overview: () => getClient().get<OverviewResponse>('/overview').then((r) => r.data),

  platformStatus: () =>
    getClient().get<PlatformStatusResponse>('/platform/status').then((r) => r.data),

  platformConfig: () =>
    getClient().get<PlatformConfigResponse>('/platform/config').then((r) => r.data),

  listEvalScenarios: () =>
    getClient().get<EvalScenariosResponse>('/evals/scenarios').then((r) => r.data.scenarios),

  runEvalSync: (body: EvalRunRequest = {}) =>
    getClient().post<EvalRunResponse>('/evals/runs', body).then((r) => r.data),

  runEvalAsync: (body: EvalRunRequest = {}) =>
    getClient().post<EvalAsyncRunResponse>('/evals/runs/async', body).then((r) => r.data),

  getEvalRun: (runId: string) =>
    getClient().get<EvalRunStatusResponse>(`/evals/runs/${runId}`).then((r) => r.data),

  reloadFlows: () => getClient().post('/flows/reload').then((r) => r.data),

  reloadPrompts: () => getClient().post('/prompt-sets/reload').then((r) => r.data),
}
