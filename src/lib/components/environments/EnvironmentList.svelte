<script lang="ts">
  import { onMount } from 'svelte';
  import { environments, activeEnvironment } from '$lib/stores/environment';
  import { listEnvironments, saveEnvironment, deleteEnvironment } from '$lib/services/tauri-commands';
  import { project } from '$lib/stores/project';

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
          activeEnvironment.set(entries[0][0]);
        }
      } catch (e) {
        console.error('Failed to load environments:', e);
      }
    }
  });

  async function createEnvironment() {
    if (!$project.path) return;
    const name = prompt('Environment name:');
    if (!name || !name.trim()) return;

    const trimmed = name.trim();
    const fileName = `${trimmed.toLowerCase().replace(/\s+/g, '-')}.env.json`;
    const env = { name: trimmed, variables: {}, secrets: [] };

    try {
      await saveEnvironment($project.path, fileName, env);
      environments.update((list) => [...list, { fileName, environment: env, secrets: {} }]);
      activeEnvironment.set(fileName);
    } catch (e) {
      console.error('Failed to create environment:', e);
    }
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
    } catch (e) {
      console.error('Failed to delete environment:', e);
    }
  }
</script>

<div class="flex flex-col h-full bg-zinc-900">
  <div class="px-3 py-2 text-xs text-zinc-500 uppercase tracking-wider border-b border-zinc-800">
    Environments
  </div>

  <div class="flex-1 overflow-y-auto">
    {#each $environments as entry}
      <div
        class="group flex items-center justify-between px-3 py-2 text-sm cursor-pointer select-none
          {$activeEnvironment === entry.fileName
          ? 'bg-zinc-800 text-zinc-100'
          : 'text-zinc-300 hover:bg-zinc-800/50 hover:text-zinc-100'}"
        role="button"
        tabindex="0"
        onclick={() => activeEnvironment.set(entry.fileName)}
        onkeydown={(e) => e.key === 'Enter' && activeEnvironment.set(entry.fileName)}
      >
        <span class="truncate">{entry.environment.name}</span>
        <button
          class="opacity-0 group-hover:opacity-100 text-zinc-600 hover:text-red-400 transition-all text-base leading-none ml-2 shrink-0"
          onclick={(e) => { e.stopPropagation(); removeEnvironment(entry.fileName); }}
          aria-label="Delete environment"
        >×</button>
      </div>
    {/each}

    {#if $environments.length === 0}
      <p class="px-3 py-4 text-xs text-zinc-600">No environments yet.</p>
    {/if}
  </div>

  <div class="border-t border-zinc-800 px-3 py-2">
    <button
      class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
      onclick={createEnvironment}
    >+ New Environment</button>
  </div>
</div>
