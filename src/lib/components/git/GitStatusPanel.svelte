<script lang="ts">
  import { onMount } from 'svelte';
  import { GitBranch, RefreshCw, ChevronDown, ChevronRight } from 'lucide-svelte';
  import { project } from '$lib/stores/project';
  import { gitPageState, gitAutoRefreshMs, type GitSelectedFile } from '$lib/stores/git';
  import { getGitStatus, gitFetch, gitPull, getPreferences } from '$lib/services/tauri-commands';
  import { formatRelativeTime } from '$lib/utils/format';
  import GitFileTree from './GitFileTree.svelte';
  import GitBranchHeader from './GitBranchHeader.svelte';

  let conflictError = $state<string | null>(null);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;
  let collapsed = $state<Record<string, boolean>>({});

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
      if (msg.includes('CONFLICT')) {
        conflictError = msg.replace('CONFLICT: ', '');
      } else {
        gitPageState.update((s) => ({ ...s, error: msg }));
      }
    } finally {
      gitPageState.update((s) => ({ ...s, isPulling: false }));
    }
  }

  function selectFile(path: string, kind: GitSelectedFile['kind']) {
    gitPageState.update((s) => ({ ...s, selectedFile: { path, kind } }));
  }

  // Restart the interval whenever the configured refresh ms changes.
  $effect(() => {
    const ms = $gitAutoRefreshMs;
    if (refreshInterval !== null) clearInterval(refreshInterval);
    refreshInterval = setInterval(load, ms);
    return () => {
      if (refreshInterval !== null) clearInterval(refreshInterval);
    };
  });

  // Load whenever the project path changes (handles initial load too).
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
    <h2 class="text-sm font-semibold text-app-text">Git Status</h2>
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
      {conflictError}
      error={$gitPageState.error}
      onfetch={handleFetch}
      onpull={handlePull}
    />

    <div class="border-t border-app-border shrink-0"></div>

    <!-- File trees (scrollable) -->
    <div class="flex flex-col gap-4 overflow-y-auto flex-1 min-h-0">
      {#if s.modified.length > 0}
        <section class="flex flex-col gap-1">
          <button
            class="flex items-center gap-1 text-xs font-semibold text-app-text-3 uppercase tracking-wider px-1 hover:text-app-text-2 transition-colors w-full text-left"
            onclick={() => (collapsed.modified = !collapsed.modified)}
          >
            {#if collapsed.modified}<ChevronRight size={12} />{:else}<ChevronDown size={12} />{/if}
            Modified ({s.modified.length})
          </button>
          {#if !collapsed.modified}
            <GitFileTree
              files={s.modified}
              kind="modified"
              selectedPath={$gitPageState.selectedFile?.kind === 'modified' ? $gitPageState.selectedFile.path : null}
              onselect={selectFile}
            />
          {/if}
        </section>
      {/if}

      {#if s.deleted.length > 0}
        <section class="flex flex-col gap-1">
          <button
            class="flex items-center gap-1 text-xs font-semibold text-app-text-3 uppercase tracking-wider px-1 hover:text-app-text-2 transition-colors w-full text-left"
            onclick={() => (collapsed.deleted = !collapsed.deleted)}
          >
            {#if collapsed.deleted}<ChevronRight size={12} />{:else}<ChevronDown size={12} />{/if}
            Deleted ({s.deleted.length})
          </button>
          {#if !collapsed.deleted}
            <GitFileTree
              files={s.deleted}
              kind="deleted"
              selectedPath={$gitPageState.selectedFile?.kind === 'deleted' ? $gitPageState.selectedFile.path : null}
              onselect={selectFile}
            />
          {/if}
        </section>
      {/if}

      {#if s.untracked.length > 0}
        <section class="flex flex-col gap-1">
          <button
            class="flex items-center gap-1 text-xs font-semibold text-app-text-3 uppercase tracking-wider px-1 hover:text-app-text-2 transition-colors w-full text-left"
            onclick={() => (collapsed.untracked = !collapsed.untracked)}
          >
            {#if collapsed.untracked}<ChevronRight size={12} />{:else}<ChevronDown size={12} />{/if}
            Untracked ({s.untracked.length})
          </button>
          {#if !collapsed.untracked}
            <GitFileTree
              files={s.untracked}
              kind="untracked"
              selectedPath={$gitPageState.selectedFile?.kind === 'untracked' ? $gitPageState.selectedFile.path : null}
              onselect={selectFile}
            />
          {/if}
        </section>
      {/if}

      {#if s.modified.length === 0 && s.deleted.length === 0 && s.untracked.length === 0}
        <p class="text-xs text-app-text-3 px-1">Working tree is clean.</p>
      {/if}
    </div>
  {:else if $gitPageState.error}
    <div class="px-2.5 py-2 rounded bg-red-500/10 border border-red-500/30 text-red-400 text-xs font-mono">
      {$gitPageState.error}
    </div>
  {/if}
</div>
