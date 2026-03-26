import { writable } from 'svelte/store';

export interface ExecutionResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  durationMs: number;
}

export const lastResponse = writable<ExecutionResponse | null>(null);
export const isExecuting = writable<boolean>(false);
