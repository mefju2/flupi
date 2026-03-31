<script lang="ts">
  import { tryParseJson } from '$lib/utils/format';

  interface Props {
    variables: Record<string, string>;
    secretKeys?: string[];
  }

  let { variables, secretKeys = [] }: Props = $props();

  let entries = $derived(Object.entries(variables));
  let expandedKeys = $state(new Set<string>());

  function toggleKey(key: string) {
    const next = new Set(expandedKeys);
    if (next.has(key)) next.delete(key); else next.add(key);
    expandedKeys = next;
  }

</script>

<div class="space-y-1">
  {#if entries.length === 0}
    <p class="text-xs text-app-text-4 italic">No variables in context.</p>
  {:else}
    {#each entries as [key, value]}
      {@const isSecret = secretKeys.includes(key)}
      {@const parsed = isSecret ? null : tryParseJson(value)}
      {@const isJson = parsed !== null && typeof parsed === 'object'}
      <div class="py-0.5">
        {#if isJson}
          <!-- header row: key + toggle button -->
          <div class="flex items-center gap-2">
            <span class="font-mono text-xs text-app-text-3 shrink-0">{key}</span>
            <button
              class="text-xs text-app-text-4 hover:text-app-text-2 transition-colors font-mono"
              onclick={() => toggleKey(key)}
            >
              {#if expandedKeys.has(key)}
                {'{ json }'} · Collapse
              {:else}
                {'{ json }'}
              {/if}
            </button>
          </div>
          <!-- expanded json below -->
          {#if expandedKeys.has(key)}
            <pre class="mt-1 font-mono text-xs text-app-text-2 bg-app-card rounded p-2 overflow-auto max-h-64 whitespace-pre-wrap wrap-break-word">{JSON.stringify(parsed, null, 2)}</pre>
          {/if}
        {:else}
          <div class="flex items-baseline gap-2">
            <span class="font-mono text-xs text-app-text-3 shrink-0">{key}</span>
            <span class="font-mono text-xs text-app-text wrap-break-word min-w-0">
              {isSecret ? '••••••' : value}
            </span>
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>
