<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import TopBar from '$lib/components/layout/TopBar.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import { project } from '$lib/stores/project';

  onMount(() => {
    if (!$project.isOpen) goto('/');
  });
</script>

<div class="flex flex-col h-screen bg-zinc-950 text-zinc-100">
  <TopBar />
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-auto">
      <slot />
    </main>
  </div>
</div>
