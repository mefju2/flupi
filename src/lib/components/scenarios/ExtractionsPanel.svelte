<script lang="ts">
  import type { Extraction } from '$lib/services/tauri-commands';
  import SchemaAutocomplete from './SchemaAutocomplete.svelte';
  import { buildJsonPathSuggestions } from '$lib/utils/schema-paths';

  interface Props {
    extractions: Extraction[];
    onUpdate: (extractions: Extraction[]) => void;
    responseSchema?: unknown;
  }

  let { extractions, onUpdate, responseSchema = null }: Props = $props();

  let suggestions = $derived(buildJsonPathSuggestions(responseSchema));

  function addRow() {
    onUpdate([...extractions, { variable: '', from: 'response.body', path: '' }]);
  }

  function removeRow(i: number) {
    onUpdate(extractions.filter((_, idx) => idx !== i));
  }

  function updateRow(i: number, field: keyof Extraction, value: string) {
    const updated = [...extractions];
    updated[i] = { ...updated[i], [field]: value };
    onUpdate(updated);
  }
</script>

<div class="space-y-1">
  <p class="text-xs text-zinc-500 mb-1">Extract values from the response into variables.</p>
  <p class="text-xs text-zinc-500 mt-1">JSONPath: $.field, $.array[0].value</p>

  {#if extractions.length > 0}
    <div class="grid grid-cols-[1fr_auto_1fr_auto] gap-2 mb-1">
      <span class="text-xs text-zinc-500 px-1">Variable</span>
      <span class="text-xs text-zinc-500 px-1">Source</span>
      <span class="text-xs text-zinc-500 px-1">Path (JSONPath)</span>
      <span></span>
    </div>
  {/if}

  {#each extractions as extraction, i}
    <div class="grid grid-cols-[1fr_auto_1fr_auto] gap-2 items-center">
      <input
        class="bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
        value={extraction.variable}
        oninput={(e) => updateRow(i, 'variable', e.currentTarget.value)}
        placeholder="variableName"
      />
      <select
        class="bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-300 focus:outline-none focus:border-zinc-500"
        value={extraction.from}
        onchange={(e) => updateRow(i, 'from', e.currentTarget.value)}
      >
        <option value="response.body">response.body</option>
        <option value="response.headers">response.headers</option>
      </select>
      <SchemaAutocomplete
        {suggestions}
        value={extraction.path}
        placeholder="$.data.id"
        inputClass="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
        onSelect={(path) => updateRow(i, 'path', path)}
        onInput={(v) => updateRow(i, 'path', v)}
      />
      <button
        class="text-zinc-600 hover:text-red-400 transition-colors text-lg leading-none"
        onclick={() => removeRow(i)}
        aria-label="Remove extraction"
      >×</button>
    </div>
  {/each}

  <button
    class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors mt-1"
    onclick={addRow}
  >+ Add Extraction</button>
</div>
