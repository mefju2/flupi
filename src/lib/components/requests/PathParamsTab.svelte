<script lang="ts">
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';

  interface Props {
    path: string;
    pathParams: Record<string, string>;
    onPathParamsChange: (params: Record<string, string>) => void;
  }

  let { path, pathParams, onPathParamsChange }: Props = $props();

  function parsePathParams(p: string): string[] {
    const matches = [...p.matchAll(/(?<!\{)\{([a-zA-Z0-9_-]+)\}(?!\})/g)];
    const seen = new Set<string>();
    const result: string[] = [];
    for (const m of matches) {
      if (!seen.has(m[1])) {
        seen.add(m[1]);
        result.push(m[1]);
      }
    }
    return result;
  }

  let detectedParams = $derived(parsePathParams(path));

  function handleValueChange(param: string, value: string) {
    onPathParamsChange({ ...pathParams, [param]: value });
  }
</script>

<div class="p-4 h-full overflow-y-auto">
  {#if detectedParams.length === 0}
    <p class="text-xs text-app-text-3">
      Add <span class="font-mono">{'{paramName}'}</span> segments to your URL path to define path parameters.
    </p>
  {:else}
    <p class="text-xs text-app-text-3 mb-3">Path parameters are substituted into the URL at execution time.</p>
    <div class="space-y-1">
      {#each detectedParams as param (param)}
        <div class="flex gap-2 items-center">
          <span class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text-3">
            {'{' + param + '}'}
          </span>
          <VariableAutocomplete
            class="flex-1"
            value={pathParams[param] ?? ''}
            placeholder={'{{' + param + '}}'}
            onChange={(v) => handleValueChange(param, v)}
          />
        </div>
      {/each}
    </div>
  {/if}
</div>
