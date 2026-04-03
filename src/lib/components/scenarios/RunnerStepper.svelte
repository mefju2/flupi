<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import type { ScenarioData, ScenarioStep, StepResult } from '$lib/services/tauri-commands';
  import StepResultCard from './StepResultCard.svelte';
  import VariableStatePanel from './VariableStatePanel.svelte';
  import SectionHeader from '$lib/components/shared/SectionHeader.svelte';
  import ToolBar from '$lib/components/shared/ToolBar.svelte';

  interface Props {
    scenario: ScenarioData;
    inputs?: Record<string, string>;
    onBack: () => void;
    onRetry: () => void;
  }

  let { scenario, inputs = {}, onBack, onRetry }: Props = $props();

  type StepState = 'waiting' | 'running' | 'success' | 'error' | 'skipped';

  interface StepStatus {
    state: StepState;
    result?: StepResult;
  }

  let stepStatuses = $state<Record<string, StepStatus>>(
    Object.fromEntries(scenario.steps.map((s) => [s.id, { state: 'waiting' as StepState }]))
  );
  let currentVariables = $state<Record<string, string>>({});
  let runComplete = $state(false);
  let runFailed = $derived(Object.values(stepStatuses).some((s) => s.state === 'error'));

  let unlisten: (() => void) | null = null;

  onMount(async () => {
    unlisten = await listen<StepResult>('scenario-step-result', (event) => {
      const result = event.payload;
      const state: StepState = result.status === 'success' ? 'success' : 'error';
      let newStatuses = { ...stepStatuses, [result.step_id]: { state, result } };
      currentVariables = { ...currentVariables, ...result.extracted };

      if (result.status === 'error') {
        // Mark all subsequent steps as skipped and stop the run
        const failedIndex = scenario.steps.findIndex((s) => s.id === result.step_id);
        for (let i = failedIndex + 1; i < scenario.steps.length; i++) {
          newStatuses[scenario.steps[i].id] = { state: 'skipped' };
        }
        stepStatuses = newStatuses;
        runComplete = true;
        return;
      }

      stepStatuses = newStatuses;

      // Check if all steps are done (success path)
      const allDone = scenario.steps.every((s) => {
        const status = newStatuses[s.id];
        return status && (status.state === 'success' || status.state === 'error' || status.state === 'skipped');
      });
      if (allDone) runComplete = true;
    });

    // Mark first step as running (backend will drive actual progress via events)
    if (scenario.steps.length > 0) {
      const firstId = scenario.steps[0].id;
      stepStatuses = { ...stepStatuses, [firstId]: { state: 'running' } };
    }
  });

  onDestroy(() => { unlisten?.(); });

  $effect(() => {
    // When a step succeeds, mark the next waiting step as running
    for (let i = 0; i < scenario.steps.length - 1; i++) {
      const current = stepStatuses[scenario.steps[i].id];
      const next = stepStatuses[scenario.steps[i + 1].id];
      if (current?.state === 'success' && next?.state === 'waiting') {
        stepStatuses = { ...stepStatuses, [scenario.steps[i + 1].id]: { state: 'running' } };
        break;
      }
    }
  });
</script>

<div class="flex flex-col h-full bg-app-bg">
  <ToolBar>
    <button class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors" onclick={onBack}>
      ← Back
    </button>
    <span class="text-sm text-app-text-2 font-medium">{scenario.name}</span>
    {#if runComplete}
      {#if runFailed}
        <span class="text-xs text-red-400 ml-auto">Failed</span>
      {:else}
        <span class="text-xs text-green-400 ml-auto">Completed</span>
      {/if}
      <button
        class="text-xs px-2.5 py-1 rounded border border-app-border-2 text-app-text-2 hover:bg-app-hover transition-colors"
        onclick={onRetry}
      >↺ Retry</button>
    {:else}
      <span class="text-xs text-cyan-400 ml-auto animate-pulse">Running…</span>
    {/if}
  </ToolBar>

  <div class="flex flex-1 overflow-hidden">
    <!-- Steps stepper -->
    <div class="flex-1 overflow-y-auto px-4 py-4 space-y-2">
      {#each scenario.steps as step, i (step.id)}
        {@const status = stepStatuses[step.id] ?? { state: 'waiting' as StepState }}
        <div class="flex gap-3">
          <!-- Step indicator -->
          <div class="flex flex-col items-center shrink-0 pt-2">
            <div class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium
              {status.state === 'waiting' ? 'bg-app-hover text-app-text-3'
                : status.state === 'running' ? 'bg-cyan-900 text-cyan-300'
                : status.state === 'success' ? 'bg-green-900 text-green-300'
                : status.state === 'skipped' ? 'bg-app-hover text-app-text-4'
                : 'bg-red-900 text-red-300'}">
              {#if status.state === 'running'}
                <span class="animate-spin text-xs">◌</span>
              {:else if status.state === 'success'}
                ✓
              {:else if status.state === 'error'}
                ✗
              {:else if status.state === 'skipped'}
                —
              {:else}
                {i + 1}
              {/if}
            </div>
            {#if i < scenario.steps.length - 1}
              <div class="w-px flex-1 mt-1 bg-app-border min-h-4"></div>
            {/if}
          </div>

          <!-- Step content -->
          <div class="flex-1 min-w-0 pb-4">
            {#if status.result}
              <StepResultCard {step} result={status.result} />
            {:else}
              <div class="border border-app-border rounded bg-app-panel px-3 py-2 {status.state === 'running' ? 'ring-1 ring-cyan-500/30' : ''}">
                <span class="text-sm {status.state === 'running' ? 'text-cyan-300' : status.state === 'skipped' ? 'text-app-text-4 line-through' : 'text-app-text-3'}">{step.name}</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Variable state panel -->
    <div class="w-56 border-l border-app-border px-3 py-4 overflow-y-auto shrink-0 space-y-4">
      {#if Object.keys(inputs).length > 0}
        <div>
          <SectionHeader class="mb-3">Inputs</SectionHeader>
          <VariableStatePanel variables={inputs} />
        </div>
      {/if}
      <div>
        <SectionHeader class="mb-3">Extracted</SectionHeader>
        <VariableStatePanel variables={currentVariables} />
      </div>
    </div>
  </div>
</div>
