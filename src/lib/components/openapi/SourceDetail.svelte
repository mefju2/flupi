<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { project } from "$lib/stores/project";
  import { openApiSources } from "$lib/stores/openapi";
  import { activeRequestId, activeRequest } from "$lib/stores/requests";
  import {
    renameOpenApiSource,
    listRequestsBySource,
    getRequest,
    type SourceRequest,
  } from "$lib/services/tauri-commands";
  import SourceRequestList from "$lib/components/openapi/SourceRequestList.svelte";

  interface Props {
    sourceId: string;
  }

  let { sourceId }: Props = $props();

  let source = $derived($openApiSources.find((s) => s.id === sourceId));

  let importedRequests = $state<SourceRequest[]>([]);
  let loadingRequests = $state(false);
  let loadError = $state<string | null>(null);

  let renamingSource = $state(false);
  let renameValue = $state("");
  let renameInputEl = $state<HTMLInputElement | undefined>(undefined);

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
      if (!isCancelled()) {
        loadingRequests = false;
      }
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

  function formatDate(iso: string | null): string {
    if (!iso) return "Never";
    return new Date(iso).toLocaleString();
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
            <h2
              class="text-base font-semibold text-app-text cursor-text"
              ondblclick={startRename}
              title="Double-click to rename"
            >
              {source.name}
            </h2>
          {/if}
          <p class="font-mono text-xs text-app-text-3 mt-1 break-all">
            {source.type === "url" ? source.url : source.path}
          </p>
        </div>
        <button
          class="shrink-0 text-xs text-app-text-3 hover:text-app-text border border-app-border rounded px-2 py-1 transition-colors"
          onclick={startRename}
          title="Rename source">Rename</button
        >
      </div>

      <div class="grid grid-cols-2 gap-2 text-xs">
        <div class="bg-app-panel rounded p-2">
          <p class="text-app-text-4 mb-0.5">Type</p>
          <p class="text-app-text font-mono">
            {source.type === "url" ? "URL" : "File"}
          </p>
        </div>
        <div class="bg-app-panel rounded p-2">
          <p class="text-app-text-4 mb-0.5">Last Synced</p>
          <p class="text-app-text">{formatDate(source.lastFetchedAt)}</p>
        </div>
      </div>
    </div>

    <!-- Imported requests -->
    <SourceRequestList
      requests={importedRequests}
      loading={loadingRequests}
      error={loadError}
      onNavigate={navigateToRequest}
    />
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-app-text-4 text-sm">Source not found.</p>
    </div>
  {/if}
</div>
