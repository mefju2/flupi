<script lang="ts">
  import { project } from "$lib/stores/project";
  import { openApiSources, driftedIdsBySource } from "$lib/stores/openapi";
  import {
    removeOpenApiSource,
    refreshSource,
    listOpenApiSources,
  } from "$lib/services/tauri-commands";
  import EmptyState from "$lib/components/shared/EmptyState.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { formatRelativeTime } from "$lib/utils/format";

  interface Props {
    onAddSource: () => void;
    onImport: (sourceId: string) => void;
    onSelectSource?: (sourceId: string) => void;
    selectedSourceId?: string | null;
    addedSourceId?: string | null;
  }

  let {
    onAddSource,
    onImport,
    onSelectSource,
    selectedSourceId = null,
    addedSourceId = null,
  }: Props = $props();

  let loadingIds = $state<Set<string>>(new Set());
  let syncedSources = $state<Set<string>>(new Set());
  let syncErrors = $state<Map<string, string>>(new Map());
  let deletingIds = $state<Set<string>>(new Set());
  let pendingDeleteId = $state<string | null>(null);

  async function handleRefresh(sourceId: string) {
    if (!$project.path) return;
    const addLoading = new Set(loadingIds);
    addLoading.add(sourceId);
    loadingIds = addLoading;
    const nextErrors = new Map(syncErrors);
    nextErrors.delete(sourceId);
    syncErrors = nextErrors;
    try {
      const drifted = await refreshSource($project.path, sourceId);
      driftedIdsBySource.update((prev) => {
        const next = new Map(prev);
        next.set(sourceId, drifted);
        return next;
      });
      openApiSources.set(await listOpenApiSources($project.path));
      const addSynced = new Set(syncedSources);
      addSynced.add(sourceId);
      syncedSources = addSynced;
      setTimeout(() => {
        const removeSynced = new Set(syncedSources);
        removeSynced.delete(sourceId);
        syncedSources = removeSynced;
      }, 2000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      const errMap = new Map(syncErrors);
      errMap.set(sourceId, msg);
      syncErrors = errMap;
    } finally {
      const removeLoading = new Set(loadingIds);
      removeLoading.delete(sourceId);
      loadingIds = removeLoading;
    }
  }

  async function handleSyncAll() {
    if (!$project.path) return;
    await Promise.all($openApiSources.map((s) => handleRefresh(s.id)));
  }

  async function confirmDelete() {
    const sourceId = pendingDeleteId;
    if (!sourceId || !$project.path) {
      pendingDeleteId = null;
      return;
    }
    pendingDeleteId = null;
    const addDeleting = new Set(deletingIds);
    addDeleting.add(sourceId);
    deletingIds = addDeleting;
    try {
      await removeOpenApiSource($project.path, sourceId);
      openApiSources.update((prev) => prev.filter((s) => s.id !== sourceId));
      driftedIdsBySource.update((prev) => {
        const next = new Map(prev);
        next.delete(sourceId);
        return next;
      });
    } catch (e) {
      console.error("Failed to remove source:", e);
    } finally {
      const removeDeleting = new Set(deletingIds);
      removeDeleting.delete(sourceId);
      deletingIds = removeDeleting;
    }
  }
</script>

<div class="flex flex-col gap-3">
  <div class="flex items-center justify-between">
    <span class="text-xs text-app-text-3 uppercase tracking-wider"
      >OpenAPI Sources</span
    >
    <div class="flex gap-2">
      <button
        class="px-2 py-1 text-xs bg-app-card hover:bg-app-hover text-app-text-2 rounded transition-colors"
        onclick={handleSyncAll}
        title="Re-fetch all sources and update drift status"
      >
        Sync All
      </button>
      <button
        class="px-2 py-1 text-xs bg-cyan-600 hover:bg-cyan-500 text-white rounded transition-colors"
        onclick={onAddSource}
      >
        + Add Source
      </button>
    </div>
  </div>

  {#if $openApiSources.length === 0}
    <EmptyState
      message="No sources yet"
      description="Add a URL or file path to start importing endpoints as requests"
      centered
    />
  {/if}

  {#each $openApiSources as source (source.id)}
    {@const loading = loadingIds.has(source.id)}
    {@const drift = $driftedIdsBySource.get(source.id)?.length ?? 0}
    {@const synced = syncedSources.has(source.id)}
    {@const syncError = syncErrors.get(source.id) ?? null}
    {@const deleting = deletingIds.has(source.id)}
    <div
      class="group/card bg-app-panel border {source.id === addedSourceId ||
      source.id === selectedSourceId
        ? 'border-cyan-700'
        : 'border-app-border'} rounded-lg p-3 flex flex-col gap-2 transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-cyan-600"
      role="button"
      tabindex="0"
      onclick={() => onSelectSource?.(source.id)}
      onkeydown={(e) => e.key === "Enter" && onSelectSource?.(source.id)}
    >
      <div class="flex items-start justify-between gap-2">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <span
              class="text-sm font-semibold text-app-text truncate"
              title={source.name}>{source.name}</span
            >
            {#if drift > 0}
              <span
                class="px-1 py-0 text-[10px] leading-4 bg-red-900/70 text-red-300 rounded shrink-0"
                title="{drift} endpoint{drift === 1
                  ? ''
                  : 's'} changed since last import">{drift} drifted</span
              >
            {/if}
          </div>
          <p
            class="font-mono text-xs text-app-text-3 truncate mt-0.5"
            title={source.type === "url" ? source.url : source.path}
          >
            {source.type === "url" ? source.url : source.path}
          </p>
          <p
            class="text-xs text-app-text-4 mt-0.5"
            title={source.lastFetchedAt
              ? new Date(source.lastFetchedAt).toLocaleString()
              : undefined}
          >
            {formatRelativeTime(source.lastFetchedAt)}
          </p>
          {#if syncError}
            <p class="text-xs text-red-400 mt-1 break-all">
              Sync failed: {syncError}
            </p>
          {/if}
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <button
            class="px-2 py-1 text-xs bg-app-card hover:bg-app-hover text-app-text-2 rounded transition-colors"
            onclick={(e) => {
              e.stopPropagation();
              onImport(source.id);
            }}
            title="Choose endpoints to add to your request collection"
          >
            Import
          </button>
          <button
            class="px-2 py-1 text-xs bg-app-card hover:bg-app-hover {synced
              ? 'text-green-400'
              : 'text-cyan-400'} rounded transition-colors disabled:opacity-50"
            disabled={loading}
            onclick={(e) => {
              e.stopPropagation();
              handleRefresh(source.id);
            }}
            title="Re-fetch the spec and check for changes"
          >
            {loading ? "…" : synced ? "Synced ✓" : "Sync"}
          </button>
          <button
            class="px-2 py-1 text-xs text-app-text-4 opacity-0 group-hover/card:opacity-60 hover:opacity-100! hover:bg-red-900/30 hover:text-red-300 rounded transition-all disabled:opacity-50"
            disabled={deleting}
            onclick={(e) => {
              e.stopPropagation();
              pendingDeleteId = source.id;
            }}
            aria-label="Remove source"
          >
            ✕
          </button>
        </div>
      </div>
    </div>
  {/each}
</div>

{#if pendingDeleteId !== null}
  {@const pendingSource = $openApiSources.find((s) => s.id === pendingDeleteId)}
  <ConfirmDialog
    message={`Remove "${pendingSource?.name ?? "this source"}"?`}
    detail="All imported requests from this source will remain, but the source will no longer sync."
    confirmLabel="Remove source"
    onConfirm={confirmDelete}
    onCancel={() => {
      pendingDeleteId = null;
    }}
  />
{/if}
