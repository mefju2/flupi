<script lang="ts">
  import type { SourceRequest } from "$lib/services/tauri-commands";

  const METHOD_COLORS: Record<string, string> = {
    GET: "text-green-400",
    POST: "text-cyan-400",
    PUT: "text-yellow-400",
    PATCH: "text-orange-400",
    DELETE: "text-red-400",
    HEAD: "text-purple-400",
    OPTIONS: "text-app-text-3",
  };

  interface Props {
    requests: SourceRequest[];
    loading: boolean;
    error: string | null;
    onNavigate: (id: string) => void;
  }

  let { requests, loading, error, onNavigate }: Props = $props();
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
      No requests imported from this source yet. Use Import to add requests.
    </p>
  {:else}
    <div class="flex flex-col gap-1">
      {#each requests as req (req.id)}
        <button
          class="flex items-center gap-2 px-3 py-2 rounded bg-app-panel border border-app-border hover:border-cyan-700 hover:bg-app-card text-left transition-colors group"
          onclick={() => onNavigate(req.id)}
        >
          <span
            class="font-mono text-xs font-semibold w-14 shrink-0 {METHOD_COLORS[req.method.toUpperCase()] ?? 'text-app-text-3'}"
          >
            {req.method.toUpperCase()}
          </span>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-app-text truncate">{req.name}</p>
            <p class="font-mono text-xs text-app-text-4 truncate">{req.path}</p>
          </div>
          <span class="text-app-text-4 group-hover:text-app-text-2 text-xs shrink-0 transition-colors">→</span>
        </button>
      {/each}
    </div>
  {/if}
</div>
