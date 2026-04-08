<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import SearchModal from "$lib/components/shared/SearchModal.svelte";
  import { registerShortcuts } from "$lib/services/keyboard-shortcuts";
  import { searchOpen, theme, type Theme } from "$lib/stores/ui";
  import { project } from "$lib/stores/project";
  import {
    environments,
    activeEnvironment,
    selectedEnvironmentFile,
  } from "$lib/stores/environment";
  import {
    listEnvironments,
    getPreferences,
    getRecentProjects,
    listOpenApiSources,
    refreshSource,
    listFunctions,
    getResolvedVariables,
    loadRequestTree,
    loadScenarioTree,
    setProjectActiveEnvironment,
  } from "$lib/services/tauri-commands";
  import { openApiSources, driftedIdsBySource } from "$lib/stores/openapi";
  import { functions } from "$lib/stores/functions";
  import { requestTree } from "$lib/stores/requests";
  import { scenarioTree, activeScenarioId } from "$lib/stores/scenarios";

  let driftScanCancelled = false;
  let cleanupShortcuts: (() => void) | undefined;

  onMount(async () => {
    if (!$project.isOpen) {
      goto("/");
      return;
    }

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
          return Object.fromEntries(
            environment.secrets.map((k) => [k, resolved[k] ?? ""]),
          );
        } catch {
          return {};
        }
      }),
    );

    const envList = entries.map(([fileName, environment], i) => ({
      fileName,
      environment,
      secrets: secretMaps[i],
    }));
    environments.set(envList);
    functions.set(fnList);
    if (envList.length > 0 && $activeEnvironment === null) {
      const { projects } = await getRecentProjects();
      const stored =
        projects.find((p) => p.path === projectPath)?.activeEnvironment ?? null;
      const validStored =
        stored && envList.some((e) => e.fileName === stored)
          ? stored
          : envList[0].fileName;
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
      {
        key: "n",
        ctrl: true,
        handler: () => {
          const path = get(page).url.pathname;
          if (path === "/requests")
            window.dispatchEvent(new CustomEvent("flupi:new-request"));
          else if (path === "/scenarios")
            window.dispatchEvent(new CustomEvent("flupi:new-scenario"));
          else if (path === "/functions")
            window.dispatchEvent(new CustomEvent("flupi:new-function"));
          else if (path === "/openapi")
            window.dispatchEvent(new CustomEvent("flupi:new-openapi-source"));
        },
      },
      {
        key: "s",
        ctrl: true,
        handler: () => {
          if (get(page).url.pathname === "/scenarios") {
            window.dispatchEvent(new CustomEvent("flupi:save"));
          }
        },
      },
      { key: "p", ctrl: true, handler: () => searchOpen.set(true) },
      {
        key: "Enter",
        ctrl: true,
        handler: () => {
          const path = get(page).url.pathname;
          if (path === "/requests")
            window.dispatchEvent(new CustomEvent("flupi:send-request"));
          else if (path === "/scenarios" && get(activeScenarioId) !== null) {
            window.dispatchEvent(new CustomEvent("flupi:run-scenario"));
          } else if (path === "/openapi") {
            window.dispatchEvent(new CustomEvent("flupi:sync-all"));
          }
        },
      },
      {
        key: "e",
        ctrl: true,
        handler: () => {
          const envs = get(environments);
          if (envs.length < 2) return;
          const current = get(activeEnvironment);
          if (!current) return;
          const idx = envs.findIndex((e) => e.fileName === current);
          const next = envs[(idx + 1) % envs.length];
          activeEnvironment.set(next.fileName);
          selectedEnvironmentFile.set(next.fileName);
          const projectPath = get(project).path;
          if (projectPath)
            setProjectActiveEnvironment(projectPath, next.fileName);
        },
      },
      {
        key: "F2",
        handler: () => {
          window.dispatchEvent(new CustomEvent("flupi:rename-active"));
        },
      },
    ]);
  });

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
