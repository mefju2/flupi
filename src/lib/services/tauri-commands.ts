import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface RecentProject {
  name: string;
  path: string;
  lastOpenedAt: string;
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
  | { type: 'form'; content: Record<string, string> }
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
  body?: BodyConfig;
  templateRef?: TemplateRef;
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
