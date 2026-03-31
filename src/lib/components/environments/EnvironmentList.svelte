<script lang="ts">
  import { onMount } from 'svelte';
  import { environments, activeEnvironment, selectedEnvironmentFile } from '$lib/stores/environment';
  import { listEnvironments, saveEnvironment, deleteEnvironment, getRecentProjects, setProjectActiveEnvironment } from '$lib/services/tauri-commands';
  import EmptyState from '$lib/components/shared/EmptyState.svelte';
  import { project } from '$lib/stores/project';

  let creatingNew = false;
  let newName = '';
  let inputEl: HTMLInputElement;

  onMount(async () => {
    if ($project.path) {
      try {
        const entries = await listEnvironments($project.path);
        environments.set(
          entries.map(([fileName, environment]) => ({
            fileName,
            environment,
            secrets: {},
          }))
        );

        if (entries.length > 0 && $activeEnvironment === null) {
          const { projects } = await getRecentProjects();
          const stored = projects.find((p) => p.path === $project.path)?.activeEnvironment ?? null;
          const validStored = stored && entries.some(([f]) => f === stored) ? stored : entries[0][0];
          activeEnvironment.set(validStored);
          selectedEnvironmentFile.set(validStored);
        } else if (entries.length > 0 && $selectedEnvironmentFile === null) {
          selectedEnvironmentFile.set($activeEnvironment ?? entries[0][0]);
        }
      } catch (e) {
        console.error('Failed to load environments:', e);
      }
    }
  });

  function startCreating() {
    creatingNew = true;
    newName = '';
    setTimeout(() => inputEl?.focus(), 0);
  }

  async function confirmCreate() {
    if (!$project.path || !newName.trim()) { cancelCreate(); return; }

    const trimmed = newName.trim();
    const fileName = `${trimmed.toLowerCase().replace(/\s+/g, '-')}.env.json`;
    const env = { name: trimmed, variables: {}, secrets: [] };

    creatingNew = false;
    newName = '';

    try {
      await saveEnvironment($project.path, fileName, env);
      environments.update((list) => [...list, { fileName, environment: env, secrets: {} }]);
      selectedEnvironmentFile.set(fileName);
    } catch (e) {
      console.error('Failed to create environment:', e);
    }
  }

  function cancelCreate() {
    creatingNew = false;
    newName = '';
  }

  async function removeEnvironment(fileName: string) {
    if (!$project.path) return;
    if (!confirm(`Delete environment "${fileName}"?`)) return;

    try {
      await deleteEnvironment($project.path, fileName);
      environments.update((list) => list.filter((e) => e.fileName !== fileName));
      if ($activeEnvironment === fileName) {
        activeEnvironment.set($environments[0]?.fileName ?? null);
      }
      if ($selectedEnvironmentFile === fileName) {
        selectedEnvironmentFile.set($environments[0]?.fileName ?? null);
      }
    } catch (e) {
      console.error('Failed to delete environment:', e);
    }
  }
</script>

<div class="flex flex-col h-full bg-app-panel">
  <div class="px-3 py-2 text-xs font-medium text-app-text-4 border-b border-app-border">
    Environments
  </div>

  <div class="flex-1 overflow-y-auto">
    {#each $environments as entry (entry.fileName)}
      <div
        class="group flex items-center justify-between px-3 py-2 text-sm cursor-pointer select-none
          {$selectedEnvironmentFile === entry.fileName
          ? 'border-l-2 border-cyan-500 bg-app-card text-app-text'
          : 'border-l-2 border-transparent text-app-text-2 hover:bg-app-card/50 hover:text-app-text'}"
        role="button"
        tabindex="0"
        onclick={() => selectedEnvironmentFile.set(entry.fileName)}
        onkeydown={(e) => e.key === 'Enter' && selectedEnvironmentFile.set(entry.fileName)}
      >
        <span class="truncate">{entry.environment.name}</span>
        <div class="flex items-center gap-1 shrink-0 ml-2">
          <button
            class="text-xs px-1 rounded transition-colors {$activeEnvironment === entry.fileName
              ? 'text-cyan-400'
              : 'opacity-0 group-hover:opacity-60 hover:!opacity-100 text-app-text-4 hover:text-cyan-400'}"
            onclick={(e) => {
                e.stopPropagation();
                activeEnvironment.set(entry.fileName);
                if ($project.path) {
                  setProjectActiveEnvironment($project.path, entry.fileName);
                }
              }}
            title={$activeEnvironment === entry.fileName ? 'Active environment' : 'Set as active'}
            aria-label="Set as active environment"
          >✓</button>
          <button
            class="opacity-0 group-hover:opacity-30 hover:!opacity-100 text-app-text-4 hover:text-red-400 transition-opacity text-base leading-none"
            onclick={(e) => { e.stopPropagation(); removeEnvironment(entry.fileName); }}
            aria-label="Delete environment"
          >×</button>
        </div>
      </div>
    {/each}

    {#if $environments.length === 0}
      <EmptyState message="No environments yet. Create one to manage variables like API keys and base URLs." />
    {/if}
  </div>

  <div class="border-t border-app-border px-3 py-2">
    {#if creatingNew}
      <input
        bind:this={inputEl}
        bind:value={newName}
        class="w-full bg-app-card text-app-text text-xs px-2 py-1 rounded outline-none border border-app-border-2 focus:border-cyan-500 font-mono"
        placeholder="Environment name"
        onkeydown={(e) => { if (e.key === 'Enter') confirmCreate(); else if (e.key === 'Escape') cancelCreate(); }}
        onblur={confirmCreate}
      />
    {:else}
      <button
        class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
        onclick={startCreating}
      >+ New Environment</button>
    {/if}
  </div>
</div>
