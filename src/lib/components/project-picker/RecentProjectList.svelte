<script lang="ts">
  import type { RecentProject } from '$lib/services/tauri-commands';

  interface Props {
    projects: RecentProject[];
    onSelect: (project: RecentProject) => void;
  }

  let { projects, onSelect }: Props = $props();
</script>

<div class="w-full max-w-md">
  <h2 class="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">Recent Projects</h2>
  <ul class="space-y-1">
    {#each projects as p}
      <li>
        <button
          class="w-full text-left px-3 py-2 rounded hover:bg-zinc-800 transition-colors duration-150 group cursor-pointer"
          onclick={() => onSelect(p)}
        >
          <div class="font-medium text-zinc-100">{p.name}</div>
          <div class="text-xs text-zinc-500 group-hover:text-zinc-300 font-mono truncate transition-colors duration-150">{p.path}</div>
        </button>
      </li>
    {/each}
  </ul>
</div>
