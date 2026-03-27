<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { Settings2, FileCode, ArrowUpRight, Play, SlidersHorizontal } from 'lucide-svelte';

  const sections = [
    { path: '/environments', label: 'Environments', icon: Settings2 },
    { path: '/openapi',      label: 'OpenAPI',      icon: FileCode },
    { path: '/requests',     label: 'Requests',     icon: ArrowUpRight },
    { path: '/scenarios',    label: 'Scenarios',    icon: Play },
    { path: '/settings',     label: 'Settings',     icon: SlidersHorizontal },
  ];
</script>

<nav class="flex flex-col w-14 border-r border-zinc-800 bg-zinc-900 shrink-0">
  {#each sections as s}
    <div class="relative group/nav">
      <button
        class="flex items-center justify-center h-14 w-14 transition-colors
          {$page.url.pathname.startsWith(s.path)
            ? 'text-cyan-400 bg-zinc-800'
            : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50'}"
        onclick={() => goto(s.path)}
        aria-label={s.label}
      >
        <s.icon size={18} />
      </button>
      <div class="pointer-events-none absolute left-full top-1/2 -translate-y-1/2 ml-2 px-2 py-1
                  bg-zinc-800 border border-zinc-700 rounded text-xs text-zinc-200 whitespace-nowrap
                  opacity-0 group-hover/nav:opacity-100 transition-opacity duration-150 z-50">
        {s.label}
      </div>
    </div>
  {/each}
</nav>
