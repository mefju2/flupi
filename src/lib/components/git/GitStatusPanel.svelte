<script lang="ts">
  import { onMount } from "svelte";
  import { RefreshCw, GitBranch } from "lucide-svelte";
  import { project } from "$lib/stores/project";
  import { gitPageState, gitAutoRefreshMs } from "$lib/stores/git";
  import { formatRelativeTime } from "$lib/utils/format";
  import GitBranchHeader from "./GitBranchHeader.svelte";
  import GitChangesPanel from "./GitChangesPanel.svelte";
  import GitCommitPanel from "./GitCommitPanel.svelte";
  import {
    load,
    handleFetch,
    handlePull,
    handlePush,
    handleStageFile,
    handleUnstageFile,
    handleStageAll,
    handleUnstageAll,
    handleCommit,
    handleLoadBranches,
    handleCheckoutBranch,
    selectFile,
    initAutoRefresh,
  } from "./git-actions";

  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  const lastRefreshedLabel = $derived(
    $gitPageState.lastRefreshed
      ? formatRelativeTime($gitPageState.lastRefreshed.toISOString())
      : null,
  );

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

  onMount(() => {
    initAutoRefresh();
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
      <RefreshCw
        size={12}
        class={$gitPageState.isLoading ? "animate-spin" : ""}
      />
      Refresh
    </button>
  </div>

  {#if $gitPageState.lastRefreshed}
    <p class="text-xs text-app-text-3 -mt-2 shrink-0">
      <span
        title={$gitPageState.lastRefreshed.toLocaleTimeString()}
        class="cursor-default border-b border-dotted border-app-border"
      >
        {lastRefreshedLabel}
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
      conflictError={$gitPageState.conflictError}
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
        selectedFile={$gitPageState.selectedFile}
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
