<script lang="ts">
  interface Props {
    overrides: Record<string, string>;
    onUpdate: (overrides: Record<string, string>) => void;
  }

  let { overrides, onUpdate }: Props = $props();

  let rows = $derived(Object.entries(overrides));

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
  <p class="text-xs text-zinc-500 mb-2">Override request fields (dot-notation: <span class="font-mono">body.fieldName</span>, <span class="font-mono">headers.X-Custom</span>)</p>

  {#if rows.length > 0}
    <div class="grid grid-cols-[1fr_1fr_auto] gap-2 mb-1">
      <span class="text-xs text-zinc-500 px-1">Field</span>
      <span class="text-xs text-zinc-500 px-1">Value</span>
      <span></span>
    </div>
  {/if}

  {#each rows as [key, value], i}
    <div class="grid grid-cols-[1fr_1fr_auto] gap-2 items-center">
      <input
        class="bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
        value={key}
        oninput={(e) => updateKey(key, e.currentTarget.value)}
        placeholder="body.fieldName"
      />
      <input
        class="bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
        value={value}
        oninput={(e) => updateValue(key, e.currentTarget.value)}
        placeholder={"{{variable}} or literal"}
      />
      <button
        class="text-zinc-600 hover:text-red-400 transition-colors text-lg leading-none"
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
