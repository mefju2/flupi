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
          class="w-full text-left px-3 py-2 rounded hover:bg-zinc-800 transition-colors group"
          onclick={() => onSelect(p)}
        >
          <div class="font-medium text-zinc-100">{p.name}</div>
          <div class="text-xs text-zinc-500 font-mono truncate">{p.path}</div>
        </button>
      </li>
    {/each}
  </ul>
</div>
