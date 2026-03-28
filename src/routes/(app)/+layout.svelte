<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import TopBar from '$lib/components/layout/TopBar.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import SearchModal from '$lib/components/shared/SearchModal.svelte';
  import { registerShortcuts } from '$lib/services/keyboard-shortcuts';
  import { searchOpen } from '$lib/stores/ui';
  import { project } from '$lib/stores/project';

  onMount(() => {
    if (!$project.isOpen) goto('/');

    return registerShortcuts([
      { key: 'Enter', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:send-request')) },
      { key: 's', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:save')) },
      { key: 'n', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:new-request')) },
      { key: 'p', ctrl: true, handler: () => searchOpen.set(true) },
      { key: 'e', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:switch-environment')) },
      { key: 'Enter', ctrl: true, shift: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:run-scenario')) },
    ]);
  });
</script>

<div class="flex flex-col h-screen bg-app-bg text-app-text">
  <TopBar />
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-auto">
      <slot />
    </main>
  </div>
  <SearchModal />
</div>
