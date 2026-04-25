<script lang="ts">
  import type { ScenarioData, ScenarioStep, RequestStep, DelayStep, PauseStep } from '$lib/services/tauri-commands';
  import { isDelayStep, isRequestStep, isPauseStep } from '$lib/services/tauri-commands';
  import { requestTree } from '$lib/stores/requests';
  import InputsList from './InputsList.svelte';
  import StepCard from './StepCard.svelte';
  import DelayStepCard from './DelayStepCard.svelte';
  import PauseStepCard from './PauseStepCard.svelte';
  import SectionHeader from '$lib/components/shared/SectionHeader.svelte';
  import ToolBar from '$lib/components/shared/ToolBar.svelte';
  import ConfirmDialog from '$lib/components/shared/ConfirmDialog.svelte';

  interface VarMeta {
    name: string;
    kind: 'input' | 'local';
    description?: string;
    defaultValue?: string;
  }

  interface Props {
    scenario: ScenarioData;
    onUpdate: (scenario: ScenarioData) => void;
    onSave: () => void;
    onRun: () => void;
    isDirty?: boolean;
  }

  let { scenario, onUpdate, onSave, onRun, isDirty = false }: Props = $props();

  let nameDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingDeleteStepId = $state<string | null>(null);
  let pendingDeleteStep = $derived(
    pendingDeleteStepId
      ? scenario.steps.find((step) => step.id === pendingDeleteStepId) ?? null
      : null
  );

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
    const newStep: RequestStep = {
      id: crypto.randomUUID(),
      name: 'New Step',
      requestId: '',
      overrides: {},
      extract: [],
    };
    onUpdate({ ...scenario, steps: [...scenario.steps, newStep] });
  }

  function addDelay() {
    const newStep: DelayStep = {
      id: crypto.randomUUID(),
      name: 'Delay',
      duration: 500,
    };
    onUpdate({ ...scenario, steps: [...scenario.steps, newStep] });
  }

  function addPause() {
    const newStep: PauseStep = {
      id: crypto.randomUUID(),
      name: 'Pause',
      pause: true,
    };
    onUpdate({ ...scenario, steps: [...scenario.steps, newStep] });
  }

  function updateStep(index: number, step: ScenarioStep) {
    const steps = [...scenario.steps];
    steps[index] = step;
    onUpdate({ ...scenario, steps });
  }

  function requestDeleteStep(stepId: string) {
    pendingDeleteStepId = stepId;
  }

  function confirmDeleteStep() {
    if (!pendingDeleteStepId) return;
    onUpdate({
      ...scenario,
      steps: scenario.steps.filter((step) => step.id !== pendingDeleteStepId),
    });
    pendingDeleteStepId = null;
  }

  function cancelDeleteStep() {
    pendingDeleteStepId = null;
  }

  function duplicateStep(index: number) {
    const step = scenario.steps[index];
    const duplicate: ScenarioStep = {
      ...step,
      id: crypto.randomUUID(),
      name: getDuplicateStepName(step.name),
    };
    const steps = [...scenario.steps];
    steps.splice(index + 1, 0, duplicate);
    onUpdate({ ...scenario, steps });
  }

  function moveStep(index: number, dir: 'up' | 'down') {
    const steps = [...scenario.steps];
    const target = dir === 'up' ? index - 1 : index + 1;
    [steps[index], steps[target]] = [steps[target], steps[index]];
    onUpdate({ ...scenario, steps });
  }

  function moveInput(index: number, dir: 'up' | 'down') {
    const inputs = [...scenario.inputs];
    const target = dir === 'up' ? index - 1 : index + 1;
    [inputs[index], inputs[target]] = [inputs[target], inputs[index]];
    onUpdate({ ...scenario, inputs });
  }

  function updateInputDefault(name: string, value: string) {
    const inputs = scenario.inputs.map((inp) =>
      inp.name === name ? { ...inp, default: value } : inp
    );
    onUpdate({ ...scenario, inputs });
  }

  // Variables available to step N: scenario inputs + variables extracted by steps 0..N-1
  function extractedVarsBefore(index: number): VarMeta[] {
    const result: VarMeta[] = scenario.inputs.map((inp) => ({
      name: inp.name,
      kind: 'input',
      description: inp.description || undefined,
      defaultValue: inp.default,
    }));
    const seen = new Set(result.map((v) => v.name));
    for (let i = 0; i < index; i++) {
      const s = scenario.steps[i];
      if (isRequestStep(s)) {
        for (const ext of s.extract) {
          if (ext.variable && !seen.has(ext.variable)) {
            result.push({ name: ext.variable, kind: 'local' });
            seen.add(ext.variable);
          }
        }
      }
    }
    return result;
  }

  function getDuplicateStepName(name: string): string {
    const trimmedName = name.trim();
    const baseName = (trimmedName.match(/^(.*?)(?: \(\d+\))?$/)?.[1] ?? trimmedName).trimEnd() || 'Step';
    const escapedBaseName = baseName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const duplicatePattern = new RegExp(`^${escapedBaseName}(?: \\((\\d+)\\))?$`);
    const usedNumbers = new Set(
      scenario.steps
        .map((step) => step.name.trim().match(duplicatePattern))
        .filter((match): match is RegExpMatchArray => match !== null)
        .map((match) => (match[1] ? Number(match[1]) : 0))
    );

    let nextNumber = 1;
    while (usedNumbers.has(nextNumber)) {
      nextNumber += 1;
    }

    return `${baseName} (${nextNumber})`;
  }
</script>

<div class="flex flex-col h-full bg-app-bg">
  {#if pendingDeleteStep}
    <ConfirmDialog
      message="Delete this step?"
      detail={`This will delete "${pendingDeleteStep.name || 'Unnamed Step'}" from the scenario.`}
      confirmLabel="Delete"
      onConfirm={confirmDeleteStep}
      onCancel={cancelDeleteStep}
    />
  {/if}

  <!-- Top bar -->
  <ToolBar>
    <input
      class="flex-1 bg-transparent text-app-text text-base font-medium focus:outline-none placeholder:text-app-text-4"
      value={scenario.name}
      oninput={(e) => handleNameInput(e.currentTarget.value)}
      placeholder="Scenario name"
    />
    <button
      class="px-3 py-1 text-xs bg-transparent border rounded transition-colors
        {isDirty
          ? 'border-cyan-500 text-cyan-400 hover:border-cyan-400 hover:text-cyan-300'
          : 'border-app-border-2 text-app-text-3 hover:border-app-border-2 hover:text-app-text'}"
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
        onMoveUp={(i) => moveInput(i, 'up')}
        onMoveDown={(i) => moveInput(i, 'down')}
      />
    </section>

    <!-- Steps section -->
    <section>
      <SectionHeader class="mb-3">Steps</SectionHeader>
      <div class="space-y-0">
        {#each scenario.steps as step, i (step.id)}
          {#if isDelayStep(step)}
            <DelayStepCard
              {step}
              index={i}
              onUpdate={(s) => updateStep(i, s)}
              onDuplicate={() => duplicateStep(i)}
              onDelete={() => requestDeleteStep(step.id)}
              onMoveUp={i > 0 ? () => moveStep(i, 'up') : undefined}
              onMoveDown={i < scenario.steps.length - 1 ? () => moveStep(i, 'down') : undefined}
            />
          {:else if isPauseStep(step)}
            <PauseStepCard
              {step}
              index={i}
              onUpdate={(s) => updateStep(i, s)}
              onDuplicate={() => duplicateStep(i)}
              onDelete={() => requestDeleteStep(step.id)}
              onMoveUp={i > 0 ? () => moveStep(i, 'up') : undefined}
              onMoveDown={i < scenario.steps.length - 1 ? () => moveStep(i, 'down') : undefined}
            />
          {:else}
            <StepCard
              step={step as RequestStep}
              requestTree={$requestTree}
              index={i}
              extractedVars={extractedVarsBefore(i)}
              onUpdate={(s) => updateStep(i, s)}
              onDuplicate={() => duplicateStep(i)}
              onDelete={() => requestDeleteStep(step.id)}
              onMoveUp={i > 0 ? () => moveStep(i, 'up') : undefined}
              onMoveDown={i < scenario.steps.length - 1 ? () => moveStep(i, 'down') : undefined}
              onInputEdit={(name, val) => updateInputDefault(name, val)}
            />
          {/if}
        {/each}
      </div>
      {#if scenario.steps.length === 0}
        <p class="text-xs text-app-text-4 italic">No steps yet.</p>
      {/if}
      <div class="flex gap-3 mt-2">
        <button
          type="button"
          class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
          onclick={addStep}
        >+ Add Step</button>
        <button
          type="button"
          class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
          onclick={addDelay}
        >+ Add Delay</button>
        <button
          type="button"
          class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
          onclick={addPause}
        >+ Add Pause</button>
      </div>
    </section>
  </div>
</div>
