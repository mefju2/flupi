<script lang="ts">
  import { goto } from '$app/navigation';
  import { project } from '$lib/stores/project';
  import { environments, activeEnvironment } from '$lib/stores/environment';
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
        bind:value={$activeEnvironment}
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

    <button
      class="text-xs text-app-text-4 hover:text-app-text-3 px-1.5 py-1 rounded transition-colors"
      title="Keyboard shortcuts: Ctrl+P Search, Ctrl+N New request, Ctrl+S Save, Ctrl+Enter Send, Ctrl+E Switch env, Ctrl+Shift+Enter Run scenario"
      aria-label="Keyboard shortcuts"
    >
      ?
    </button>
  </div>
</header>
