<script lang="ts">
  import { lastResponse, isExecuting } from '$lib/stores/execution';

  let headersOpen = $state(false);

  function statusClass(status: number): string {
    if (status >= 200 && status < 300) return 'bg-emerald-900 text-emerald-300 border-emerald-700';
    if (status >= 400 && status < 500) return 'bg-yellow-900 text-yellow-300 border-yellow-700';
    if (status >= 500) return 'bg-red-900 text-red-300 border-red-700';
    return 'bg-zinc-800 text-zinc-300 border-zinc-700';
  }

  const MAX_BODY_BYTES = 1_048_576; // 1MB

  function formatBody(raw: string): string {
    if (raw.length > MAX_BODY_BYTES) {
      return raw.slice(0, MAX_BODY_BYTES) + '\n\n[Response truncated at 1MB]';
    }
    try {
      return JSON.stringify(JSON.parse(raw), null, 2);
    } catch {
      return raw;
    }
  }
</script>

<div class="border-t border-zinc-800 bg-zinc-950 min-h-[160px]">
  {#if $isExecuting}
    <div class="p-6 text-sm text-zinc-500">Sending...</div>
  {:else if !$lastResponse}
    <div class="p-6 text-sm text-zinc-600">Send a request to see the response.</div>
  {:else}
    <div class="p-4 space-y-3">
      <div class="flex items-center gap-3">
        <span class="text-xs font-mono px-2 py-0.5 rounded border {statusClass($lastResponse.status)}">
          {$lastResponse.status} {$lastResponse.statusText}
        </span>
        <span class="text-xs text-zinc-500">{$lastResponse.durationMs}ms</span>
      </div>

      <div>
        <button
          class="text-xs text-zinc-400 hover:text-zinc-200 transition-colors flex items-center gap-1 mb-1"
          onclick={() => (headersOpen = !headersOpen)}
        >
          <span>{headersOpen ? '▾' : '▸'}</span>
          Headers ({Object.keys($lastResponse.headers).length})
        </button>
        {#if headersOpen}
          <div class="space-y-0.5">
            {#each Object.entries($lastResponse.headers) as [k, v]}
              <div class="flex gap-2 text-xs font-mono">
                <span class="text-zinc-400 shrink-0">{k}:</span>
                <span class="text-zinc-300 break-all">{v}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div>
        <p class="text-xs text-zinc-500 mb-1">Body</p>
        <pre class="text-xs font-mono text-zinc-200 bg-zinc-900 border border-zinc-800 p-3 overflow-auto max-h-[300px] whitespace-pre-wrap break-all">{formatBody($lastResponse.body)}</pre>
      </div>
    </div>
  {/if}
</div>
