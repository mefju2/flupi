import { writable } from 'svelte/store';
import type { SentRequest } from '$lib/services/tauri-commands';

export interface ExecutionResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  durationMs: number;
}

export const lastResponse = writable<ExecutionResponse | null>(null);
export const lastSentRequest = writable<SentRequest | null>(null);
export const isExecuting = writable<boolean>(false);
export const lastError = writable<string | null>(null);
