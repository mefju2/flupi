<script lang="ts">
  import { project } from '$lib/stores/project';
  import { openApiSources, driftedRequestIds } from '$lib/stores/openapi';
  import { removeOpenApiSource, refreshSource, listOpenApiSources } from '$lib/services/tauri-commands';

  interface Props {
    onAddSource: () => void;
    onImport: (sourceId: string) => void;
  }

  let { onAddSource, onImport }: Props = $props();

  let loadingIds = $state<Set<string>>(new Set());

  function driftCount(sourceId: string): number {
    let count = 0;
    $driftedRequestIds.forEach((id) => {
      if (id.startsWith(sourceId + '/') || id.includes(sourceId)) count++;
    });
    return count;
  }

  async function handleRefresh(sourceId: string) {
    if (!$project.path) return;
    loadingIds = new Set([...loadingIds, sourceId]);
    try {
      const drifted = await refreshSource($project.path, sourceId);
      driftedRequestIds.update((prev) => {
        const next = new Set(prev);
        drifted.forEach((id) => next.add(id));
        return next;
      });
      openApiSources.set(await listOpenApiSources($project.path));
    } catch (e) {
      console.error('Failed to refresh source:', e);
    } finally {
      loadingIds = new Set([...loadingIds].filter((id) => id !== sourceId));
    }
  }

  async function handleSyncAll() {
    if (!$project.path) return;
    for (const source of $openApiSources) {
      await handleRefresh(source.id);
    }
  }

  async function handleDelete(sourceId: string) {
    if (!$project.path || !confirm('Remove this OpenAPI source?')) return;
    try {
      await removeOpenApiSource($project.path, sourceId);
      openApiSources.update((prev) => prev.filter((s) => s.id !== sourceId));
    } catch (e) {
      console.error('Failed to remove source:', e);
    }
  }

  function formatDate(iso: string | null): string {
    if (!iso) return 'Never';
    return new Date(iso).toLocaleString();
  }
</script>

<div class="flex flex-col gap-3">
  <div class="flex items-center justify-between">
    <span class="text-xs text-zinc-500 uppercase tracking-wider">OpenAPI Sources</span>
    <div class="flex gap-2">
      <button
        class="px-2 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded transition-colors"
        onclick={handleSyncAll}
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
    <p class="text-sm text-zinc-600 py-4">No sources registered. Add one to import API operations.</p>
  {/if}

  {#each $openApiSources as source (source.id)}
    {@const loading = loadingIds.has(source.id)}
    {@const drift = driftCount(source.id)}
    <div class="bg-zinc-900 border border-zinc-800 rounded-lg p-3 flex flex-col gap-2">
      <div class="flex items-start justify-between gap-2">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-semibold text-zinc-100 truncate">{source.name}</span>
            {#if drift > 0}
              <span class="px-1.5 py-0.5 text-xs bg-red-900 text-red-300 rounded-full">{drift} drifted</span>
            {/if}
          </div>
          <p class="font-mono text-xs text-zinc-500 truncate mt-0.5">
            {source.type === 'url' ? source.url : source.path}
          </p>
          <p class="text-xs text-zinc-600 mt-0.5">Last synced: {formatDate(source.lastFetchedAt)}</p>
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <button
            class="px-2 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded transition-colors disabled:opacity-50"
            onclick={() => onImport(source.id)}
          >
            Import
          </button>
          <button
            class="px-2 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-cyan-400 rounded transition-colors disabled:opacity-50"
            disabled={loading}
            onclick={() => handleRefresh(source.id)}
          >
            {loading ? '…' : 'Sync'}
          </button>
          <button
            class="px-2 py-1 text-xs bg-zinc-800 hover:bg-red-900 text-zinc-500 hover:text-red-300 rounded transition-colors"
            onclick={() => handleDelete(source.id)}
          >
            ✕
          </button>
        </div>
      </div>
    </div>
  {/each}
</div>
