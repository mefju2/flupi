<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import type { ScenarioData, ScenarioStep, StepResult } from '$lib/services/tauri-commands';
  import StepResultCard from './StepResultCard.svelte';
  import VariableStatePanel from './VariableStatePanel.svelte';

  interface Props {
    scenario: ScenarioData;
    onBack: () => void;
  }

  let { scenario, onBack }: Props = $props();

  type StepState = 'waiting' | 'running' | 'success' | 'error';

  interface StepStatus {
    state: StepState;
    result?: StepResult;
  }

  let stepStatuses = $state<Record<string, StepStatus>>(
    Object.fromEntries(scenario.steps.map((s) => [s.id, { state: 'waiting' as StepState }]))
  );
  let currentVariables = $state<Record<string, string>>({});
  let runComplete = $state(false);

  let unlisten: (() => void) | null = null;

  onMount(async () => {
    unlisten = await listen<StepResult>('scenario-step-result', (event) => {
      const result = event.payload;
      const state: StepState = result.status === 'success' ? 'success' : 'error';
      stepStatuses = { ...stepStatuses, [result.step_id]: { state, result } };
      currentVariables = { ...currentVariables, ...result.extracted };

      // Check if all steps are done
      const allDone = scenario.steps.every((s) => {
        const status = stepStatuses[s.id];
        return status && (status.state === 'success' || status.state === 'error');
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
    // When a step completes, mark the next one as running
    for (let i = 0; i < scenario.steps.length - 1; i++) {
      const current = stepStatuses[scenario.steps[i].id];
      const next = stepStatuses[scenario.steps[i + 1].id];
      if ((current?.state === 'success' || current?.state === 'error') && next?.state === 'waiting') {
        stepStatuses = { ...stepStatuses, [scenario.steps[i + 1].id]: { state: 'running' } };
        break;
      }
    }
  });
</script>

<div class="flex flex-col h-full bg-zinc-950">
  <div class="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 shrink-0">
    <button class="text-xs text-zinc-500 hover:text-zinc-300 transition-colors" onclick={onBack}>
      ← Back
    </button>
    <span class="text-sm text-zinc-300 font-medium">{scenario.name}</span>
    {#if runComplete}
      <span class="text-xs text-green-400 ml-auto">Complete</span>
    {:else}
      <span class="text-xs text-cyan-400 ml-auto animate-pulse">Running…</span>
    {/if}
  </div>

  <div class="flex flex-1 overflow-hidden">
    <!-- Steps stepper -->
    <div class="flex-1 overflow-y-auto px-4 py-4 space-y-2">
      {#each scenario.steps as step, i (step.id)}
        {@const status = stepStatuses[step.id] ?? { state: 'waiting' as StepState }}
        <div class="flex gap-3">
          <!-- Step indicator -->
          <div class="flex flex-col items-center shrink-0 pt-2">
            <div class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium
              {status.state === 'waiting' ? 'bg-zinc-700 text-zinc-400'
                : status.state === 'running' ? 'bg-cyan-900 text-cyan-300'
                : status.state === 'success' ? 'bg-green-900 text-green-300'
                : 'bg-red-900 text-red-300'}">
              {#if status.state === 'running'}
                <span class="animate-spin text-xs">◌</span>
              {:else if status.state === 'success'}
                ✓
              {:else if status.state === 'error'}
                ✗
              {:else}
                {i + 1}
              {/if}
            </div>
            {#if i < scenario.steps.length - 1}
              <div class="w-px flex-1 mt-1 bg-zinc-800 min-h-4"></div>
            {/if}
          </div>

          <!-- Step content -->
          <div class="flex-1 pb-4">
            {#if status.result}
              <StepResultCard {step} result={status.result} />
            {:else}
              <div class="border border-zinc-800 rounded bg-zinc-900 px-3 py-2">
                <span class="text-sm {status.state === 'running' ? 'text-cyan-300' : 'text-zinc-500'}">{step.name}</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Variable state panel -->
    <div class="w-56 border-l border-zinc-800 px-3 py-4 overflow-y-auto shrink-0">
      <p class="text-xs text-zinc-500 uppercase tracking-wider mb-3">Variables</p>
      <VariableStatePanel variables={currentVariables} />
    </div>
  </div>
</div>
