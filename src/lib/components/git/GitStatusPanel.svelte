<script lang="ts">
  import { onMount } from 'svelte';
  import { RefreshCw, GitBranch } from 'lucide-svelte';
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
    getPreferences,
  } from '$lib/services/tauri-commands';
  import { formatRelativeTime } from '$lib/utils/format';
  import GitBranchHeader from './GitBranchHeader.svelte';
  import GitChangesPanel from './GitChangesPanel.svelte';
  import GitCommitPanel from './GitCommitPanel.svelte';

  let conflictError = $state<string | null>(null);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  async function load() {
    const path = $project.path;
    if (!path) return;
    gitPageState.update((s) => ({ ...s, isLoading: s.status === null, error: null }));
    try {
      const status = await getGitStatus(path);
      gitPageState.update((s) => ({ ...s, status, isLoading: false, lastRefreshed: new Date() }));
    } catch (e) {
      gitPageState.update((s) => ({ ...s, isLoading: false, error: String(e) }));
    }
  }

  async function handleFetch() {
    const path = $project.path;
    if (!path) return;
    conflictError = null;
    gitPageState.update((s) => ({ ...s, isFetching: true, error: null }));
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

  async function handlePull() {
    const path = $project.path;
    if (!path) return;
    conflictError = null;
    gitPageState.update((s) => ({ ...s, isPulling: true, error: null }));
    try {
      await gitPull(path);
      await load();
    } catch (e) {
      const msg = String(e);
      if (msg.includes('CONFLICT')) conflictError = msg.replace('CONFLICT: ', '');
      else gitPageState.update((s) => ({ ...s, error: msg }));
    } finally {
      gitPageState.update((s) => ({ ...s, isPulling: false }));
    }
  }

  async function handlePush() {
    const path = $project.path;
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

  async function handleStageFile(filePath: string) {
    const path = $project.path;
    if (!path) return;
    try {
      await gitStageFile(path, filePath);
      await load();
    } catch (e) {
      gitPageState.update((s) => ({ ...s, error: String(e) }));
    }
  }

  async function handleUnstageFile(filePath: string) {
    const path = $project.path;
    if (!path) return;
    try {
      await gitUnstageFile(path, filePath);
      await load();
    } catch (e) {
      gitPageState.update((s) => ({ ...s, error: String(e) }));
    }
  }

  async function handleStageAll() {
    const path = $project.path;
    if (!path) return;
    try {
      await gitStageAll(path);
      await load();
    } catch (e) {
      gitPageState.update((s) => ({ ...s, error: String(e) }));
    }
  }

  async function handleUnstageAll() {
    const path = $project.path;
    if (!path) return;
    try {
      await gitUnstageAll(path);
      await load();
    } catch (e) {
      gitPageState.update((s) => ({ ...s, error: String(e) }));
    }
  }

  async function handleCommit(message: string) {
    const path = $project.path;
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

  async function handleLoadBranches() {
    const path = $project.path;
    if (!path) return;
    try {
      const branches = await gitListBranches(path);
      gitPageState.update((s) => ({ ...s, branches }));
    } catch (_) {
      // Non-critical — branch dropdown stays empty on error
    }
  }

  async function handleCheckoutBranch(branch: string) {
    const path = $project.path;
    if (!path) return;
    gitPageState.update((s) => ({ ...s, isSwitchingBranch: true, error: null }));
    try {
      await gitCheckoutBranch(path, branch);
      await load();
    } catch (e) {
      gitPageState.update((s) => ({ ...s, error: String(e) }));
    } finally {
      gitPageState.update((s) => ({ ...s, isSwitchingBranch: false }));
    }
  }

  function selectFile(path: string, status: GitFileStatus) {
    gitPageState.update((s) => ({ ...s, selectedFile: { path, status } }));
  }

  $effect(() => {
    const ms = $gitAutoRefreshMs;
    if (refreshInterval !== null) clearInterval(refreshInterval);
    refreshInterval = setInterval(load, ms);
    return () => {
      if (refreshInterval !== null) clearInterval(refreshInterval);
    };
  });

  $effect(() => {
    const path = $project.path;
    if (path) load();
  });

  onMount(async () => {
    const prefs = await getPreferences();
    gitAutoRefreshMs.set(prefs.gitAutoRefreshMs ?? 30_000);
  });
</script>

<div class="flex flex-col gap-4 p-4 h-full">
  <!-- Header -->
  <div class="flex items-center justify-between shrink-0">
    <h2 class="text-sm font-semibold text-app-text">Git</h2>
    <button
      class="flex items-center gap-1 px-2 py-1 rounded text-xs text-app-text-3
             hover:bg-app-card hover:text-app-text-2 transition-colors disabled:opacity-50"
      onclick={load}
      disabled={$gitPageState.isLoading}
      aria-label="Refresh"
    >
      <RefreshCw size={12} class={$gitPageState.isLoading ? 'animate-spin' : ''} />
      Refresh
    </button>
  </div>

  {#if $gitPageState.lastRefreshed}
    <p class="text-xs text-app-text-3 -mt-2 shrink-0">
      <span
        title={$gitPageState.lastRefreshed.toLocaleTimeString()}
        class="cursor-default border-b border-dotted border-app-border"
      >
        {formatRelativeTime($gitPageState.lastRefreshed.toISOString())}
      </span>
    </p>
  {/if}

  {#if $gitPageState.isLoading && !$gitPageState.status}
    <div class="flex items-center gap-2 text-xs text-app-text-3">
      <RefreshCw size={12} class="animate-spin" /> Loading…
    </div>
  {:else if $gitPageState.status && !$gitPageState.status.isGitRepo}
    <div class="flex flex-col items-center gap-2 py-8 text-app-text-3">
      <GitBranch size={24} class="opacity-40" />
      <p class="text-xs text-center">Not a git repository.</p>
    </div>
  {:else if $gitPageState.status}
    {@const s = $gitPageState.status}

    <GitBranchHeader
      status={s}
      isFetching={$gitPageState.isFetching}
      isPulling={$gitPageState.isPulling}
      isPushing={$gitPageState.isPushing}
      isSwitchingBranch={$gitPageState.isSwitchingBranch}
      branches={$gitPageState.branches}
      lastFetched={$gitPageState.lastFetched}
      {conflictError}
      error={$gitPageState.error}
      onfetch={handleFetch}
      onpull={handlePull}
      onpush={handlePush}
      onbranch={handleCheckoutBranch}
      onloadbranches={handleLoadBranches}
    />

    <div class="border-t border-app-border shrink-0"></div>

    <!-- Changes (scrollable) -->
    <div class="overflow-y-auto flex-1 min-h-0">
      <GitChangesPanel
        status={s}
        selectedPath={$gitPageState.selectedFile?.path ?? null}
        onselect={selectFile}
        onstage={handleStageFile}
        onunstage={handleUnstageFile}
        onstageall={handleStageAll}
        onunstageall={handleUnstageAll}
      />
    </div>

    <!-- Commit (fixed at bottom) -->
    <GitCommitPanel
      hasStagedFiles={s.staged.length > 0}
      isCommitting={$gitPageState.isCommitting}
      oncommit={handleCommit}
    />
  {:else if $gitPageState.error}
    <div
      class="px-2.5 py-2 rounded bg-red-500/10 border border-red-500/30 text-red-400 text-xs font-mono"
    >
      {$gitPageState.error}
    </div>
  {/if}
</div>
