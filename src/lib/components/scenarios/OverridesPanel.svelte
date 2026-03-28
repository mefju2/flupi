<script lang="ts">
  import SchemaAutocomplete from './SchemaAutocomplete.svelte';
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';
  import { buildOverrideSuggestions } from '$lib/utils/schema-paths';

  interface Props {
    overrides: Record<string, string>;
    onUpdate: (overrides: Record<string, string>) => void;
    requestSchema?: unknown;
  }

  let { overrides, onUpdate, requestSchema = null }: Props = $props();

  let rows = $derived(Object.entries(overrides));
  let suggestions = $derived(buildOverrideSuggestions(requestSchema));

  function addRow() {
    onUpdate({ ...overrides, '': '' });
  }

  function removeRow(key: string) {
    const next = { ...overrides };
    delete next[key];
    onUpdate(next);
  }

  function updateKey(oldKey: string, newKey: string) {
    const next: Record<string, string> = {};
    for (const [k, v] of Object.entries(overrides)) {
      next[k === oldKey ? newKey : k] = v;
    }
    onUpdate(next);
  }

  function updateValue(key: string, value: string) {
    onUpdate({ ...overrides, [key]: value });
  }
</script>

<div class="space-y-1">
  <p class="text-xs text-app-text-3 mb-2">
    Override request fields (dot-notation: <span class="font-mono">body.fieldName</span>,
    <span class="font-mono">headers.X-Custom</span>)
  </p>

  {#if rows.length > 0}
    <div class="grid grid-cols-[1fr_1fr_auto] gap-2 mb-1">
      <span class="text-xs text-app-text-3 px-1">Field</span>
      <span class="text-xs text-app-text-3 px-1">Value</span>
      <span></span>
    </div>
  {/if}

  {#each rows as [key, value], i}
    <div class="grid grid-cols-[1fr_1fr_auto] gap-2 items-center">
      <SchemaAutocomplete
        {suggestions}
        value={key}
        placeholder="body.fieldName"
        inputClass="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
        onSelect={(path) => updateKey(key, path)}
        onInput={(v) => updateKey(key, v)}
      />
      <VariableAutocomplete
        value={value}
        onChange={(v) => updateValue(key, v)}
        placeholder={"{{variable}} or literal"}
      />
      <button
        class="text-app-text-4 hover:text-red-400 transition-colors text-lg leading-none"
        onclick={() => removeRow(key)}
        aria-label="Remove override"
      >×</button>
    </div>
  {/each}

  <button
    class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors mt-1"
    onclick={addRow}
  >+ Add Override</button>
</div>
