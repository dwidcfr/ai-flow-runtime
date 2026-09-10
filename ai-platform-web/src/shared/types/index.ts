export type RuntimeState = 'idle' | 'flow' | 'paused' | 'module' | 'finished'

export interface HealthResponse {
  status: string
}

export interface GeminiStatusDto {
  enabled: boolean
  configured: boolean
  model_router: string
  model_response: string
}

export interface OverviewCountsDto {
  active_sessions: number
  flows: number
  prompt_sets: number
  modules: number
  search_index_documents: number
}

export interface OverviewResponse {
  api_version: string
  runtime_version: string
  uptime_secs: number
  gemini: GeminiStatusDto
  counts: OverviewCountsDto
}

export interface PlatformComponentStatus {
  id: string
  name: string
  status: string
  detail?: string
}

export interface PlatformStatusResponse {
  components: PlatformComponentStatus[]
}

export interface PlatformConfigResponse {
  api_url_hint: string
  data_root: string
  flows_dir: string
  request_timeout_secs: number
  max_body_size: number
  default_prompt_set: string
  loaded_flows: string[]
  loaded_prompt_sets: string[]
  default_modules: string[]
  gemini_model_router: string
  gemini_model_response: string
  include_router_decision: boolean
}

export interface StudioProjectDto {
  id: string
  name: string
  description?: string
  dirty: boolean
  published_version: number
  edited_at?: string
}

export interface StudioProjectsResponse {
  projects: StudioProjectDto[]
}

export interface StudioProjectDetailResponse {
  id: string
  name: string
  description?: string
  prompt_set: string
  knowledge_source: string
  flows: string[]
  dirty: boolean
  published_version: number
  edited_at?: string
  bootstrapped_at?: string
}

export interface CreateProjectRequest {
  id: string
  name: string
  description?: string
  prompt_set?: string
  knowledge_source?: string
  flows?: string[]
}

export interface UpdateProjectRequest {
  name?: string
  description?: string
}

export interface DraftCompanyResponse {
  id: string
  name: string
  description?: string
  prompt_set: string
  knowledge_source: string
  flows: string[]
}

export interface UpdateCompanyRequest extends DraftCompanyResponse {}

export interface NodePositionDto {
  x: number
  y: number
}

export interface FlowTransitionDto {
  action: string
  target: string
}

export interface FlowNodeDto {
  id: string
  node_type: string
  payload: Record<string, unknown>
  transitions: FlowTransitionDto[]
  position?: NodePositionDto
}

export interface DraftFlowDto {
  id: string
  name?: string
  version: number
  initial: string
  nodes: FlowNodeDto[]
}

export interface CreateFlowRequest {
  id: string
  name?: string
  initial?: string
}

export interface ValidationIssueDto {
  rule: string
  message: string
  node_id?: string
  action?: string
  target?: string
}

export interface FlowValidationResponse {
  valid: boolean
  issues: ValidationIssueDto[]
}

export interface KnowledgeCategoryDto {
  id: string
  name: string
}

export interface KnowledgeDocumentDto {
  id: string
  category_id: string
  title: string
  section: string
  content: string
  metadata: Record<string, unknown>
}

export interface DraftKnowledgeResponse {
  categories: KnowledgeCategoryDto[]
  documents: KnowledgeDocumentDto[]
}

export interface UpsertDocumentRequest {
  id?: string
  category_id: string
  title: string
  section: string
  content: string
  metadata?: Record<string, unknown>
}

export interface KnowledgeSearchRequest {
  query: string
  top_k?: number
}

export interface SearchCandidateDto {
  module_id: string
  section_id?: string
  score: number
  title: string
  snippet: string
}

export interface KnowledgeSearchResponse {
  results: SearchCandidateDto[]
}

export interface PromptTemplateDto {
  system: string
  developer: string
  assembly: string
}

export interface DraftPromptResponse {
  id: string
  name: string
  version: string
  identity: Record<string, unknown>
  router: PromptTemplateDto
  response: PromptTemplateDto
}

export interface UpdatePromptRequest extends DraftPromptResponse {}

export interface PromptPreviewRequest {
  template: string
  sample_context?: Record<string, unknown>
}

export interface PromptPreviewResponse {
  rendered: string
}

export interface ClientTemplateResponse {
  name: string
  data: Record<string, unknown>
}

export interface UpsertClientRequest {
  name: string
  data: Record<string, unknown>
}

export interface PublishResponse {
  status: string
  version?: number
  stage?: string
  error?: string
  report?: Record<string, unknown>
}

export interface VersionDto {
  version: number
  published_at: string
  files: string[]
}

export interface VersionsResponse {
  versions: VersionDto[]
}

export interface VersionDetailResponse {
  version: number
  published_at: string
  files: string[]
  manifest: Record<string, unknown>
}

export interface ProjectStatusResponse {
  project_id: string
  dirty: boolean
  published_version: number
  edited_at?: string
  bootstrapped_at?: string
  draft_exists: boolean
}

export interface ProjectAssetsResponse {
  project_id: string
  company: DraftCompanyResponse
  flows: string[]
  prompt_sets: string[]
  clients: string[]
  knowledge_documents: number
  knowledge_categories: number
  modules: string[]
}

export interface KnowledgeReindexResponse {
  status: string
  document_count: number
}

export interface CreateSessionRequest {
  flow_id: string
  client_source_path: string
  prompt_set?: string
}

export interface CreateSessionResponse {
  session_id: string
  current_node?: string
  prompt_set: string
  metadata: Record<string, unknown>
  opening_message?: string
}

export interface ActiveModuleDto {
  module_id: string
  section?: string
}

export interface SessionResponse {
  session_id: string
  flow_id?: string
  current_node?: string
  runtime_state: RuntimeState
  active_module?: ActiveModuleDto
  metadata: Record<string, unknown>
}

export interface SessionListItemDto {
  session_id: string
  flow_id?: string
  prompt_set: string
  current_node?: string
  runtime_state: RuntimeState
  active_module?: ActiveModuleDto
  created_at?: string
  last_activity?: string
}

export interface SessionsListResponse {
  sessions: SessionListItemDto[]
}

export interface MessageRequest {
  message: string
}

export interface TimelineStepDto {
  step: string
  duration_ms: number
}

export interface FlowDebugDto {
  flow_id?: string
  previous_node?: string
  current_node?: string
  paused_node?: string
  available_transitions: string[]
  is_end: boolean
  runtime_state: string
}

export interface ModuleDebugDto {
  active_module?: string
  conversation_entries: Record<string, string>
}

export interface PromptDebugDto {
  prompt_set: string
  identity_name: string
  identity_role: string
  router_prompt: string
  response_prompt: string
}

export interface RuntimeDebugDto {
  search_ms: number
  router_ms: number
  response_ms: number
  total_ms: number
  timeline: TimelineStepDto[]
}

export interface JsonDebugDto {
  router_raw_json: string
  response_raw_json: string
  router_decision: Record<string, unknown>
}

export interface MessageDebugDto {
  search_candidates: SearchCandidateDto[]
  flow: FlowDebugDto
  modules: ModuleDebugDto
  prompts: PromptDebugDto
  runtime: RuntimeDebugDto
  json: JsonDebugDto
}

export interface MessageResponse {
  assistant_message: string
  current_node?: string
  active_module?: ActiveModuleDto
  finish_conversation: boolean
  router_decision?: Record<string, unknown>
  debug?: MessageDebugDto
}

export interface HistoryEntryDto {
  role: string
  content: string
  event?: string
  timestamp: string
}

export interface HistoryResponse {
  entries: HistoryEntryDto[]
}

export interface SessionInspectResponse {
  session_id: string
  flow_id?: string
  current_node?: string
  paused_node?: string
  runtime_state: RuntimeState
  active_module?: ActiveModuleDto
  prompt_set: string
  metadata: Record<string, unknown>
  client_data: Record<string, unknown>
}

export interface FlowInspectResponse {
  flow_id?: string
  current_node?: string
  paused_node?: string
  available_transitions: string[]
  node_payload: Record<string, unknown>
  is_end: boolean
  runtime_state: RuntimeState
}

export interface EvalScenarioDto {
  name: string
  domain?: string
  llm?: string
  source_path: string
}

export interface EvalScenariosResponse {
  scenarios: EvalScenarioDto[]
}

export interface EvalRunRequest {
  scenario?: string
  domain?: string
  llm?: string
}

export interface AssertionFailureDto {
  field: string
  expected: string
  actual: string
}

export interface EvalScenarioResultDto {
  name: string
  domain?: string
  status: string
  skip_reason?: string
  failures: AssertionFailureDto[]
}

export interface EvalMetricsDto {
  total: number
  passed: number
  failed: number
  skipped: number
  duration_ms: number
}

export interface EvalReportDto {
  metrics: EvalMetricsDto
  results: EvalScenarioResultDto[]
}

export interface EvalRunResponse {
  run_id?: string
  status: string
  report?: EvalReportDto
}

export interface EvalAsyncRunResponse {
  run_id: string
  status: string
}

export interface EvalRunStatusResponse {
  run_id: string
  status: string
  report?: EvalReportDto
  error?: string
}

export const PUBLISH_STAGES = [
  'validate',
  'backup',
  'save',
  'reindex',
  'reload_prompts',
  'reload_flows',
  'evals',
  'version',
  'done',
] as const

export type PublishStage = (typeof PUBLISH_STAGES)[number]
