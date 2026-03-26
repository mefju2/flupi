<script lang="ts">
  import type { ScenarioStep, StepResult } from '$lib/services/tauri-commands';

  interface Props {
    step: ScenarioStep;
    result: StepResult;
  }

  let { step, result }: Props = $props();

  let expanded = $state(false);
  let isSuccess = $derived(result.status === 'success');
  let statusCode = $derived(result.response?.status);
  let duration = $derived(result.response?.durationMs);
  let extractedEntries = $derived(Object.entries(result.extracted ?? {}));
</script>

<div class="border border-zinc-800 rounded bg-zinc-900">
  <div
    class="flex items-center gap-3 px-3 py-2 cursor-pointer select-none"
    role="button" tabindex="0"
    onclick={() => expanded = !expanded}
    onkeydown={(e) => e.key === 'Enter' && (expanded = !expanded)}
  >
    {#if isSuccess}
      <span class="text-green-400 text-sm shrink-0">✓</span>
    {:else}
      <span class="text-red-400 text-sm shrink-0">✗</span>
    {/if}

    <span class="text-sm text-zinc-200 flex-1 truncate">{step.name}</span>

    <div class="flex items-center gap-2 shrink-0">
      {#if statusCode !== undefined}
        <span class="font-mono text-xs px-1.5 py-0.5 rounded
          {statusCode >= 200 && statusCode < 300 ? 'text-green-400 bg-green-950/40' : 'text-red-400 bg-red-950/40'}">
          {statusCode}
        </span>
      {/if}
      {#if duration !== undefined}
        <span class="text-xs text-zinc-500">{duration}ms</span>
      {/if}
      {#if extractedEntries.length > 0}
        {#each extractedEntries as [key]}
          <span class="text-xs text-cyan-400 bg-cyan-950/40 border border-cyan-800/60 rounded px-1.5 py-0.5">{key}</span>
        {/each}
      {/if}
      <span class="text-zinc-500 text-xs">{expanded ? '▾' : '▸'}</span>
    </div>
  </div>

  {#if expanded}
    <div class="border-t border-zinc-800 px-3 py-3 space-y-3">
      {#if result.error}
        <div>
          <p class="text-xs text-zinc-500 mb-1">Error</p>
          <pre class="font-mono text-xs text-red-400 bg-zinc-800 rounded p-2 overflow-x-auto">{result.error}</pre>
        </div>
      {/if}

      {#if extractedEntries.length > 0}
        <div>
          <p class="text-xs text-zinc-500 mb-1">Extracted Variables</p>
          <div class="space-y-0.5">
            {#each extractedEntries as [key, value]}
              <div class="flex gap-2">
                <span class="font-mono text-xs text-zinc-400 min-w-24">{key}</span>
                <span class="font-mono text-xs text-zinc-200">{value}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if result.response}
        <div>
          <p class="text-xs text-zinc-500 mb-1">Response Body</p>
          <pre class="font-mono text-xs text-zinc-300 bg-zinc-800 rounded p-2 overflow-x-auto max-h-48">{result.response.body}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>
