import { writable } from 'svelte/store';
import type { GitStatus } from '$lib/services/tauri-commands';

export type { GitStatus };

export interface GitSelectedFile {
  path: string;
  kind: 'staged' | 'modified' | 'deleted' | 'untracked';
}

export interface GitPageState {
  status: GitStatus | null;
  isLoading: boolean;
  isFetching: boolean;
  isPulling: boolean;
  error: string | null;
  lastRefreshed: Date | null;
  lastFetched: Date | null;
  selectedFile: GitSelectedFile | null;
}

export const gitAutoRefreshMs = writable<number>(30_000);

export const gitPageState = writable<GitPageState>({
  status: null,
  isLoading: false,
  isFetching: false,
  isPulling: false,
  error: null,
  lastRefreshed: null,
  lastFetched: null,
  selectedFile: null,
});
