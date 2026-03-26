<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  const sections = [
    { path: '/environments', label: 'Environments', icon: '⚙' },
    { path: '/openapi',      label: 'OpenAPI',      icon: '📄' },
    { path: '/requests',     label: 'Requests',     icon: '↗' },
    { path: '/scenarios',    label: 'Scenarios',    icon: '▶' },
    { path: '/settings',     label: 'Settings',     icon: '🔧' },
  ];
</script>

<nav class="flex flex-col w-14 border-r border-zinc-800 bg-zinc-900 shrink-0">
  {#each sections as s}
    <button
      class="flex items-center justify-center h-14 text-lg transition-colors
        {$page.url.pathname.startsWith(s.path)
          ? 'text-cyan-400 bg-zinc-800'
          : 'text-zinc-600 hover:text-zinc-300 hover:bg-zinc-800/50'}"
      onclick={() => goto(s.path)}
      title={s.label}
    >
      {s.icon}
    </button>
  {/each}
</nav>
