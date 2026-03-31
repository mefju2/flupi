<script lang="ts">
  import { tryParseJson } from '$lib/utils/format';
  import type { ScenarioStep, StepResult } from '$lib/services/tauri-commands';

  interface Props {
    step: ScenarioStep;
    result: StepResult;
  }

  let { step, result }: Props = $props();

  let expanded = $state(false);
  let bodyExpanded = $state(false);
  let expandedExtracted = $state(new Set<string>());

  function toggleExtracted(key: string) {
    const next = new Set(expandedExtracted);
    if (next.has(key)) next.delete(key); else next.add(key);
    expandedExtracted = next;
  }
  let isSuccess = $derived(result.status === 'success');
  let statusCode = $derived(result.response?.status);
  let duration = $derived(result.response?.durationMs);
  let extractedEntries = $derived(Object.entries(result.extracted ?? {}));

  function bodySummary(parsed: unknown, raw: string): string {
    if (parsed === null || parsed === undefined) return raw.slice(0, 120) + (raw.length > 120 ? '…' : '');
    if (Array.isArray(parsed)) return `[ ${parsed.length} item${parsed.length === 1 ? '' : 's'} ]`;
    if (typeof parsed === 'object') {
      const keys = Object.keys(parsed as object);
      return `{ ${keys.slice(0, 4).join(', ')}${keys.length > 4 ? `, … +${keys.length - 4} more` : ''} }`;
    }
    return String(parsed).slice(0, 120);
  }
</script>

<div class="border border-app-border rounded bg-app-panel">
  <div
    class="flex items-center gap-3 px-3 py-2 cursor-pointer select-none"
    role="button" tabindex="0"
    onclick={() => { expanded = !expanded; if (!expanded) { bodyExpanded = false; expandedExtracted = new Set(); } }}
    onkeydown={(e) => { if (e.key === 'Enter') { expanded = !expanded; if (!expanded) { bodyExpanded = false; expandedExtracted = new Set(); } } }}
  >
    {#if isSuccess}
      <span class="text-green-400 text-sm shrink-0">✓</span>
    {:else}
      <span class="text-red-400 text-sm shrink-0">✗</span>
    {/if}

    <span class="text-sm text-app-text flex-1 truncate">{step.name}</span>

    <div class="flex items-center gap-2 shrink-0">
      {#if statusCode !== undefined}
        <span class="font-mono text-xs px-1.5 py-0.5 rounded
          {statusCode >= 200 && statusCode < 300 ? 'text-green-400 bg-green-950/40' : 'text-red-400 bg-red-950/40'}">
          {statusCode}
        </span>
      {/if}
      {#if duration !== undefined}
        <span class="text-xs text-app-text-3">{duration}ms</span>
      {/if}
      {#if extractedEntries.length > 0}
        {#each extractedEntries.slice(0, 3) as [key]}
          <span class="text-xs text-cyan-400 bg-cyan-950/40 border border-cyan-800/60 rounded px-1.5 py-0.5">{key}</span>
        {/each}
        {#if extractedEntries.length > 3}
          <span class="text-xs text-app-text-3">+{extractedEntries.length - 3} more</span>
        {/if}
      {/if}
      <span class="text-app-text-3 text-xs">{expanded ? '▾' : '▸'}</span>
    </div>
  </div>

  {#if expanded}
    <div class="border-t border-app-border px-3 py-3 space-y-3">
      {#if result.error}
        <div>
          <p class="text-xs text-app-text-3 mb-1">Error</p>
          <pre class="font-mono text-xs text-red-400 bg-app-card rounded p-2 overflow-x-auto">{result.error}</pre>
        </div>
      {/if}

      {#if extractedEntries.length > 0}
        <div>
          <p class="text-xs text-app-text-3 mb-1">Extracted Variables</p>
          <div class="space-y-1">
            {#each extractedEntries as [key, value]}
              {@const parsedVal = tryParseJson(value)}
              {@const isJson = parsedVal !== null && typeof parsedVal === 'object'}
              <div class="flex gap-2 items-start">
                <span class="font-mono text-xs text-app-text-3 min-w-24 shrink-0 pt-0.5">{key}</span>
                {#if isJson}
                  <div class="flex-1 min-w-0">
                    {#if expandedExtracted.has(key)}
                      <div class="flex items-start gap-2">
                        <pre class="font-mono text-xs text-app-text-2 bg-app-card rounded p-2 overflow-auto max-h-64 flex-1 whitespace-pre-wrap break-all min-w-0">{JSON.stringify(parsedVal, null, 2)}</pre>
                        <button
                          class="text-xs text-app-text-4 hover:text-app-text-2 transition-colors shrink-0 mt-1"
                          onclick={() => toggleExtracted(key)}
                        >Collapse</button>
                      </div>
                    {:else}
                      <button
                        class="w-full text-left font-mono text-xs text-app-text-3 bg-app-card rounded px-2 py-0.5 hover:text-app-text-2 transition-colors truncate"
                        onclick={() => toggleExtracted(key)}
                      >{bodySummary(parsedVal, value)}</button>
                    {/if}
                  </div>
                {:else}
                  <span class="font-mono text-xs text-app-text">{value}</span>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if result.response}
        {@const raw = result.response.body}
        {@const parsed = tryParseJson(raw)}
        {@const pretty = parsed !== null ? JSON.stringify(parsed, null, 2) : raw}
        <div>
          <div class="flex items-center justify-between mb-1">
            <p class="text-xs text-app-text-3">Response Body</p>
            <button
              class="text-xs text-app-text-4 hover:text-app-text-2 transition-colors"
              onclick={() => bodyExpanded = !bodyExpanded}
            >{bodyExpanded ? 'Collapse' : 'Expand'}</button>
          </div>
          {#if bodyExpanded}
            <pre class="font-mono text-xs text-app-text-2 bg-app-card rounded p-2 overflow-auto max-h-96 whitespace-pre-wrap break-all">{pretty}</pre>
          {:else}
            <button
              class="w-full text-left font-mono text-xs text-app-text-3 bg-app-card rounded p-2 hover:text-app-text-2 transition-colors truncate"
              onclick={() => bodyExpanded = true}
            >{bodySummary(parsed, raw)}</button>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>
