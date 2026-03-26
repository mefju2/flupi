<script lang="ts">
  import { goto } from '$app/navigation';
  import { project } from '$lib/stores/project';
</script>

<header class="flex items-center h-10 px-4 border-b border-zinc-800 bg-zinc-900 shrink-0">
  <span class="text-sm font-semibold text-zinc-100 mr-4">Flupi</span>

  {#if $project.name}
    <span class="text-sm text-zinc-400 font-mono truncate max-w-xs">{$project.name}</span>
  {/if}

  <div class="ml-auto flex items-center gap-2">
    <!-- Environment switcher placeholder - implemented in Phase 2 -->
    <select class="text-xs bg-zinc-800 border border-zinc-700 text-zinc-300 rounded px-2 py-1" disabled>
      <option>No environment</option>
    </select>

    <button
      class="text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
      onclick={() => goto('/')}
    >
      Switch Project
    </button>
  </div>
</header>
