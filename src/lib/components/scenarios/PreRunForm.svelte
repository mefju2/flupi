<script lang="ts">
  import { untrack } from 'svelte';
  import type { ScenarioData } from '$lib/services/tauri-commands';
  import SectionHeader from '$lib/components/shared/SectionHeader.svelte';
  import ToolBar from '$lib/components/shared/ToolBar.svelte';
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';

  interface Props {
    scenario: ScenarioData;
    onRun: (inputs: Record<string, string>) => void;
    onBack: () => void;
  }

  let { scenario, onRun, onBack }: Props = $props();

  let inputValues = $state<Record<string, string>>(
    untrack(() => Object.fromEntries(scenario.inputs.map((i) => [i.name, i.default ?? ''])))
  );

  let allRequiredFilled = $derived(
    scenario.inputs
      .filter((i) => i.required)
      .every((i) => (inputValues[i.name] ?? '').trim() !== '')
  );

  function handleRun() {
    onRun({ ...inputValues });
  }
</script>

<div class="flex flex-col h-full bg-app-bg">
  <ToolBar>
    <button
      class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
      onclick={onBack}
    >← Back to Editor</button>
    <span class="text-sm text-app-text-2 font-medium">{scenario.name}</span>
  </ToolBar>

  <div class="flex-1 overflow-y-auto px-4 py-6 max-w-xl">
    <SectionHeader class="mb-4">Run Parameters</SectionHeader>

    {#if scenario.inputs.length === 0}
      <p class="text-sm text-app-text-3 mb-6">This scenario has no input parameters.</p>
    {:else}
      <div class="space-y-4 mb-6">
        {#each scenario.inputs as input}
          {@const isInvalid = input.required && (inputValues[input.name] ?? '').trim() === ''}
          <div class="{isInvalid ? 'border-l-2 border-red-500 pl-2' : ''}">
            <label for={input.name} class="flex items-center gap-2 text-sm text-app-text-2 mb-1">
              <span id={input.name} class="font-mono">{input.name}</span>
              {#if input.required}
                <span class="text-xs {isInvalid ? 'text-red-400' : 'text-app-text-3'}">required</span>
              {/if}
            </label>
            {#if input.description}
              <p class="text-xs text-app-text-3 mb-1">{input.description}</p>
            {/if}
            <VariableAutocomplete
              value={inputValues[input.name] ?? ''}
              onChange={(v) => { inputValues = { ...inputValues, [input.name]: v }; }}
              placeholder={input.default || 'Enter value…'}
            />
          </div>
        {/each}
      </div>
    {/if}

    <button
      class="px-4 py-2 text-sm text-zinc-900 bg-cyan-400 hover:bg-cyan-300 rounded font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      onclick={handleRun}
      disabled={!allRequiredFilled}
    >Run Scenario</button>
  </div>
</div>
