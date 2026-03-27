<script lang="ts">
  import type { ScenarioData, ScenarioStep } from '$lib/services/tauri-commands';
  import { requestTree } from '$lib/stores/requests';
  import InputsList from './InputsList.svelte';
  import StepCard from './StepCard.svelte';

  interface Props {
    scenario: ScenarioData;
    onUpdate: (scenario: ScenarioData) => void;
    onSave: () => void;
    onRun: () => void;
  }

  let { scenario, onUpdate, onSave, onRun }: Props = $props();

  function addStep() {
    const newStep: ScenarioStep = {
      id: crypto.randomUUID(),
      name: 'New Step',
      requestId: '',
      overrides: {},
      extract: [],
    };
    onUpdate({ ...scenario, steps: [...scenario.steps, newStep] });
  }

  function updateStep(index: number, step: ScenarioStep) {
    const steps = [...scenario.steps];
    steps[index] = step;
    onUpdate({ ...scenario, steps });
  }

  function deleteStep(index: number) {
    onUpdate({ ...scenario, steps: scenario.steps.filter((_, i) => i !== index) });
  }
</script>

<div class="flex flex-col h-full bg-zinc-950">
  <!-- Top bar -->
  <div class="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 shrink-0">
    <input
      class="flex-1 bg-transparent text-zinc-100 text-base font-medium focus:outline-none placeholder:text-zinc-600"
      value={scenario.name}
      oninput={(e) => onUpdate({ ...scenario, name: e.currentTarget.value })}
      placeholder="Scenario name"
    />
    <button
      class="px-3 py-1 text-xs bg-transparent border border-zinc-700 hover:border-zinc-500 text-zinc-400 hover:text-zinc-200 rounded transition-colors"
      onclick={onSave}
    >Save</button>
    <button
      class="px-3 py-1 text-xs text-zinc-900 bg-cyan-400 hover:bg-cyan-300 rounded font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      onclick={onRun}
      disabled={scenario.steps.length === 0}
      title={scenario.steps.length === 0 ? 'Add at least one step to run' : 'Run scenario (Ctrl+Shift+Enter)'}
    >Run</button>
  </div>

  <div class="flex-1 overflow-y-auto px-4 py-4 space-y-6">
    <!-- Inputs section -->
    <section>
      <h2 class="text-xs text-zinc-500 uppercase tracking-wider mb-3">Inputs</h2>
      <InputsList
        inputs={scenario.inputs}
        onUpdate={(inputs) => onUpdate({ ...scenario, inputs })}
      />
    </section>

    <!-- Steps section -->
    <section>
      <h2 class="text-xs text-zinc-500 uppercase tracking-wider mb-3">Steps</h2>
      {#each scenario.steps as step, i (step.id)}
        <StepCard
          {step}
          requestTree={$requestTree}
          index={i}
          onUpdate={(s) => updateStep(i, s)}
          onDelete={() => deleteStep(i)}
        />
      {/each}
      {#if scenario.steps.length === 0}
        <p class="text-xs text-zinc-600 italic">No steps yet.</p>
      {/if}
      <button
        class="mt-2 text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
        onclick={addStep}
      >+ Add Step</button>
    </section>
  </div>
</div>
