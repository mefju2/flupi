<script lang="ts">
  import { onMount } from 'svelte';
  import { environments, activeEnvironment, selectedEnvironmentFile } from '$lib/stores/environment';
  import { listEnvironments, saveEnvironment, deleteEnvironment, getRecentProjects, setProjectActiveEnvironment, getResolvedVariables, duplicateEnvironment, renameEnvironment } from '$lib/services/tauri-commands';
  import EmptyState from '$lib/components/shared/EmptyState.svelte';
  import { project } from '$lib/stores/project';

  let creatingNew = false;
  let newName = '';
  let inputEl: HTMLInputElement;

  let renamingFileName: string | null = null;
  let renameValue = '';
  let renameInputEl: HTMLInputElement;

  onMount(async () => {
    if ($project.path) {
      try {
        const entries = await listEnvironments($project.path);
        const existing = $environments;
        const projectPath = $project.path;
        const secretMaps = await Promise.all(
          entries.map(async ([fileName, environment]) => {
            const cached = existing.find((e) => e.fileName === fileName);
            if (cached) return cached.secrets;
            if (!environment.secrets.length) return {};
            try {
              const resolved = await getResolvedVariables(projectPath, fileName);
              return Object.fromEntries(environment.secrets.map((k) => [k, resolved[k] ?? '']));
            } catch {
              return {};
            }
          })
        );
        environments.set(
          entries.map(([fileName, environment], i) => ({
            fileName,
            environment,
            secrets: secretMaps[i],
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

  async function duplicateEnv(fileName: string) {
    if (!$project.path) return;
    try {
      const newFileName = await duplicateEnvironment($project.path, fileName);
      const entries = await listEnvironments($project.path);
      environments.set(
        entries.map(([fn, environment]) => {
          const existing = $environments.find((e) => e.fileName === fn);
          return { fileName: fn, environment, secrets: existing?.secrets ?? {} };
        })
      );
      selectedEnvironmentFile.set(newFileName);
    } catch (e) {
      console.error('Failed to duplicate environment:', e);
    }
  }

  function startRenaming(fileName: string, currentName: string) {
    renamingFileName = fileName;
    renameValue = currentName;
    setTimeout(() => renameInputEl?.focus(), 0);
  }

  async function confirmRename() {
    if (!$project.path || !renamingFileName || !renameValue.trim()) { cancelRename(); return; }

    const oldFileName = renamingFileName;
    const trimmed = renameValue.trim();
    renamingFileName = null;

    try {
      const newFileName = await renameEnvironment($project.path, oldFileName, trimmed);
      environments.update((list) =>
        list.map((e) =>
          e.fileName === oldFileName
            ? { ...e, fileName: newFileName, environment: { ...e.environment, name: trimmed } }
            : e
        )
      );
      if ($selectedEnvironmentFile === oldFileName) selectedEnvironmentFile.set(newFileName);
      if ($activeEnvironment === oldFileName) activeEnvironment.set(newFileName);
    } catch (e) {
      console.error('Failed to rename environment:', e);
    }
  }

  function cancelRename() {
    renamingFileName = null;
    renameValue = '';
  }
</script>

<div class="flex flex-col h-full bg-app-panel">
  <div class="px-3 py-2 text-xs text-app-text-3 uppercase tracking-wider border-b border-app-border">
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
        {#if renamingFileName === entry.fileName}
          <input
            bind:this={renameInputEl}
            bind:value={renameValue}
            class="flex-1 min-w-0 bg-app-card text-app-text text-xs px-1 py-0.5 rounded outline-none border border-app-border-2 focus:border-cyan-500 font-mono"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => { e.stopPropagation(); if (e.key === 'Enter') confirmRename(); else if (e.key === 'Escape') cancelRename(); }}
            onblur={confirmRename}
          />
        {:else}
          <span
            class="truncate"
            ondblclick={(e) => { e.stopPropagation(); startRenaming(entry.fileName, entry.environment.name); }}
            title="Double-click to rename"
          >{entry.environment.name}</span>
        {/if}
        <div class="flex items-center gap-1 shrink-0 ml-2">
          <button
            class="text-sm w-5 h-5 flex items-center justify-center rounded transition-colors {$activeEnvironment === entry.fileName
              ? 'text-cyan-400'
              : 'opacity-0 group-hover:opacity-100 hover:text-cyan-400 text-app-text-2'}"
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
            class="opacity-0 group-hover:opacity-100 text-app-text-2 hover:text-cyan-400 transition-opacity text-sm w-5 h-5 flex items-center justify-center"
            onclick={(e) => { e.stopPropagation(); duplicateEnv(entry.fileName); }}
            title="Duplicate environment"
            aria-label="Duplicate environment"
          >⧉</button>
          <button
            class="opacity-0 group-hover:opacity-100 text-app-text-2 hover:text-red-400 transition-opacity text-base w-6 h-6 flex items-center justify-center"
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
