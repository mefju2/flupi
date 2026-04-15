<script lang="ts">
  import { goto } from '$app/navigation';
  import { project } from '$lib/stores/project';
  import { environments, activeEnvironment } from '$lib/stores/environment';
  import { setProjectActiveEnvironment } from '$lib/services/tauri-commands';

  function selectEnvironment(e: Event) {
    const fileName = (e.target as HTMLSelectElement).value;
    activeEnvironment.set(fileName);
    if ($project.path) {
      setProjectActiveEnvironment($project.path, fileName);
    }
  }

  let showShortcuts = $state(false);

  const shortcuts = [
    { keys: 'Ctrl+P', action: 'Search everything' },
    { keys: 'Ctrl+N', action: 'New (request / scenario / function / source)' },
    { keys: 'Ctrl+Enter', action: 'Send / Run / Sync' },
    { keys: 'Ctrl+S', action: 'Save (scenarios page)' },
    { keys: 'Ctrl+E', action: 'Next environment' },
    { keys: 'F2', action: 'Rename active item' },
  ];
</script>

<header class="flex items-center h-12 px-4 border-b border-app-border bg-app-panel shrink-0">
  <span class="text-sm text-app-text-4 mr-2">Flupi</span>

  {#if $project.name}
    <span class="font-mono text-sm font-medium text-app-text truncate max-w-xs">{$project.name}</span>
  {/if}

  <div class="ml-auto flex items-center gap-2">
    {#if $environments.length === 0}
      <select class="text-xs bg-app-card text-app-text-2 rounded px-2 py-1" disabled>
        <option>No environment</option>
      </select>
    {:else}
      <select
        class="text-xs bg-app-card text-app-text-2 rounded px-2 py-1"
        value={$activeEnvironment}
        onchange={selectEnvironment}
      >
        {#each $environments as env}
          <option value={env.fileName}>{env.environment.name}</option>
        {/each}
      </select>
    {/if}

    <button
      class="text-xs text-app-text-3 hover:text-app-text-2 hover:bg-app-card px-2 py-1 rounded transition-colors"
      onclick={() => goto('/')}
    >
      Switch Project
    </button>

    <div class="relative">
      <button
        class="text-xs text-app-text-4 hover:text-app-text-3 px-1.5 py-1 rounded transition-colors {showShortcuts ? 'bg-app-card text-app-text-3' : ''}"
        aria-label="Keyboard shortcuts"
        onclick={() => (showShortcuts = !showShortcuts)}
      >
        ?
      </button>

      {#if showShortcuts}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="fixed inset-0 z-40"
          onclick={() => (showShortcuts = false)}
          onkeydown={(e) => e.key === 'Escape' && (showShortcuts = false)}
        ></div>
        <div class="absolute right-0 top-full mt-1 z-50 bg-app-card border border-app-border rounded shadow-lg py-2 min-w-56">
          <p class="px-3 pb-2 text-xs font-medium text-app-text-3 border-b border-app-border mb-1">Keyboard Shortcuts</p>
          {#each shortcuts as s}
            <div class="flex items-center justify-between px-3 py-1 gap-6">
              <span class="font-mono text-xs text-cyan-400 shrink-0">{s.keys}</span>
              <span class="text-xs text-app-text-2">{s.action}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</header>
