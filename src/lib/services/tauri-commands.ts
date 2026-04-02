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
}

export async function getRecentProjects(): Promise<RecentProjects> {
  return invoke('get_recent_projects');
}

export async function addRecentProject(name: string, path: string): Promise<void> {
  return invoke('add_recent_project', { name, path });
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

export type BodyConfig =
  | { type: 'none' }
  | { type: 'json'; content: unknown }
  | { type: 'form'; content: Record<string, string>; disabledFields?: string[] }
  | { type: 'raw'; content: string };

export interface TemplateRef {
  sourceId: string;
  operationId: string;
  schemaHash: string;
  requestSchema: unknown;
  responseSchema: unknown;
}

export interface RequestData {
  name: string;
  method: string;
  path: string;
  auth?: AuthConfig;
  headers: Record<string, string>;
  disabledHeaders?: string[];
  disabledCollectionHeaders?: string[];
  body?: BodyConfig;
  templateRef?: TemplateRef;
  pathParams?: Record<string, string>;
  extractions?: Extraction[];
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

export async function sendRequest(
  projectPath: string,
  requestId: string,
  envFileName: string,
  timeoutMs: number = 30000,
): Promise<HttpResponse> {
  return invoke('send_request', { projectPath, requestId, envFileName, timeoutMs });
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
}

export interface ScenarioStep {
  id: string;
  name: string;
  requestId: string;
  overrides: Record<string, string>;
  extract: Extraction[];
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
}

export async function runScenario(
  projectPath: string,
  scenarioId: string,
  envFileName: string,
  inputs: Record<string, string>,
  timeoutMs: number = 30000,
): Promise<void> {
  return invoke('run_scenario', { projectPath, scenarioId, envFileName, inputs, timeoutMs });
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
