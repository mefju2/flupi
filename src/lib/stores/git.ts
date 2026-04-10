import { writable } from 'svelte/store';
import type { GitStatus, BranchInfo } from '$lib/services/tauri-commands';

export type { GitStatus };

export type GitFileStatus = 'staged' | 'modified' | 'deleted' | 'untracked';

export interface GitSelectedFile {
  path: string;
  status: GitFileStatus;
}

export interface GitPageState {
  status: GitStatus | null;
  isLoading: boolean;
  isFetching: boolean;
  isPulling: boolean;
  isPushing: boolean;
  isCommitting: boolean;
  isSwitchingBranch: boolean;
  error: string | null;
  conflictError: string | null;
  lastRefreshed: Date | null;
  lastFetched: Date | null;
  selectedFile: GitSelectedFile | null;
  branches: BranchInfo[];
}

export const gitAutoRefreshMs = writable<number>(30_000);

/** Number of commits the current branch is behind its remote, as determined by
 *  the background fetch that runs on app startup. Cleared when the user visits
 *  the Git page. */
export const gitBehindCount = writable<number>(0);

export const gitPageState = writable<GitPageState>({
  status: null,
  isLoading: false,
  isFetching: false,
  isPulling: false,
  isPushing: false,
  isCommitting: false,
  isSwitchingBranch: false,
  error: null,
  conflictError: null,
  lastRefreshed: null,
  lastFetched: null,
  selectedFile: null,
  branches: [],
});
