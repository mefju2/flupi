<script lang="ts">
  import SchemaAutocomplete from '$lib/components/shared/SchemaAutocomplete.svelte';
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';
  import { buildOverrideSuggestions } from '$lib/utils/schema-paths';

  interface VarMeta {
    name: string;
    kind: 'input' | 'local';
    description?: string;
    defaultValue?: string;
  }

  interface Props {
    overrides: Record<string, string>;
    onUpdate: (overrides: Record<string, string>) => void;
    requestSchema?: unknown;
    requestPath?: string;
    extractedVars?: VarMeta[];
    onInputEdit?: (name: string, value: string) => void;
  }

  let { overrides, onUpdate, requestSchema = null, requestPath, extractedVars = [], onInputEdit }: Props = $props();

  let extraVarItems = $derived(
    extractedVars.map((v) => ({
      name: v.name,
      value: v.kind === 'input' ? (v.defaultValue ?? '(input)') : '(extracted)',
      kind: v.kind as 'input' | 'local',
      description: v.description,
      defaultValue: v.defaultValue,
    }))
  );

  let rows = $derived(Object.entries(overrides));
  let suggestions = $derived(buildOverrideSuggestions(requestSchema, requestPath));

  let duplicateKeys = $state(new Set<string>());

  function addRow() {
    onUpdate({ ...overrides, '': '' });
  }

  function removeRow(key: string) {
    const next = { ...overrides };
    delete next[key];
    if (duplicateKeys.has(key)) {
      duplicateKeys = new Set([...duplicateKeys].filter((k) => k !== key));
    }
    onUpdate(next);
  }

  function updateKey(oldKey: string, newKey: string) {
    if (newKey !== oldKey && newKey in overrides) {
      duplicateKeys = new Set([...duplicateKeys, oldKey]);
      return;
    }
    if (duplicateKeys.has(oldKey)) {
      duplicateKeys = new Set([...duplicateKeys].filter((k) => k !== oldKey));
    }
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

  {#each rows as [key, value]}
    {@const isDuplicate = duplicateKeys.has(key)}
    <div class="grid grid-cols-[1fr_1fr_auto] gap-2 items-start">
      <div>
        <SchemaAutocomplete
          {suggestions}
          value={key}
          placeholder="body.fieldName"
          inputClass="w-full bg-app-card border rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none {isDuplicate ? 'border-red-500 focus:border-red-500' : 'border-app-border-2 focus:border-app-border-2'}"
          onSelect={(path) => updateKey(key, path)}
          onInput={(v) => updateKey(key, v)}
        />
        {#if isDuplicate}
          <p class="text-xs text-red-400 mt-0.5 font-mono">Duplicate key</p>
        {/if}
      </div>
      <VariableAutocomplete
        value={value}
        onChange={(v) => updateValue(key, v)}
        placeholder={"{{variable}} or literal"}
        extraVars={extraVarItems}        onExtraVarEdit={onInputEdit}      />
      <button
        class="text-app-text-4 hover:text-red-400 transition-colors text-lg leading-none mt-1.5"
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
