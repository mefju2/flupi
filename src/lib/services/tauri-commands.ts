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
