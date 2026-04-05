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
  import { listEnvironments, getPreferences, getRecentProjects, listOpenApiSources, refreshSource, listFunctions, getResolvedVariables, loadRequestTree, loadScenarioTree } from '$lib/services/tauri-commands';
  import { openApiSources, driftedIdsBySource } from '$lib/stores/openapi';
  import { functions } from '$lib/stores/functions';
  import { requestTree } from '$lib/stores/requests';
  import { scenarioTree } from '$lib/stores/scenarios';

  let driftScanCancelled = false;

  onMount(async () => {
    if (!$project.isOpen) { goto('/'); return; }

    // Clear stale drift state from any previous project session.
    driftedIdsBySource.set(new Map());

    // Capture path once so the background scan always uses this project's path,
    // even if the store changes before the async work completes.
    const projectPath = $project.path!;

    const [prefs, entries, fnList, reqTree, scTree] = await Promise.all([
      getPreferences(),
      listEnvironments(projectPath),
      listFunctions(projectPath),
      loadRequestTree(projectPath),
      loadScenarioTree(projectPath),
    ]);
    requestTree.set(reqTree);
    scenarioTree.set(scTree);

    theme.set(prefs.theme as Theme);

    const secretMaps = await Promise.all(
      entries.map(async ([fileName, environment]) => {
        if (!environment.secrets.length) return {};
        try {
          const resolved = await getResolvedVariables(projectPath, fileName);
          return Object.fromEntries(environment.secrets.map((k) => [k, resolved[k] ?? '']));
        } catch {
          return {};
        }
      })
    );

    const envList = entries.map(([fileName, environment], i) => ({ fileName, environment, secrets: secretMaps[i] }));
    environments.set(envList);
    functions.set(fnList);
    if (envList.length > 0 && $activeEnvironment === null) {
      const { projects } = await getRecentProjects();
      const stored = projects.find((p) => p.path === projectPath)?.activeEnvironment ?? null;
      const validStored = stored && envList.some((e) => e.fileName === stored) ? stored : envList[0].fileName;
      activeEnvironment.set(validStored);
      selectedEnvironmentFile.set(validStored);
    }

    // Startup drift scan — runs in the background without blocking the UI.
    (async () => {
      try {
        const sources = await listOpenApiSources(projectPath);
        if (driftScanCancelled) return;
        openApiSources.set(sources);
        await Promise.all(
          sources.map(async (source) => {
            try {
              const drifted = await refreshSource(projectPath, source.id);
              if (driftScanCancelled) return;
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
  onDestroy(() => {
    driftScanCancelled = true;
    cleanupShortcuts?.();
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
