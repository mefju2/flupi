<script lang="ts">
  import { onMount } from 'svelte';
  import { beforeNavigate, goto } from '$app/navigation';
  import { project } from '$lib/stores/project';
  import { activeScenario, activeScenarioId, scenarioTree } from '$lib/stores/scenarios';
  import { activeEnvironment } from '$lib/stores/environment';
  import { requestTree } from '$lib/stores/requests';
  import {
    saveScenario, runScenario, getScenario, type ScenarioData,
  } from '$lib/services/tauri-commands';
  import { evaluateFunctionCalls } from '$lib/services/function-evaluator';
  import { functions } from '$lib/stores/functions';
  import { fade } from 'svelte/transition';
  import ScenarioTree from '$lib/components/scenarios/ScenarioTree.svelte';
  import ScenarioEditor from '$lib/components/scenarios/ScenarioEditor.svelte';
  import ScenarioRunner from '$lib/components/scenarios/ScenarioRunner.svelte';
  import EmptyState from '$lib/components/shared/EmptyState.svelte';
  import UnsavedChangesDialog from '$lib/components/scenarios/UnsavedChangesDialog.svelte';

  type View = 'editor' | 'runner';
  let view = $state<View>('editor');
  let saveToast = $state<string | null>(null);

  type PendingAction =
    | { type: 'navigate'; url: string }
    | { type: 'selectScenario'; id: string }
    | { type: 'run' };

  let isDirty = $state(false);
  let pendingAction = $state<PendingAction | null>(null);

  $effect(() => {
    if ($activeScenarioId) view = 'editor';
  });

  onMount(() => {
    const onRunShortcut = () => { if ($activeScenario) handleRunGuard(); };
    const onSave = () => handleSave();
    window.addEventListener('flupi:run-scenario', onRunShortcut);
    window.addEventListener('flupi:save', onSave);
    return () => {
      window.removeEventListener('flupi:run-scenario', onRunShortcut);
      window.removeEventListener('flupi:save', onSave);
    };
  });

  beforeNavigate(({ cancel, to }) => {
    if (isDirty && to) {
      cancel();
      pendingAction = { type: 'navigate', url: to.url.pathname };
    }
  });

  async function handleSave() {
    if (!$project.path || !$activeScenarioId || !$activeScenario) return;
    try {
      await saveScenario($project.path, $activeScenarioId, $activeScenario);
      isDirty = false;
      saveToast = 'Saved';
      setTimeout(() => (saveToast = null), 2000);
    } catch (e) {
      console.error('Failed to save scenario:', e);
    }
  }

  async function handleRun(inputs: Record<string, string>) {
    if (!$project.path || !$activeScenarioId) return;
    const envFile = $activeEnvironment ?? '';

    // Collect all string templates across all steps so function calls can be
    // pre-evaluated once for the entire scenario run (same call → same value).
    const scenario = $activeScenario;
    const templates = scenario ? [
      ...Object.values(inputs),
      ...scenario.steps.flatMap((s) => Object.values(s.overrides ?? {})),
    ] : [];

    let injectedVars: Record<string, string> = {};
    try {
      injectedVars = await evaluateFunctionCalls(templates, $functions);
    } catch (e) {
      console.error('Function evaluation failed:', e);
      return {};
    }

    // Build resolved inputs for display: replace {{$fn()}} tokens with their evaluated values
    const resolvedInputs = Object.fromEntries(
      Object.entries(inputs).map(([k, v]) => [
        k,
        v.replace(/\{\{(\$[a-zA-Z_$][a-zA-Z0-9_$]*\([^)]*\))\}\}/g, (_, token) => injectedVars[token] ?? v),
      ])
    );

    await runScenario($project.path, $activeScenarioId, envFile, inputs, 30000, injectedVars);
    return resolvedInputs;
  }

  function handleScenarioUpdate(updated: ScenarioData) {
    activeScenario.set(updated);
    isDirty = true;
  }

  async function handleSelectScenario(id: string) {
    if (isDirty) {
      pendingAction = { type: 'selectScenario', id };
      return;
    }
    await loadScenario(id);
  }

  async function loadScenario(id: string) {
    if (!$project.path) return;
    activeScenarioId.set(id);
    try {
      activeScenario.set(await getScenario($project.path, id));
      isDirty = false;
    } catch (e) {
      console.error('Failed to load scenario:', e);
    }
  }

  function handleRunGuard() {
    if (isDirty) {
      pendingAction = { type: 'run' };
      return;
    }
    view = 'runner';
  }

  async function handleDialogSave() {
    await handleSave();
    await executePendingAction();
  }

  async function handleDialogDiscard() {
    isDirty = false;
    await executePendingAction();
  }

  function handleDialogCancel() {
    pendingAction = null;
  }

  async function executePendingAction() {
    const action = pendingAction;
    pendingAction = null;
    if (!action) return;
    if (action.type === 'navigate') {
      await goto(action.url);
    } else if (action.type === 'selectScenario') {
      await loadScenario(action.id);
    } else if (action.type === 'run') {
      view = 'runner';
    }
  }
</script>

<div class="flex h-full">
  <!-- Sidebar -->
  <div class="w-56 shrink-0 border-r border-app-border">
    <ScenarioTree onSelect={handleSelectScenario} />
  </div>

  <!-- Main area -->
  <div class="flex-1 flex flex-col min-w-0 relative">
    {#if saveToast}
      <div transition:fade={{ duration: 150 }} class="absolute top-2 right-17.5 z-10 text-xs text-cyan-400 bg-app-card border border-app-border-2 rounded px-3 py-1">
        {saveToast}
      </div>
    {/if}

    {#if pendingAction}
      <UnsavedChangesDialog
        onSave={handleDialogSave}
        onDiscard={handleDialogDiscard}
        onCancel={handleDialogCancel}
      />
    {/if}

    {#if !$activeScenario}
      <EmptyState message="Select a scenario to edit." centered />
    {:else if view === 'editor'}
      <ScenarioEditor
        scenario={$activeScenario}
        onUpdate={handleScenarioUpdate}
        onSave={handleSave}
        onRun={handleRunGuard}
        {isDirty}
      />
    {:else}
      <ScenarioRunner
        scenario={$activeScenario}
        onBack={() => (view = 'editor')}
        onRun={handleRun}
      />
    {/if}
  </div>
</div>
