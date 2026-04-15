import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface RecentProject {
  name: string;
  path: string;
  lastOpenedAt: string;
  activeEnvironment?: string | null;
}

export interface RecentProjects {
  projects: RecentProject[];
}

export interface Preferences {
  theme: string;
  defaultTimeoutMs: number;
  gitAutoRefreshMs: number;
}

export interface GitStatus {
  branch: string;
  upstream: string | null;
  ahead: number;
  behind: number;
  staged: string[];
  modified: string[];
  deleted: string[];
  untracked: string[];
  isGitRepo: boolean;
}

export async function getRecentProjects(): Promise<RecentProjects> {
  return invoke('get_recent_projects');
}

export async function addRecentProject(name: string, path: string): Promise<void> {
  return invoke('add_recent_project', { name, path });
}

export async function removeRecentProject(path: string): Promise<void> {
  return invoke('remove_recent_project', { path });
}

export async function setProjectActiveEnvironment(path: string, envFileName: string | null): Promise<void> {
  return invoke('set_project_active_environment', { path, envFileName });
}

export async function createProject(path: string): Promise<void> {
  return invoke('create_project', { path });
}

export async function openProject(path: string): Promise<string> {
  return invoke('open_project', { path });
}

export async function getPreferences(): Promise<Preferences> {
  return invoke('get_preferences');
}

export async function savePreferences(preferences: Preferences): Promise<void> {
  return invoke('save_preferences', { preferences });
}

export async function getGitStatus(projectPath: string): Promise<GitStatus> {
  return invoke('get_git_status', { projectPath });
}

export async function gitFetch(projectPath: string): Promise<void> {
  return invoke('git_fetch', { projectPath });
}

export async function gitPull(projectPath: string): Promise<void> {
  return invoke('git_pull', { projectPath });
}

export interface DiffLine {
  type: 'add' | 'remove' | 'same';
  text: string;
}

export interface GitFileDiff {
  lines: DiffLine[];
  isNewFile: boolean;
}

export async function gitFileDiff(projectPath: string, filePath: string): Promise<GitFileDiff> {
  return invoke('git_file_diff', { projectPath, filePath });
}

export async function diffText(oldText: string, newText: string): Promise<DiffLine[]> {
  return invoke('diff_text', { oldText, newText });
}

export interface BranchInfo {
  name: string;
  isCurrent: boolean;
  isRemote: boolean;
}

export async function gitStageFile(projectPath: string, filePath: string): Promise<void> {
  return invoke('git_stage_file', { projectPath, filePath });
}

export async function gitUnstageFile(projectPath: string, filePath: string): Promise<void> {
  return invoke('git_unstage_file', { projectPath, filePath });
}

export async function gitStageAll(projectPath: string): Promise<void> {
  return invoke('git_stage_all', { projectPath });
}

export async function gitUnstageAll(projectPath: string): Promise<void> {
  return invoke('git_unstage_all', { projectPath });
}

export async function gitCommit(projectPath: string, message: string): Promise<void> {
  return invoke('git_commit', { projectPath, message });
}

export async function gitPush(projectPath: string): Promise<void> {
  return invoke('git_push', { projectPath });
}

export async function gitListBranches(projectPath: string): Promise<BranchInfo[]> {
  return invoke('git_list_branches', { projectPath });
}

export async function gitCheckoutBranch(projectPath: string, branch: string, isRemote: boolean): Promise<void> {
  return invoke('git_checkout_branch', { projectPath, branch, isRemote });
}

export async function gitDiscardFile(projectPath: string, filePath: string): Promise<void> {
  return invoke('git_discard_file', { projectPath, filePath });
}

export async function gitDeleteFile(projectPath: string, filePath: string): Promise<void> {
  return invoke('git_delete_file', { projectPath, filePath });
}

export async function pickFolder(): Promise<string | null> {
  return open({ directory: true });
}

export interface Environment {
  name: string;
  variables: Record<string, string>;
  secrets: string[];
}

export async function listEnvironments(projectPath: string): Promise<[string, Environment][]> {
  return invoke('list_environments', { projectPath });
}

export async function saveEnvironment(projectPath: string, fileName: string, env: Environment): Promise<void> {
  return invoke('save_environment', { projectPath, fileName, env });
}

export async function saveSecrets(projectPath: string, fileName: string, secrets: Record<string, string>): Promise<void> {
  return invoke('save_secrets', { projectPath, fileName, secrets });
}

export async function getResolvedVariables(projectPath: string, fileName: string): Promise<Record<string, string>> {
  return invoke('get_resolved_variables', { projectPath, fileName });
}

export async function deleteEnvironment(projectPath: string, fileName: string): Promise<void> {
  return invoke('delete_environment', { projectPath, fileName });
}

export async function duplicateEnvironment(projectPath: string, fileName: string): Promise<string> {
  return invoke('duplicate_environment', { projectPath, fileName });
}

export async function renameEnvironment(projectPath: string, fileName: string, newName: string): Promise<string> {
  return invoke('rename_environment', { projectPath, fileName, newName });
}

// === Request Library Types ===

export type RequestTreeNode =
  | { type: 'Collection'; name: string; folder_name: string; children: RequestTreeNode[] }
  | { type: 'Folder'; name: string; children: RequestTreeNode[] }
  | { type: 'Request'; id: string; name: string; method: string };

export type AuthConfig =
  | { type: 'none' }
  | { type: 'inherit' }
  | { type: 'bearer'; token: string }
  | { type: 'basic'; username: string; password: string }
  | { type: 'apiKey'; header: string; value: string }
  | { type: 'custom'; headers: Record<string, string> };

export type RawFormat = 'json' | 'xml' | 'text';

export type BodyConfig =
  | { type: 'none' }
  | { type: 'form-urlencoded'; content: Record<string, string>; disabledFields?: string[] }
  | { type: 'raw'; format: RawFormat; content: string };

export interface TemplateRef {
  sourceId: string;
  operationId: string;
  schemaHash: string;
  requestSchema: unknown;
  responseSchema: unknown;
}

export interface SetVariableAction {
  type: 'set_variable';
  variable: string;
  function_name: string;
  args: string[];
}

export type PreRequestAction = SetVariableAction;

export interface RequestData {
  name: string;
  method: string;
  path: string;
  collection?: string;
  auth?: AuthConfig;
  headers: Record<string, string>;
  disabledHeaders?: string[];
  disabledCollectionHeaders?: string[];
  body?: BodyConfig;
  templateRef?: TemplateRef;
  pathParams?: Record<string, string>;
  extractions?: Extraction[];
  preRequestActions?: PreRequestAction[];
}

export interface CollectionData {
  name: string;
  baseUrl?: string;
  auth?: AuthConfig;
  headers: Record<string, string>;
}

// === Request Tree Commands ===
export async function loadRequestTree(projectPath: string): Promise<RequestTreeNode[]> {
  return invoke('load_request_tree', { projectPath });
}

// === Request CRUD Commands ===
export async function getRequest(projectPath: string, requestId: string): Promise<RequestData> {
  return invoke('get_request', { projectPath, requestId });
}

export async function saveRequest(projectPath: string, requestId: string, request: RequestData): Promise<void> {
  return invoke('save_request', { projectPath, requestId, request });
}

export async function createRequest(projectPath: string, collectionFolder: string | null, name: string): Promise<string> {
  return invoke('create_request', { projectPath, collectionFolder, name });
}

export async function deleteRequest(projectPath: string, requestId: string): Promise<void> {
  return invoke('delete_request', { projectPath, requestId });
}

export async function renameRequest(projectPath: string, requestId: string, newName: string): Promise<string> {
  return invoke('rename_request', { projectPath, requestId, newName });
}

export async function moveRequest(projectPath: string, requestId: string, targetCollectionFolder: string | null): Promise<string> {
  return invoke('move_request', { projectPath, requestId, targetCollectionFolder });
}

export async function duplicateRequest(projectPath: string, requestId: string): Promise<string> {
  return invoke('duplicate_request', { projectPath, requestId });
}

// === Collection CRUD Commands ===
export async function getCollection(projectPath: string, folderName: string): Promise<CollectionData> {
  return invoke('get_collection', { projectPath, folderName });
}

export async function createCollection(projectPath: string, name: string): Promise<string> {
  return invoke('create_collection', { projectPath, name });
}

export async function saveCollection(projectPath: string, folderName: string, collection: CollectionData): Promise<void> {
  return invoke('save_collection', { projectPath, folderName, collection });
}

export async function deleteCollection(projectPath: string, folderName: string): Promise<void> {
  return invoke('delete_collection', { projectPath, folderName });
}

export async function renameCollection(projectPath: string, folderName: string, newName: string): Promise<string> {
  return invoke('rename_collection', { projectPath, folderName, newName });
}

// === Execution Commands ===

export interface HttpResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  durationMs: number;
  bodyTruncated: boolean;
}

export interface SendRequestResult {
  response: HttpResponse;
  sent_request: SentRequest;
}

export async function sendRequest(
  projectPath: string,
  requestId: string,
  envFileName: string,
  timeoutMs: number = 30000,
  injectedVars?: Record<string, string>,
): Promise<SendRequestResult> {
  return invoke('send_request', { projectPath, requestId, envFileName, timeoutMs, injectedVars: injectedVars ?? null });
}

// === Scenario Types ===

export interface ScenarioInput {
  name: string;
  description: string;
  default: string;
  required: boolean;
}

export interface Extraction {
  variable: string;
  from: string; // "response.body" | "response.headers"
  path: string;
  scope?: 'env' | 'scenario';
}

export interface RequestStep {
  id: string;
  name: string;
  requestId: string;
  overrides: Record<string, string>;
  extract: Extraction[];
  expectedStatus?: string[];
}

export interface DelayStep {
  id: string;
  name: string;
  duration: number; // milliseconds
}

export interface PauseStep {
  id: string;
  name: string;
  pause: true; // discriminator — always true
}

export type ScenarioStep = RequestStep | DelayStep | PauseStep;

export function isDelayStep(step: ScenarioStep): step is DelayStep {
  return 'duration' in step;
}

export function isRequestStep(step: ScenarioStep): step is RequestStep {
  return 'requestId' in step;
}

export function isPauseStep(step: ScenarioStep): step is PauseStep {
  return 'pause' in step;
}

export interface ScenarioData {
  name: string;
  inputs: ScenarioInput[];
  steps: ScenarioStep[];
}

export type ScenarioTreeNode =
  | { type: 'Scenario'; id: string; name: string }
  | { type: 'Group'; name: string; children: ScenarioTreeNode[] };

// === Scenario CRUD Commands ===

export async function loadScenarioTree(projectPath: string): Promise<ScenarioTreeNode[]> {
  return invoke('load_scenario_tree', { projectPath });
}

export async function getScenario(projectPath: string, scenarioId: string): Promise<ScenarioData> {
  return invoke('get_scenario', { projectPath, scenarioId });
}

export async function saveScenario(projectPath: string, scenarioId: string, scenario: ScenarioData): Promise<void> {
  return invoke('save_scenario', { projectPath, scenarioId, scenario });
}

export async function createScenario(projectPath: string, group: string | null, name: string): Promise<string> {
  return invoke('create_scenario', { projectPath, group, name });
}

export async function deleteScenario(projectPath: string, scenarioId: string): Promise<void> {
  return invoke('delete_scenario', { projectPath, scenarioId });
}

export async function renameScenario(projectPath: string, scenarioId: string, newName: string): Promise<string> {
  return invoke('rename_scenario', { projectPath, scenarioId, newName });
}

export async function duplicateScenario(projectPath: string, scenarioId: string): Promise<string> {
  return invoke('duplicate_scenario', { projectPath, scenarioId });
}

// === Scenario Runner ===

export interface StepResult {
  step_id: string;
  status: 'success' | 'error';
  response?: HttpResponse;
  error?: string;
  extracted: Record<string, string>;
  sent_request?: SentRequest | null;
}

export interface SentRequestBody {
  type: 'json' | 'form' | 'raw';
  content: unknown;
}

export type SentRequestBodyTyped =
  | { type: 'json'; content: unknown }
  | { type: 'form'; content: Record<string, string> }
  | { type: 'raw'; content: string };

export interface SentRequest {
  method: string;
  url: string;
  headers: Record<string, string>;
  body?: SentRequestBodyTyped | null;
  timeout_ms: number;
}

export async function runScenario(
  projectPath: string,
  scenarioId: string,
  envFileName: string,
  inputs: Record<string, string>,
  timeoutMs: number = 30000,
  injectedVars?: Record<string, string>,
): Promise<void> {
  return invoke('run_scenario', { projectPath, scenarioId, envFileName, inputs, timeoutMs, injectedVars: injectedVars ?? null });
}

export async function resumeScenario(resume: boolean): Promise<void> {
  return invoke('resume_scenario', { resume });
}

// === OpenAPI Types ===

export type OpenApiSource =
  | { type: 'url'; id: string; name: string; url: string; lastFetchedAt: string | null; lastHash: string | null }
  | { type: 'file'; id: string; name: string; path: string; lastFetchedAt: string | null; lastHash: string | null };

export interface ImportableOperation {
  tag: string;
  operationId: string;
  method: string;
  path: string;
  summary: string | null;
}

// === OpenAPI Commands ===

export async function listOpenApiSources(projectPath: string): Promise<OpenApiSource[]> {
  return invoke('list_openapi_sources', { projectPath });
}

export async function addOpenApiSource(projectPath: string, source: OpenApiSource): Promise<void> {
  return invoke('add_openapi_source', { projectPath, source });
}

export async function removeOpenApiSource(projectPath: string, sourceId: string): Promise<void> {
  return invoke('remove_openapi_source', { projectPath, sourceId });
}

export async function fetchOperations(projectPath: string, sourceId: string): Promise<ImportableOperation[]> {
  return invoke('fetch_operations', { projectPath, sourceId });
}

export async function importOperations(
  projectPath: string,
  sourceId: string,
  operationIds: string[],
  collectionFolder: string,
): Promise<string[]> {
  return invoke('import_operations', { projectPath, sourceId, operationIds, collectionFolder });
}

export async function refreshSource(projectPath: string, sourceId: string): Promise<string[]> {
  return invoke('refresh_source', { projectPath, sourceId });
}

export interface PathCandidate {
  operationId: string;
  path: string;
  method: string;
  summary: string | null;
}

export async function resolveDrift(
  projectPath: string,
  requestId: string,
  sourceId: string,
  /** Required when the stored operationId no longer exists (rename case). Null for exact-match path changes. */
  chosenOperationId: string | null,
): Promise<void> {
  return invoke('resolve_drift', { projectPath, requestId, sourceId, chosenOperationId });
}

export interface DriftDetails {
  sourceId: string;
  operationId: string;
  storedPath: string;
  /** For exact-match path change: the new path. Null when user must pick from candidates. */
  currentPath: string | null;
  pathChanged: boolean;
  schemaChanged: boolean;
  operationRemoved: boolean;
  /** Rename candidates sorted by similarity (descending). Non-empty only when stored operationId is gone. */
  candidates: PathCandidate[];
  /** Only populated when schemaChanged is true. */
  storedRequestSchema?: unknown;
  storedResponseSchema?: unknown;
  newRequestSchema?: unknown;
  newResponseSchema?: unknown;
}

export async function getDriftDetails(projectPath: string, requestId: string): Promise<DriftDetails> {
  return invoke('get_drift_details', { projectPath, requestId });
}

export async function renameOpenApiSource(projectPath: string, sourceId: string, newName: string): Promise<void> {
  return invoke('rename_openapi_source', { projectPath, sourceId, newName });
}

export interface SourceRequest {
  id: string;
  name: string;
  method: string;
  path: string;
}

export async function listRequestsBySource(projectPath: string, sourceId: string): Promise<SourceRequest[]> {
  return invoke('list_requests_by_source', { projectPath, sourceId });
}

export async function generateBodyFromSchema(schema: unknown): Promise<string> {
  return invoke('generate_body_from_schema', { schema });
}

// === Script Functions ===

export interface FunctionParam {
  name: string;
  param_type: 'string' | 'number' | 'boolean';
}

export interface ScriptFunction {
  name: string;
  body: string;
  params?: FunctionParam[];
}

export async function listFunctions(projectPath: string): Promise<ScriptFunction[]> {
  return invoke('list_functions', { projectPath });
}

export async function saveFunction(projectPath: string, fn: ScriptFunction): Promise<void> {
  return invoke('save_function', { projectPath, function: fn });
}

export async function deleteFunction(projectPath: string, name: string): Promise<void> {
  return invoke('delete_function', { projectPath, name });
}

export async function renameFunction(projectPath: string, oldName: string, newName: string): Promise<number> {
  return invoke('rename_function', { projectPath, oldName, newName });
}

// === Environment Variable Keys ===

export async function renameVariableKey(projectPath: string, oldKey: string, newKey: string): Promise<number> {
  return invoke('rename_variable_key', { projectPath, oldKey, newKey });
}
