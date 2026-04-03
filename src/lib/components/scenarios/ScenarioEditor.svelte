<script lang="ts">
  import type { ScenarioData, ScenarioStep } from '$lib/services/tauri-commands';
  import { requestTree } from '$lib/stores/requests';
  import { dndzone } from 'svelte-dnd-action';
  import InputsList from './InputsList.svelte';
  import StepCard from './StepCard.svelte';
  import SectionHeader from '$lib/components/shared/SectionHeader.svelte';
  import ToolBar from '$lib/components/shared/ToolBar.svelte';

  interface Props {
    scenario: ScenarioData;
    onUpdate: (scenario: ScenarioData) => void;
    onSave: () => void;
    onRun: () => void;
  }

  let { scenario, onUpdate, onSave, onRun }: Props = $props();

  let nameDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  function handleNameInput(value: string) {
    if (nameDebounceTimer) clearTimeout(nameDebounceTimer);
    nameDebounceTimer = setTimeout(() => {
      onUpdate({ ...scenario, name: value });
      nameDebounceTimer = null;
    }, 300);
  }

  $effect(() => {
    return () => {
      if (nameDebounceTimer) clearTimeout(nameDebounceTimer);
    };
  });

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

  function handleDndConsider(e: CustomEvent<{ items: ScenarioStep[] }>) {
    onUpdate({ ...scenario, steps: e.detail.items });
  }

  function handleDndFinalize(e: CustomEvent<{ items: ScenarioStep[] }>) {
    onUpdate({ ...scenario, steps: e.detail.items });
  }

  // Variables available to step N: scenario inputs + variables extracted by steps 0..N-1
  function extractedVarsBefore(index: number): string[] {
    const names: string[] = scenario.inputs.map((inp) => inp.name);
    for (let i = 0; i < index; i++) {
      for (const ext of scenario.steps[i].extract) {
        if (ext.variable && !names.includes(ext.variable)) names.push(ext.variable);
      }
    }
    return names;
  }
</script>

<div class="flex flex-col h-full bg-app-bg">
  <!-- Top bar -->
  <ToolBar>
    <input
      class="flex-1 bg-transparent text-app-text text-base font-medium focus:outline-none placeholder:text-app-text-4"
      value={scenario.name}
      oninput={(e) => handleNameInput(e.currentTarget.value)}
      placeholder="Scenario name"
    />
    <button
      class="px-3 py-1 text-xs bg-transparent border border-app-border-2 hover:border-app-border-2 text-app-text-3 hover:text-app-text rounded transition-colors"
      onclick={onSave}
    >Save</button>
    <button
      class="px-3 py-1 text-xs text-zinc-900 bg-cyan-400 hover:bg-cyan-300 rounded font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      onclick={onRun}
      disabled={scenario.steps.length === 0}
      title={scenario.steps.length === 0 ? 'Add at least one step to run' : 'Run scenario (Ctrl+Shift+Enter)'}
    >Run</button>
  </ToolBar>

  <div class="flex-1 overflow-y-auto px-4 py-4 space-y-6">
    <!-- Inputs section -->
    <section>
      <SectionHeader class="mb-3">Inputs</SectionHeader>
      <InputsList
        inputs={scenario.inputs}
        onUpdate={(inputs) => onUpdate({ ...scenario, inputs })}
      />
    </section>

    <!-- Steps section -->
    <section>
      <SectionHeader class="mb-3">Steps</SectionHeader>
      <div
        use:dndzone={{ items: scenario.steps, dropTargetStyle: {} }}
        onconsider={handleDndConsider}
        onfinalize={handleDndFinalize}
      >
        {#each scenario.steps as step, i (step.id)}
          <StepCard
            {step}
            requestTree={$requestTree}
            index={i}
            extractedVars={extractedVarsBefore(i)}
            onUpdate={(s) => updateStep(i, s)}
            onDelete={() => deleteStep(i)}
          />
        {/each}
      </div>
      {#if scenario.steps.length === 0}
        <p class="text-xs text-app-text-4 italic">No steps yet.</p>
      {/if}
      <button
        class="mt-2 text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
        onclick={addStep}
      >+ Add Step</button>
    </section>
  </div>
</div>
