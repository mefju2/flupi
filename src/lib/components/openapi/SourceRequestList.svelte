<script lang="ts">
  import type { SourceRequest } from "$lib/services/tauri-commands";
  import { getMethodColor } from "$lib/utils/format";

  interface Props {
    requests: SourceRequest[];
    loading: boolean;
    error: string | null;
    driftedIds?: string[];
    onNavigate: (id: string) => void;
  }

  let { requests, loading, error, driftedIds = [], onNavigate }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  <p class="text-xs text-app-text-3 uppercase tracking-wider">
    Imported Requests
    {#if requests.length > 0}
      <span class="ml-1 text-app-text-4 normal-case tracking-normal">({requests.length})</span>
    {/if}
  </p>

  {#if loading}
    <p class="text-xs text-app-text-4">Loading…</p>
  {:else if error}
    <p class="text-xs text-red-400">{error}</p>
  {:else if requests.length === 0}
    <p class="text-xs text-app-text-4">
      No requests yet. Use the Import button on the source card to add endpoints.
    </p>
  {:else}
    <div class="flex flex-col gap-1">
      {#each requests as req (req.id)}
        {@const isDrifted = driftedIds.includes(req.id)}
        <button
          class="flex items-center gap-2 px-3 py-2 rounded border text-left transition-colors group {isDrifted
            ? 'bg-red-950/20 border-red-900/50 hover:border-red-700'
            : 'bg-app-panel border-app-border hover:border-cyan-700 hover:bg-app-card'}"
          onclick={() => onNavigate(req.id)}
          title={isDrifted ? "This endpoint changed since last import" : undefined}
        >
          <span
            class="font-mono text-xs font-semibold w-14 shrink-0 {getMethodColor(req.method)}"
          >
            {req.method.toUpperCase()}
          </span>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-app-text truncate">{req.name}</p>
            <p class="font-mono text-xs text-app-text-4 truncate">{req.path}</p>
          </div>
          {#if isDrifted}
            <span class="px-1 py-0 text-[10px] leading-4 bg-red-900/70 text-red-300 rounded shrink-0">drifted</span>
          {/if}
          <span class="text-app-text-4 group-hover:text-app-text-2 text-xs shrink-0 transition-colors">→</span>
        </button>
      {/each}
    </div>
  {/if}
</div>
