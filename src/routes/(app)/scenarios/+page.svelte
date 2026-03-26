<script lang="ts">
  import { onMount } from 'svelte';
  import { project } from '$lib/stores/project';
  import { activeScenario, activeScenarioId, scenarioTree } from '$lib/stores/scenarios';
  import { activeEnvironment } from '$lib/stores/environment';
  import { loadRequestTree } from '$lib/services/tauri-commands';
  import { requestTree } from '$lib/stores/requests';
  import {
    loadScenarioTree, saveScenario, runScenario, type ScenarioData,
  } from '$lib/services/tauri-commands';
  import ScenarioTree from '$lib/components/scenarios/ScenarioTree.svelte';
  import ScenarioEditor from '$lib/components/scenarios/ScenarioEditor.svelte';
  import ScenarioRunner from '$lib/components/scenarios/ScenarioRunner.svelte';

  type View = 'editor' | 'runner';
  let view = $state<View>('editor');
  let saveToast = $state<string | null>(null);

  onMount(async () => {
    if (!$project.path) return;
    try {
      const [tree, reqTree] = await Promise.all([
        loadScenarioTree($project.path),
        loadRequestTree($project.path),
      ]);
      scenarioTree.set(tree);
      requestTree.set(reqTree);
    } catch (e) {
      console.error('Failed to load scenario/request trees:', e);
    }
  });

  async function handleSave() {
    if (!$project.path || !$activeScenarioId || !$activeScenario) return;
    try {
      await saveScenario($project.path, $activeScenarioId, $activeScenario);
      saveToast = 'Saved';
      setTimeout(() => (saveToast = null), 2000);
    } catch (e) {
      console.error('Failed to save scenario:', e);
    }
  }

  async function handleRun(inputs: Record<string, string>) {
    if (!$project.path || !$activeScenarioId) return;
    const envFile = $activeEnvironment ?? '';
    await runScenario($project.path, $activeScenarioId, envFile, inputs);
  }

  function handleScenarioUpdate(updated: ScenarioData) {
    activeScenario.set(updated);
  }
</script>

<div class="flex h-full">
  <!-- Sidebar -->
  <div class="w-56 shrink-0 border-r border-zinc-800">
    <ScenarioTree />
  </div>

  <!-- Main area -->
  <div class="flex-1 flex flex-col min-w-0 relative">
    {#if saveToast}
      <div class="absolute top-2 right-4 z-10 text-xs text-cyan-400 bg-zinc-800 border border-zinc-700 rounded px-2 py-1">
        {saveToast}
      </div>
    {/if}

    {#if !$activeScenario}
      <div class="flex-1 flex items-center justify-center text-zinc-600 text-sm">
        Select a scenario to edit.
      </div>
    {:else if view === 'editor'}
      <ScenarioEditor
        scenario={$activeScenario}
        onUpdate={handleScenarioUpdate}
        onSave={handleSave}
        onRun={() => (view = 'runner')}
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
