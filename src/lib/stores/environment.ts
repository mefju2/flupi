import { writable } from 'svelte/store';
import type { Environment } from '$lib/services/tauri-commands';

export interface EnvironmentEntry {
  fileName: string;
  environment: Environment;
  secrets: Record<string, string>;
}

export const environments = writable<EnvironmentEntry[]>([]);
export const activeEnvironment = writable<string | null>(null);
export const selectedEnvironmentFile = writable<string | null>(null);
