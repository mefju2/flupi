<script lang="ts">
  import type { RecentProject } from '$lib/services/tauri-commands';

  interface Props {
    projects: RecentProject[];
    onSelect: (project: RecentProject) => void;
    onRemove?: (project: RecentProject) => void;
  }

  let { projects, onSelect, onRemove }: Props = $props();
</script>

<div class="w-full max-w-md">
  <h2 class="text-xs font-medium text-app-text-3 uppercase tracking-wider mb-2">Recent Projects</h2>
  <ul class="space-y-1">
    {#each projects as p}
      <li class="relative group">
        <button
          type="button"
          class="w-full text-left px-3 py-2 rounded hover:bg-app-card transition-colors duration-150 cursor-pointer"
          onclick={() => onSelect(p)}
        >
          <div class="font-medium text-app-text pr-6">{p.name}</div>
          <div class="text-xs text-app-text-3 group-hover:text-app-text-2 font-mono truncate transition-colors duration-150 pr-6">{p.path}</div>
        </button>
        {#if onRemove}
          <button
            type="button"
            class="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded text-app-text-4 opacity-0 group-hover:opacity-100 focus-visible:opacity-100 hover:text-app-text hover:bg-app-hover transition-all duration-150"
            aria-label="Remove from recent projects"
            onclick={(e) => { e.stopPropagation(); onRemove?.(p); }}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        {/if}
      </li>
    {/each}
  </ul>
</div>
