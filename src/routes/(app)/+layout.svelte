<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import TopBar from '$lib/components/layout/TopBar.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import SearchModal from '$lib/components/shared/SearchModal.svelte';
  import { registerShortcuts } from '$lib/services/keyboard-shortcuts';
  import { searchOpen, theme, type Theme } from '$lib/stores/ui';
  import { project } from '$lib/stores/project';
  import { environments, activeEnvironment, selectedEnvironmentFile } from '$lib/stores/environment';
  import { listEnvironments, getPreferences } from '$lib/services/tauri-commands';

  onMount(async () => {
    if (!$project.isOpen) { goto('/'); return; }

    const [prefs, entries] = await Promise.all([
      getPreferences(),
      listEnvironments($project.path!),
    ]);

    theme.set(prefs.theme as Theme);

    const envList = entries.map(([fileName, environment]) => ({ fileName, environment, secrets: {} }));
    environments.set(envList);
    if (envList.length > 0 && $activeEnvironment === null) {
      activeEnvironment.set(envList[0].fileName);
      selectedEnvironmentFile.set(envList[0].fileName);
    }

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
