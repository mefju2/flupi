import { get } from 'svelte/store';
import { project } from '$lib/stores/project';
import { gitPageState, gitAutoRefreshMs, type GitFileStatus } from '$lib/stores/git';
import {
  getGitStatus,
  gitFetch,
  gitPull,
  gitPush,
  gitStageFile,
  gitUnstageFile,
  gitStageAll,
  gitUnstageAll,
  gitCommit,
  gitListBranches,
  gitCheckoutBranch,
  gitDiscardFile,
  gitDeleteFile,
  getPreferences,
} from '$lib/services/tauri-commands';

export async function load() {
  const path = get(project).path;
  if (!path) return;
  gitPageState.update((s) => ({ ...s, isLoading: s.status === null, error: null }));
  try {
    const status = await getGitStatus(path);
    gitPageState.update((s) => ({ ...s, status, isLoading: false, lastRefreshed: new Date() }));
  } catch (e) {
    gitPageState.update((s) => ({ ...s, isLoading: false, error: String(e) }));
  }
}

export async function handleFetch() {
  const path = get(project).path;
  if (!path) return;
  gitPageState.update((s) => ({ ...s, isFetching: true, error: null, conflictError: null }));
  try {
    await gitFetch(path);
    await load();
    gitPageState.update((s) => ({ ...s, lastFetched: new Date() }));
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  } finally {
    gitPageState.update((s) => ({ ...s, isFetching: false }));
  }
}

export async function handlePull() {
  const path = get(project).path;
  if (!path) return;
  gitPageState.update((s) => ({ ...s, isPulling: true, error: null, conflictError: null }));
  try {
    await gitPull(path);
    await load();
  } catch (e) {
    const msg = String(e);
    if (msg.includes('CONFLICT')) {
      gitPageState.update((s) => ({ ...s, conflictError: msg.replace('CONFLICT: ', '') }));
    } else {
      gitPageState.update((s) => ({ ...s, error: msg }));
    }
  } finally {
    gitPageState.update((s) => ({ ...s, isPulling: false }));
  }
}

export async function handlePush() {
  const path = get(project).path;
  if (!path) return;
  gitPageState.update((s) => ({ ...s, isPushing: true, error: null }));
  try {
    await gitPush(path);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  } finally {
    gitPageState.update((s) => ({ ...s, isPushing: false }));
  }
}

export async function handleStageFile(filePath: string) {
  const path = get(project).path;
  if (!path) return;
  try {
    await gitStageFile(path, filePath);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  }
}

export async function handleUnstageFile(filePath: string) {
  const path = get(project).path;
  if (!path) return;
  try {
    await gitUnstageFile(path, filePath);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  }
}

export async function handleStageAll() {
  const path = get(project).path;
  if (!path) return;
  try {
    await gitStageAll(path);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  }
}

export async function handleUnstageAll() {
  const path = get(project).path;
  if (!path) return;
  try {
    await gitUnstageAll(path);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  }
}

export async function handleCommit(message: string) {
  const path = get(project).path;
  if (!path) return;
  gitPageState.update((s) => ({ ...s, isCommitting: true, error: null }));
  try {
    await gitCommit(path, message);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  } finally {
    gitPageState.update((s) => ({ ...s, isCommitting: false }));
  }
}

export async function handleLoadBranches() {
  const path = get(project).path;
  if (!path) return;
  try {
    const branches = await gitListBranches(path);
    gitPageState.update((s) => ({ ...s, branches }));
  } catch (_) {
    // Non-critical — branch dropdown stays empty on error
  }
}

export async function handleCheckoutBranch(branch: string, isRemote: boolean) {
  const path = get(project).path;
  if (!path) return;
  gitPageState.update((s) => ({ ...s, isSwitchingBranch: true, error: null }));
  try {
    await gitCheckoutBranch(path, branch, isRemote);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  } finally {
    gitPageState.update((s) => ({ ...s, isSwitchingBranch: false }));
  }
}

export function selectFile(path: string, status: GitFileStatus) {
  gitPageState.update((s) => ({ ...s, selectedFile: { path, status } }));
}

export async function handleDiscardFile(filePath: string) {
  const path = get(project).path;
  if (!path) return;
  try {
    await gitDiscardFile(path, filePath);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  }
}

export async function handleDeleteFile(filePath: string) {
  const path = get(project).path;
  if (!path) return;
  try {
    await gitDeleteFile(path, filePath);
    await load();
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  }
}

export async function handleShowInFileExplorer(filePath: string) {
  const path = get(project).path;
  if (!path) return;
  try {
    const { revealItemInDir } = await import('@tauri-apps/plugin-opener');
    await revealItemInDir(`${path}/${filePath}`);
  } catch (e) {
    gitPageState.update((s) => ({ ...s, error: String(e) }));
  }
}

export async function initAutoRefresh(): Promise<void> {
  const prefs = await getPreferences();
  gitAutoRefreshMs.set(prefs.gitAutoRefreshMs ?? 30_000);
}
