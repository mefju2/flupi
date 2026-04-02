<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import TopBar from '$lib/components/layout/TopBar.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import SearchModal from '$lib/components/shared/SearchModal.svelte';
  import { registerShortcuts } from '$lib/services/keyboard-shortcuts';
  import { searchOpen, theme, type Theme } from '$lib/stores/ui';
  import { project } from '$lib/stores/project';
  import { environments, activeEnvironment, selectedEnvironmentFile } from '$lib/stores/environment';
  import { listEnvironments, getPreferences, getRecentProjects, listOpenApiSources, refreshSource } from '$lib/services/tauri-commands';
  import { openApiSources, driftedIdsBySource } from '$lib/stores/openapi';

  onMount(async () => {
    if (!$project.isOpen) { goto('/'); return; }

    // Clear stale drift state from any previous project session.
    driftedIdsBySource.set(new Map());

    const [prefs, entries] = await Promise.all([
      getPreferences(),
      listEnvironments($project.path!),
    ]);

    theme.set(prefs.theme as Theme);

    const envList = entries.map(([fileName, environment]) => ({ fileName, environment, secrets: {} }));
    environments.set(envList);
    if (envList.length > 0 && $activeEnvironment === null) {
      const { projects } = await getRecentProjects();
      const stored = projects.find((p) => p.path === $project.path)?.activeEnvironment ?? null;
      const validStored = stored && envList.some((e) => e.fileName === stored) ? stored : envList[0].fileName;
      activeEnvironment.set(validStored);
      selectedEnvironmentFile.set(validStored);
    }

    // Startup drift scan — runs in the background without blocking the UI.
    (async () => {
      try {
        const sources = await listOpenApiSources($project.path!);
        openApiSources.set(sources);
        await Promise.all(
          sources.map(async (source) => {
            try {
              const drifted = await refreshSource($project.path!, source.id);
              driftedIdsBySource.update((prev) => {
                const next = new Map(prev);
                next.set(source.id, drifted);
                return next;
              });
            } catch {
              // A single source failing (e.g. unreachable URL) should not block others.
            }
          }),
        );
      } catch {
        // Sources file missing or unreadable — not a startup-blocking error.
      }
    })();

    cleanupShortcuts = registerShortcuts([
      { key: 'Enter', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:send-request')) },
      { key: 's', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:save')) },
      { key: 'n', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:new-request')) },
      { key: 'p', ctrl: true, handler: () => searchOpen.set(true) },
      { key: 'e', ctrl: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:switch-environment')) },
      { key: 'Enter', ctrl: true, shift: true, handler: () => window.dispatchEvent(new CustomEvent('flupi:run-scenario')) },
    ]);
  });

  let cleanupShortcuts: (() => void) | undefined;
  onDestroy(() => cleanupShortcuts?.());
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
