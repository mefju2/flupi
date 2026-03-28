<script lang="ts">
  import VariableAutocomplete from './VariableAutocomplete.svelte';

  interface Row {
    key: string;
    value: string;
    isSecret?: boolean;
  }

  interface Props {
    rows: Row[];
    showSecretToggle?: boolean;
    readOnlyKeys?: string[];
    onUpdate: (rows: Row[]) => void;
  }

  let { rows, showSecretToggle = false, readOnlyKeys = [], onUpdate }: Props = $props();

  function addRow() {
    onUpdate([...rows, { key: '', value: '', isSecret: false }]);
  }

  function removeRow(index: number) {
    onUpdate(rows.filter((_, i) => i !== index));
  }

  function updateRow(index: number, field: keyof Row, value: string | boolean) {
    const updated = [...rows];
    updated[index] = { ...updated[index], [field]: value };
    onUpdate(updated);
  }
</script>

<div class="space-y-1">
  {#each rows as row, i}
    <div class="flex gap-2 items-center">
      <input
        class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-hover"
        value={row.key}
        readonly={readOnlyKeys.includes(row.key)}
        oninput={(e) => updateRow(i, 'key', e.currentTarget.value)}
        placeholder="Key"
      />
      {#if row.isSecret}
        <div class="flex-1 flex items-center gap-1">
          <input
            class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-hover opacity-60"
            type="password"
            value={row.value}
            oninput={(e) => updateRow(i, 'value', e.currentTarget.value)}
            placeholder="Value"
          />
          <span class="text-app-text-3 text-xs shrink-0" title="Secret">🔒</span>
        </div>
      {:else}
        <VariableAutocomplete
          class="flex-1"
          value={row.value}
          placeholder="Value"
          onChange={(v) => updateRow(i, 'value', v)}
        />
      {/if}
      {#if showSecretToggle}
        <label class="flex items-center gap-1 text-xs text-app-text-3 whitespace-nowrap cursor-pointer">
          <input
            type="checkbox"
            checked={row.isSecret}
            onchange={(e) => updateRow(i, 'isSecret', e.currentTarget.checked)}
            class="accent-cyan-500"
          />
          Secret
        </label>
      {/if}
      <button
        class="opacity-30 hover:opacity-100 text-app-text-4 hover:text-red-400 transition-opacity text-lg leading-none"
        onclick={() => removeRow(i)}
        aria-label="Remove row"
      >×</button>
    </div>
  {/each}
  <button
    class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors mt-1"
    onclick={addRow}
  >+ Add variable</button>
</div>
