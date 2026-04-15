<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import type {
    ScenarioData,
    StepResult,
  } from "$lib/services/tauri-commands";
  import { isDelayStep, isPauseStep, resumeScenario } from "$lib/services/tauri-commands";
  import StepResultCard from "./StepResultCard.svelte";
  import VariableStatePanel from "./VariableStatePanel.svelte";
  import SectionHeader from "$lib/components/shared/SectionHeader.svelte";
  import ToolBar from "$lib/components/shared/ToolBar.svelte";
  import ResizablePanel from "$lib/components/shared/ResizablePanel.svelte";

  interface Props {
    scenario: ScenarioData;
    inputs?: Record<string, string>;
    onBack: () => void;
    onRetry: () => void;
    hasFunctionInputs?: boolean;
    onRetryFresh?: () => void;
  }

  let { scenario, inputs = {}, onBack, onRetry, hasFunctionInputs = false, onRetryFresh }: Props = $props();

  type StepState = "waiting" | "running" | "paused" | "success" | "error" | "skipped";

  interface StepStatus {
    state: StepState;
    result?: StepResult;
  }

  let stepStatuses = $state<Record<string, StepStatus>>(
    untrack(() => Object.fromEntries(
      scenario.steps.map((s) => [s.id, { state: "waiting" as StepState }]),
    )),
  );
  let currentVariables = $state<Record<string, string>>({});
  let runComplete = $state(false);
  let runFailed = $derived(
    Object.values(stepStatuses).some((s) => s.state === "error"),
  );
  let isPaused = $derived(
    Object.values(stepStatuses).some((s) => s.state === "paused"),
  );

  let unlisten: (() => void) | null = null;
  let unlistenPause: (() => void) | null = null;

  async function handleResume() {
    await resumeScenario(true).catch(() => {});
  }

  onMount(async () => {
    unlistenPause = await listen<string>("scenario-paused", (event) => {
      stepStatuses = {
        ...stepStatuses,
        [event.payload]: { state: "paused" },
      };
    });

    unlisten = await listen<StepResult>("scenario-step-result", (event) => {
      const result = event.payload;
      const state: StepState =
        result.status === "success" ? "success" : "error";
      let newStatuses = {
        ...stepStatuses,
        [result.step_id]: { state, result },
      };
      currentVariables = { ...currentVariables, ...result.extracted };

      if (result.status === "error") {
        // Mark all subsequent steps as skipped and stop the run
        const failedIndex = scenario.steps.findIndex(
          (s) => s.id === result.step_id,
        );
        for (let i = failedIndex + 1; i < scenario.steps.length; i++) {
          newStatuses[scenario.steps[i].id] = { state: "skipped" };
        }
        stepStatuses = newStatuses;
        runComplete = true;
        return;
      }

      stepStatuses = newStatuses;

      // Check if all steps are done (success path)
      const allDone = scenario.steps.every((s) => {
        const status = newStatuses[s.id];
        return (
          status &&
          (status.state === "success" ||
            status.state === "error" ||
            status.state === "skipped")
        );
      });
      if (allDone) runComplete = true;
    });

    // Mark first step as running (backend will drive actual progress via events)
    if (scenario.steps.length > 0) {
      const firstId = scenario.steps[0].id;
      stepStatuses = { ...stepStatuses, [firstId]: { state: "running" } };
    }
  });

  onDestroy(() => {
    unlisten?.();
    unlistenPause?.();
    resumeScenario(false).catch(() => {});
  });

  $effect(() => {
    // When a step succeeds, mark the next waiting step as running
    for (let i = 0; i < scenario.steps.length - 1; i++) {
      const current = stepStatuses[scenario.steps[i].id];
      const next = stepStatuses[scenario.steps[i + 1].id];
      if (current?.state === "success" && next?.state === "waiting") {
        stepStatuses = {
          ...stepStatuses,
          [scenario.steps[i + 1].id]: { state: "running" },
        };
        break;
      }
    }
  });
</script>

<div class="flex flex-col h-full bg-app-bg">
  {#snippet retryButtons()}
    {#if hasFunctionInputs}
      <button
        class="text-xs px-2.5 py-1 rounded border border-app-border-2 text-app-text-2 hover:bg-app-hover transition-colors"
        title="Re-run with the same resolved values from the previous run"
        onclick={onRetry}>↺ Retry (same)</button
      >
      <button
        class="text-xs px-2.5 py-1 rounded border border-app-border-2 text-app-text-2 hover:bg-app-hover transition-colors"
        title="Re-run and re-evaluate all JS functions to generate new values"
        onclick={onRetryFresh}>↺ Retry (fresh)</button
      >
    {:else}
      <button
        class="text-xs px-2.5 py-1 rounded border border-app-border-2 text-app-text-2 hover:bg-app-hover transition-colors"
        onclick={onRetry}>↺ Retry</button
      >
    {/if}
  {/snippet}
  <ToolBar>
    <button
      class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
      onclick={onBack}
    >
      ← Back
    </button>
    <span class="text-sm text-app-text-2 font-medium">{scenario.name}</span>
    {#if runComplete}
      {#if runFailed}
        <span class="text-xs text-red-400 ml-auto">Failed</span>
      {:else}
        <span class="text-xs text-green-400 ml-auto">Completed</span>
      {/if}
      {@render retryButtons()}
    {:else if isPaused}
      <span class="text-xs text-amber-400 ml-auto">Paused</span>
      <button
        class="text-xs px-2.5 py-1 rounded border border-amber-600 text-amber-300 hover:bg-amber-900/30 transition-colors"
        onclick={handleResume}>▶ Resume</button
      >
      {@render retryButtons()}
    {:else}
      <span class="text-xs text-cyan-400 ml-auto animate-pulse">Running…</span>
    {/if}
  </ToolBar>

  <div class="flex flex-1 overflow-hidden">
    <!-- Steps stepper -->
    <div class="flex-1 overflow-y-auto px-4 py-4 space-y-2">
      {#each scenario.steps as step, i (step.id)}
        {@const status = stepStatuses[step.id] ?? {
          state: "waiting" as StepState,
        }}
        <div class="flex gap-3">
          <!-- Step indicator -->
          <div class="flex flex-col items-center shrink-0 pt-2">
            <div
              class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium
              {status.state === 'waiting'
                ? 'bg-app-hover text-app-text-3'
                : status.state === 'running'
                  ? 'bg-cyan-900 text-cyan-300'
                  : status.state === 'paused'
                    ? 'bg-amber-900 text-amber-300'
                    : status.state === 'success'
                      ? 'bg-green-900 text-green-300'
                      : status.state === 'skipped'
                        ? 'bg-app-hover text-app-text-4'
                        : 'bg-red-900 text-red-300'}"
            >
              {#if status.state === "running"}
                <span class="animate-spin text-xs">◌</span>
              {:else if status.state === "paused"}
                ⏸
              {:else if status.state === "success"}
                ✓
              {:else if status.state === "error"}
                ✗
              {:else if status.state === "skipped"}
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
              <div
                class="border border-app-border rounded bg-app-panel px-3 py-2
                  {status.state === 'running' ? 'ring-1 ring-cyan-500/30' : ''}
                  {status.state === 'paused' ? 'ring-1 ring-amber-500/30' : ''}"
              >
                <span
                  class="text-sm {status.state === 'running'
                    ? 'text-cyan-300'
                    : status.state === 'paused'
                      ? 'text-amber-300'
                      : status.state === 'skipped'
                        ? 'text-app-text-4 line-through'
                        : 'text-app-text-3'}"
                >{step.name}{#if isDelayStep(step)}<span class="text-app-text-4"> — ⏱ {step.duration}ms</span>{:else if isPauseStep(step)}<span class="text-app-text-4"> — ⏸</span>{/if}</span
                >
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Variable state panel -->
    <ResizablePanel
      defaultWidth={224}
      minWidth={150}
      maxWidth={450}
      side="left"
      storageKey="flupi:sidebar-runner"
    >
      <div class="px-3 py-4 space-y-4">
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
    </ResizablePanel>
  </div>
</div>
