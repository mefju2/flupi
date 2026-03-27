<script lang="ts">
  import type { ScenarioInput } from '$lib/services/tauri-commands';
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';

  interface Props {
    inputs: ScenarioInput[];
    onUpdate: (inputs: ScenarioInput[]) => void;
  }

  let { inputs, onUpdate }: Props = $props();

  function addInput() {
    onUpdate([...inputs, { name: '', description: '', default: '', required: false }]);
  }

  function removeInput(i: number) {
    onUpdate(inputs.filter((_, idx) => idx !== i));
  }

  function updateInput(i: number, field: keyof ScenarioInput, value: string | boolean) {
    const updated = [...inputs];
    updated[i] = { ...updated[i], [field]: value };
    onUpdate(updated);
  }
</script>

<div class="space-y-1">
  {#if inputs.length > 0}
    <div class="grid grid-cols-[1fr_1fr_1fr_auto_auto] gap-2 mb-1">
      <span class="text-xs text-zinc-500 px-1">Name</span>
      <span class="text-xs text-zinc-500 px-1">Description</span>
      <span class="text-xs text-zinc-500 px-1">Default</span>
      <span class="text-xs text-zinc-500 px-1">Req.</span>
      <span></span>
    </div>
  {/if}

  {#each inputs as input, i}
    <div class="grid grid-cols-[1fr_1fr_1fr_auto_auto] gap-2 items-center">
      <input
        class="bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
        value={input.name}
        oninput={(e) => updateInput(i, 'name', e.currentTarget.value)}
        placeholder="name"
      />
      <input
        class="bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-300 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
        value={input.description}
        oninput={(e) => updateInput(i, 'description', e.currentTarget.value)}
        placeholder="description"
      />
      <VariableAutocomplete
        value={input.default}
        placeholder="default"
        onChange={(v) => updateInput(i, 'default', v)}
      />
      <input
        type="checkbox"
        checked={input.required}
        onchange={(e) => updateInput(i, 'required', e.currentTarget.checked)}
        class="accent-cyan-500 w-4 h-4"
        title="Required"
      />
      <button
        class="text-zinc-600 hover:text-red-400 transition-colors text-lg leading-none"
        onclick={() => removeInput(i)}
        aria-label="Remove input"
      >×</button>
    </div>
  {/each}

  <button
    class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors mt-1"
    onclick={addInput}
  >+ Add Input</button>
</div>
