<script lang="ts">
  import VariableAutocomplete from './VariableAutocomplete.svelte';

  interface Row {
    id: string;
    key: string;
    value: string;
    isSecret?: boolean;
    enabled?: boolean;
  }

  interface Props {
    rows: Row[];
    showSecretToggle?: boolean;
    showEnabled?: boolean;
    readOnlyKeys?: string[];
    onUpdate: (rows: Row[]) => void;
  }

  let { rows, showSecretToggle = false, showEnabled = false, readOnlyKeys = [], onUpdate }: Props = $props();

  function removeRow(index: number) {
    onUpdate(rows.filter((_, i) => i !== index));
  }

  function updateRow(index: number, field: keyof Row, value: string | boolean) {
    const updated = [...rows];
    updated[index] = { ...updated[index], [field]: value };
    onUpdate(updated);
  }

  // Draft row state — never passed to onUpdate until committed
  let draftKey = $state('');
  let draftValue = $state('');
  let draftId = $state(crypto.randomUUID());
  let draftContainer: HTMLDivElement | undefined;

  // Reset draft when the rows source changes (e.g. switching to a different request)
  $effect(() => {
    void rows;
    draftKey = '';
    draftValue = '';
    draftId = crypto.randomUUID();
  });

  function commitDraft() {
    if (!draftKey) return;
    onUpdate([...rows, { id: draftId, key: draftKey, value: draftValue, enabled: true }]);
    draftKey = '';
    draftValue = '';
    draftId = crypto.randomUUID();
  }

  function onDraftFocusOut(e: FocusEvent) {
    const related = e.relatedTarget as HTMLElement | null;
    if (!related || !draftContainer?.contains(related)) {
      // Delay slightly past VariableAutocomplete's 150ms blur timer so dropdown
      // item clicks are processed before we commit
      setTimeout(() => {
        if (!draftContainer?.contains(document.activeElement)) {
          commitDraft();
        }
      }, 160);
    }
  }

  function onDraftKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); commitDraft(); }
    if (e.key === 'Escape') { draftKey = ''; draftValue = ''; }
  }
</script>

<div class="space-y-1">
  {#each rows as row, i (row.id)}
    <div class="flex gap-2 items-center {row.enabled === false ? 'opacity-40' : ''}">
      {#if showEnabled}
        <input
          type="checkbox"
          checked={row.enabled !== false}
          onchange={(e) => updateRow(i, 'enabled', e.currentTarget.checked)}
          class="accent-cyan-500 shrink-0"
          aria-label="Enable row"
        />
      {/if}
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

  <!-- Trailing empty row — always shown, committed on focus-out or Enter -->
  <div
    bind:this={draftContainer}
    class="flex gap-2 items-center"
    onfocusout={onDraftFocusOut}
  >
    {#if showEnabled}
      <input type="checkbox" disabled class="accent-cyan-500 shrink-0 opacity-30" aria-label="Enable row" />
    {/if}
    <input
      class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-hover"
      value={draftKey}
      oninput={(e) => { draftKey = e.currentTarget.value; }}
      onkeydown={onDraftKeyDown}
      placeholder="Key"
    />
    <VariableAutocomplete
      class="flex-1"
      value={draftValue}
      placeholder="Value"
      onChange={(v) => { draftValue = v; }}
    />
    {#if showSecretToggle}
      <div class="w-16"></div>
    {/if}
    <button disabled aria-hidden="true" class="opacity-0 text-lg leading-none pointer-events-none">×</button>
  </div>
</div>
