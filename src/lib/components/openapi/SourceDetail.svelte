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

  const METHOD_COLORS: Record<string, string> = {
    GET: "text-green-400",
    POST: "text-cyan-400",
    PUT: "text-yellow-400",
    PATCH: "text-orange-400",
    DELETE: "text-red-400",
    HEAD: "text-purple-400",
    OPTIONS: "text-app-text-3",
  };
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
    <div class="flex flex-col gap-2">
      <p class="text-xs text-app-text-3 uppercase tracking-wider">
        Imported Requests
        {#if importedRequests.length > 0}
          <span class="ml-1 text-app-text-4 normal-case tracking-normal"
            >({importedRequests.length})</span
          >
        {/if}
      </p>

      {#if loadingRequests}
        <p class="text-xs text-app-text-4">Loading…</p>
      {:else if loadError}
        <p class="text-xs text-red-400">{loadError}</p>
      {:else if importedRequests.length === 0}
        <p class="text-xs text-app-text-4">
          No requests imported from this source yet. Use Import to add requests.
        </p>
      {:else}
        <div class="flex flex-col gap-1">
          {#each importedRequests as req (req.id)}
            <button
              class="flex items-center gap-2 px-3 py-2 rounded bg-app-panel border border-app-border hover:border-cyan-700 hover:bg-app-card text-left transition-colors group"
              onclick={() => navigateToRequest(req.id)}
            >
              <span
                class="font-mono text-xs font-semibold w-14 shrink-0 {METHOD_COLORS[
                  req.method.toUpperCase()
                ] ?? 'text-app-text-3'}"
              >
                {req.method.toUpperCase()}
              </span>
              <div class="flex-1 min-w-0">
                <p class="text-xs text-app-text truncate">{req.name}</p>
                <p class="font-mono text-xs text-app-text-4 truncate">
                  {req.path}
                </p>
              </div>
              <span
                class="text-app-text-4 group-hover:text-app-text-2 text-xs shrink-0 transition-colors"
                >→</span
              >
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-app-text-4 text-sm">Source not found.</p>
    </div>
  {/if}
</div>
