<script lang="ts">
  import type { Extraction } from '$lib/services/tauri-commands';
  import SchemaAutocomplete from './SchemaAutocomplete.svelte';
  import EnvVarCombobox from '$lib/components/shared/EnvVarCombobox.svelte';
  import { buildJsonPathSuggestions } from '$lib/utils/schema-paths';
  import { environments, activeEnvironment } from '$lib/stores/environment';

  interface Props {
    extractions: Extraction[];
    onUpdate: (extractions: Extraction[]) => void;
    responseSchema?: unknown;
    mode?: 'request' | 'scenario';
  }

  let { extractions, onUpdate, responseSchema = null, mode = 'request' }: Props = $props();

  let suggestions = $derived(buildJsonPathSuggestions(responseSchema));

  const envVarNames = $derived.by(() => {
    const entry = $environments.find((e) => e.fileName === $activeEnvironment);
    if (!entry) return [];
    return [
      ...Object.keys(entry.environment.variables),
      ...entry.environment.secrets,
    ];
  });

  function addRow() {
    const scope = mode === 'scenario' ? 'scenario' : 'env';
    onUpdate([...extractions, { variable: '', from: 'response.body', path: '', scope }]);
  }

  function removeRow(i: number) {
    onUpdate(extractions.filter((_, idx) => idx !== i));
  }

  function updateRow(i: number, field: keyof Extraction, value: string) {
    const updated = [...extractions];
    updated[i] = { ...updated[i], [field]: value };
    onUpdate(updated);
  }

  function toggleScope(i: number) {
    const current = extractions[i].scope ?? 'env';
    updateRow(i, 'scope', current === 'env' ? 'scenario' : 'env');
  }
</script>

<div class="space-y-1">
  <p class="text-xs text-app-text-3 mb-2">Extract values from the response into variables. JSONPath: <span class="font-mono">$.field</span>, <span class="font-mono">$.array[0].value</span></p>

  {#if extractions.length > 0}
    {#if mode === 'scenario'}
      <div class="grid grid-cols-[auto_1fr_auto_1fr_auto] gap-2 mb-1">
        <span class="text-xs text-app-text-3 px-1">Scope</span>
        <span class="text-xs text-app-text-3 px-1">Variable</span>
        <span class="text-xs text-app-text-3 px-1">Source</span>
        <span class="text-xs text-app-text-3 px-1">Path (JSONPath)</span>
        <span></span>
      </div>
    {:else}
      <div class="grid grid-cols-[1fr_auto_1fr_auto] gap-2 mb-1">
        <span class="text-xs text-app-text-3 px-1">Variable</span>
        <span class="text-xs text-app-text-3 px-1">Source</span>
        <span class="text-xs text-app-text-3 px-1">Path (JSONPath)</span>
        <span></span>
      </div>
    {/if}
  {/if}

  {#each extractions as extraction, i}
    {#if mode === 'scenario'}
      <div class="grid grid-cols-[auto_1fr_auto_1fr_auto] gap-2 items-center">
        <button
          type="button"
          class="w-12 text-center text-[10px] font-sans font-medium px-1.5 py-0.5 rounded transition-colors {(extraction.scope ?? 'env') === 'scenario' ? 'text-violet-400 bg-violet-500/10 hover:bg-violet-500/20' : 'text-cyan-500 bg-cyan-500/10 hover:bg-cyan-500/20'}"
          onclick={() => toggleScope(i)}
          title={(extraction.scope ?? 'env') === 'scenario' ? 'local scope — click to change to env' : 'env scope — click to change to local'}
        >{(extraction.scope ?? 'env') === 'scenario' ? 'local' : 'env'}</button>

        {#if (extraction.scope ?? 'env') === 'scenario'}
          <input
            class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
            value={extraction.variable}
            placeholder="Variable name…"
            oninput={(e) => updateRow(i, 'variable', e.currentTarget.value)}
          />
        {:else}
          <EnvVarCombobox
            value={extraction.variable}
            onChange={(v) => updateRow(i, 'variable', v)}
            envVars={envVarNames}
          />
        {/if}

        <select
          class="bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text-2 focus:outline-none focus:border-app-border-2"
          value={extraction.from}
          onchange={(e) => updateRow(i, 'from', e.currentTarget.value)}
        >
          <option value="response.body">response.body</option>
          <option value="response.headers">response.headers</option>
        </select>

        {#if extraction.from === 'response.headers'}
          <input
            class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
            value={extraction.path}
            placeholder="Content-Type"
            oninput={(e) => updateRow(i, 'path', e.currentTarget.value)}
          />
        {:else}
          <SchemaAutocomplete
            {suggestions}
            value={extraction.path}
            placeholder="$.data.id"
            inputClass="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
            onSelect={(path) => updateRow(i, 'path', path)}
            onInput={(v) => updateRow(i, 'path', v)}
          />
        {/if}

        <button
          class="text-app-text-4 hover:text-red-400 transition-colors text-lg leading-none"
          onclick={() => removeRow(i)}
          aria-label="Remove extraction"
        >×</button>
      </div>
    {:else}
      <div class="grid grid-cols-[1fr_auto_1fr_auto] gap-2 items-center">
        <EnvVarCombobox
          value={extraction.variable}
          onChange={(v) => updateRow(i, 'variable', v)}
          envVars={envVarNames}
        />
        <select
          class="bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text-2 focus:outline-none focus:border-app-border-2"
          value={extraction.from}
          onchange={(e) => updateRow(i, 'from', e.currentTarget.value)}
        >
          <option value="response.body">response.body</option>
          <option value="response.headers">response.headers</option>
        </select>
        {#if extraction.from === 'response.headers'}
          <input
            class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
            value={extraction.path}
            placeholder="Content-Type"
            oninput={(e) => updateRow(i, 'path', e.currentTarget.value)}
          />
        {:else}
          <SchemaAutocomplete
            {suggestions}
            value={extraction.path}
            placeholder="$.data.id"
            inputClass="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
            onSelect={(path) => updateRow(i, 'path', path)}
            onInput={(v) => updateRow(i, 'path', v)}
          />
        {/if}
        <button
          class="text-app-text-4 hover:text-red-400 transition-colors text-lg leading-none"
          onclick={() => removeRow(i)}
          aria-label="Remove extraction"
        >×</button>
      </div>
    {/if}
  {/each}

  <button
    class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors mt-1"
    onclick={addRow}
  >+ Add Extraction</button>
</div>
