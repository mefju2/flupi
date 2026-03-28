<script lang="ts">
  import type { ScenarioData } from '$lib/services/tauri-commands';

  interface Props {
    scenario: ScenarioData;
    onRun: (inputs: Record<string, string>) => void;
    onBack: () => void;
  }

  let { scenario, onRun, onBack }: Props = $props();

  let inputValues = $state<Record<string, string>>(
    Object.fromEntries(scenario.inputs.map((i) => [i.name, i.default ?? '']))
  );

  function hasUnresolvedToken(value: string): boolean {
    return /\{\{[^}]+\}\}/.test(value);
  }

  function handleRun() {
    onRun({ ...inputValues });
  }
</script>

<div class="flex flex-col h-full bg-app-bg">
  <div class="flex items-center gap-3 px-4 py-2 border-b border-app-border shrink-0">
    <button
      class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
      onclick={onBack}
    >← Back to Editor</button>
    <span class="text-sm text-app-text-2 font-medium">{scenario.name}</span>
  </div>

  <div class="flex-1 overflow-y-auto px-4 py-6 max-w-xl">
    <h2 class="text-xs text-app-text-3 uppercase tracking-wider mb-4">Run Parameters</h2>

    {#if scenario.inputs.length === 0}
      <p class="text-sm text-app-text-3 mb-6">This scenario has no input parameters.</p>
    {:else}
      <div class="space-y-4 mb-6">
        {#each scenario.inputs as input}
          <div class="{input.required ? 'border-l-2 border-red-500 pl-2' : ''}">
            <label class="flex items-center gap-2 text-sm text-app-text-2 mb-1">
              <span class="font-mono">{input.name}</span>
              {#if input.required}
                <span class="text-xs text-red-400">required</span>
              {/if}
            </label>
            {#if input.description}
              <p class="text-xs text-app-text-3 mb-1">{input.description}</p>
            {/if}
            <input
              class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm font-mono focus:outline-none focus:border-app-border-2
                {hasUnresolvedToken(inputValues[input.name] ?? '') ? 'text-amber-400' : 'text-app-text'}"
              value={inputValues[input.name] ?? ''}
              oninput={(e) => { inputValues = { ...inputValues, [input.name]: e.currentTarget.value }; }}
              placeholder={input.default || 'Enter value…'}
            />
            {#if hasUnresolvedToken(inputValues[input.name] ?? '')}
              <p class="text-xs text-amber-500 mt-1">Contains unresolved variable tokens</p>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <button
      class="px-4 py-2 text-sm text-zinc-900 bg-cyan-400 hover:bg-cyan-300 rounded font-medium transition-colors"
      onclick={handleRun}
    >Run Scenario</button>
  </div>
</div>
