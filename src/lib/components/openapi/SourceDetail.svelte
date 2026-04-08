<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { project } from "$lib/stores/project";
  import { openApiSources, driftedIdsBySource } from "$lib/stores/openapi";
  import { activeRequestId, activeRequest } from "$lib/stores/requests";
  import {
    renameOpenApiSource,
    removeOpenApiSource,
    listRequestsBySource,
    getRequest,
    type SourceRequest,
  } from "$lib/services/tauri-commands";
  import SourceRequestList from "$lib/components/openapi/SourceRequestList.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { formatRelativeTime } from "$lib/utils/format";

  interface Props {
    sourceId: string;
    onDeleted?: () => void;
  }

  let { sourceId, onDeleted }: Props = $props();

  let source = $derived($openApiSources.find((s) => s.id === sourceId));
  let driftedIds = $derived($driftedIdsBySource.get(sourceId) ?? []);

  let importedRequests = $state<SourceRequest[]>([]);
  let loadingRequests = $state(false);
  let loadError = $state<string | null>(null);

  let renamingSource = $state(false);
  let renameValue = $state("");
  let renameInputEl = $state<HTMLInputElement | undefined>(undefined);
  let confirmingDelete = $state(false);

  $effect(() => {
    void sourceId;
    let cancelled = false;
    loadRequests(() => cancelled);
    return () => { cancelled = true; };
  });

  async function loadRequests(isCancelled: () => boolean) {
    if (!$project.path) return;
    loadingRequests = true;
    loadError = null;
    try {
      const result = await listRequestsBySource($project.path, sourceId);
      if (!isCancelled()) {
        importedRequests = result;
      }
    } catch (e) {
      if (!isCancelled()) {
        loadError = "Failed to load requests";
        console.error("Failed to load requests for source:", e);
      }
    } finally {
      loadingRequests = false;
    }
  }

  function startRename() {
    if (!source) return;
    renameValue = source.name;
    renamingSource = true;
    setTimeout(() => renameInputEl?.focus(), 0);
  }

  function cancelRename() {
    renamingSource = false;
    renameValue = "";
  }

  async function confirmRename() {
    if (!$project.path || !source) {
      cancelRename();
      return;
    }
    const trimmed = renameValue.trim();
    if (!trimmed || trimmed === source.name) {
      cancelRename();
      return;
    }
    renamingSource = false;
    try {
      await renameOpenApiSource($project.path, sourceId, trimmed);
      openApiSources.update((list) =>
        list.map((s) => (s.id === sourceId ? { ...s, name: trimmed } : s)),
      );
    } catch (e) {
      console.error("Failed to rename source:", e);
    }
  }

  async function handleDelete() {
    if (!$project.path) return;
    confirmingDelete = false;
    try {
      await removeOpenApiSource($project.path, sourceId);
      openApiSources.update((prev) => prev.filter((s) => s.id !== sourceId));
      driftedIdsBySource.update((prev) => {
        const next = new Map(prev);
        next.delete(sourceId);
        return next;
      });
      onDeleted?.();
    } catch (e) {
      console.error("Failed to remove source:", e);
    }
  }

  async function navigateToRequest(requestId: string) {
    if (!$project.path) return;
    try {
      const req = await getRequest($project.path, requestId);
      activeRequestId.set(requestId);
      activeRequest.set(req);
      await goto("/requests");
    } catch (e) {
      console.error("Failed to navigate to request:", e);
    }
  }
</script>

<div class="flex flex-col h-full overflow-y-auto p-6 gap-6">
  {#if source}
    <!-- Source header -->
    <div class="flex flex-col gap-3">
      <div class="flex items-start justify-between gap-3">
        <div class="flex-1 min-w-0">
          {#if renamingSource}
            <input
              bind:this={renameInputEl}
              bind:value={renameValue}
              class="w-full bg-app-card text-app-text text-base font-semibold px-2 py-0.5 rounded outline-none border border-app-border-2 focus:border-cyan-500"
              onkeydown={(e) => {
                if (e.key === "Enter") confirmRename();
                else if (e.key === "Escape") cancelRename();
              }}
              onblur={confirmRename}
            />
          {:else}
            <div class="group/name flex items-center gap-1.5 min-w-0">
              <h2
                class="text-base font-semibold text-app-text cursor-text truncate"
                ondblclick={startRename}
              >
                {source.name}
              </h2>
              <span
                class="opacity-0 group-hover/name:opacity-100 transition-opacity text-app-text-4 text-[11px] select-none shrink-0"
                aria-hidden="true"
                title="Double-click name to rename"
              >✎</span>
            </div>
          {/if}
          <p class="font-mono text-xs text-app-text-3 mt-1 break-all">
            {source.type === "url" ? source.url : source.path}
          </p>
        </div>
        <div class="flex items-center gap-1.5 shrink-0">
          <button
            class="text-xs text-app-text-3 hover:text-app-text border border-app-border rounded px-2 py-1 transition-colors"
            onclick={startRename}
          >Rename</button>
          <button
            class="text-xs text-app-text-3 hover:text-red-300 hover:border-red-800 border border-app-border rounded px-2 py-1 transition-colors"
            onclick={() => { confirmingDelete = true; }}
          >Delete</button>
        </div>
      </div>

      <div class="flex items-center gap-3 text-xs text-app-text-3">
        <span>Type: <span class="text-app-text font-mono">{source.type === "url" ? "URL" : "File"}</span></span>
        <span class="text-app-text-4">·</span>
        <span
          title={source.lastFetchedAt
            ? new Date(source.lastFetchedAt).toLocaleString()
            : undefined}
        >Last synced: <span class="text-app-text">{formatRelativeTime(source.lastFetchedAt)}</span></span>
      </div>
    </div>

    <!-- Imported requests -->
    <SourceRequestList
      requests={importedRequests}
      loading={loadingRequests}
      error={loadError}
      {driftedIds}
      onNavigate={navigateToRequest}
    />
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-app-text-4 text-sm">Source not found.</p>
    </div>
  {/if}
</div>

{#if confirmingDelete && source}
  <ConfirmDialog
    message={`Delete "${source.name}"?`}
    detail="All imported requests from this source will remain, but the source will no longer sync."
    confirmLabel="Delete source"
    onConfirm={handleDelete}
    onCancel={() => { confirmingDelete = false; }}
  />
{/if}
