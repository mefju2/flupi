<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import RecentProjectList from "$lib/components/project-picker/RecentProjectList.svelte";
  import {
    getRecentProjects,
    createProject,
    openProject,
    addRecentProject,
    removeRecentProject,
    pickFolder,
  } from "$lib/services/tauri-commands";
  import { project } from "$lib/stores/project";
  import { scenarioTree, activeScenarioId, activeScenario } from "$lib/stores/scenarios";
  import { requestTree, activeRequestId, activeRequest, activeCollectionFolder, activeCollection } from "$lib/stores/requests";
  import { environments, activeEnvironment, selectedEnvironmentFile } from "$lib/stores/environment";
  import { lastResponse, isExecuting, lastError } from "$lib/stores/execution";
  import { gitPageState, gitBehindCount } from "$lib/stores/git";
  import { openApiSources, driftedIdsBySource } from "$lib/stores/openapi";
  import { functions, selectedFunctionName } from "$lib/stores/functions";
  import type { RecentProject } from "$lib/services/tauri-commands";

  let recentProjects: RecentProject[] = $state([]);
  let error = $state("");
  let loaded = $state(false);

  onMount(async () => {
    const data = await getRecentProjects();
    recentProjects = data.projects;
    loaded = true;
  });

  function pathToName(path: string): string {
    return (
      path
        .split(/[\/\\]/)
        .filter(Boolean)
        .pop() || path
    );
  }

  async function openAndNavigate(path: string, name: string) {
    // Reset all project-scoped stores so stale data from the previous project
    // doesn't bleed into the newly opened one.
    scenarioTree.set([]);
    activeScenarioId.set(null);
    activeScenario.set(null);
    requestTree.set([]);
    activeRequestId.set(null);
    activeRequest.set(null);
    activeCollectionFolder.set(null);
    activeCollection.set(null);
    environments.set([]);
    activeEnvironment.set(null);
    selectedEnvironmentFile.set(null);
    lastResponse.set(null);
    isExecuting.set(false);
    lastError.set(null);
    gitPageState.set({ status: null, isLoading: false, isFetching: false, isPulling: false, isPushing: false, isCommitting: false, isSwitchingBranch: false, error: null, conflictError: null, lastRefreshed: null, lastFetched: null, selectedFile: null, branches: [] });
    gitBehindCount.set(0);
    openApiSources.set([]);
    driftedIdsBySource.set(new Map());
    functions.set([]);
    selectedFunctionName.set(null);

    project.set({ isOpen: true, path, name });
    await addRecentProject(name, path);
    goto("/requests");
  }

  async function handleNewProject() {
    const path = await pickFolder();
    if (!path) return;
    try {
      await createProject(path);
      await openAndNavigate(path, pathToName(path));
    } catch (e) {
      error = String(e);
    }
  }

  async function handleOpenFolder() {
    const path = await pickFolder();
    if (!path) return;
    try {
      const state = await openProject(path);
      if (state === "empty") {
        if (
          !confirm(
            "This folder doesn't appear to be a Flupi project. Initialize it?",
          )
        )
          return;
        await createProject(path);
      }
      await openAndNavigate(path, pathToName(path));
    } catch (e) {
      error = String(e);
    }
  }

  async function handleSelectRecent(p: RecentProject) {
    try {
      const state = await openProject(p.path);
      if (state === "empty") {
        if (
          !confirm(
            "This folder doesn't appear to be a Flupi project. Initialize it?",
          )
        )
          return;
        await createProject(p.path);
      }
      await openAndNavigate(p.path, p.name || pathToName(p.path));
    } catch (e) {
      error = String(e);
      recentProjects = recentProjects.filter((rp) => rp.path !== p.path);
    }
  }

  async function handleRemoveRecent(p: RecentProject) {
    const previous = recentProjects;
    recentProjects = recentProjects.filter((rp) => rp.path !== p.path);
    try {
      await removeRecentProject(p.path);
    } catch (e) {
      recentProjects = previous;
      error = String(e);
    }
  }
</script>

<div
  class="flex flex-col items-center justify-center h-screen bg-app-bg text-app-text"
>
  <div class="mb-8 text-center">
    <h1 class="text-2xl font-semibold tracking-tight mb-1">Flupi</h1>
    <p class="text-sm text-app-text-3">API test scenarios, powered by Git</p>
  </div>

  {#if error}
    <div
      class="bg-red-950/50 border border-red-900 rounded px-3 py-2 text-sm text-red-400 mb-4 max-w-sm text-center"
    >
      {error}
    </div>
  {/if}

  <div class="flex gap-3 mb-10">
    <button
      class="px-4 py-2 bg-cyan-500 hover:bg-cyan-400 text-zinc-950 font-medium text-sm rounded transition-colors focus-visible:ring-2 focus-visible:ring-cyan-500 focus-visible:ring-offset-2 focus-visible:ring-offset-app-bg"
      onclick={handleNewProject}
    >
      New Project
    </button>
    <button
      class="px-4 py-2 bg-app-hover hover:bg-app-hover text-app-text font-medium text-sm rounded transition-colors focus-visible:ring-2 focus-visible:ring-cyan-500 focus-visible:ring-offset-2 focus-visible:ring-offset-app-bg"
      onclick={handleOpenFolder}
    >
      Open Folder
    </button>
  </div>

  {#if !loaded}
    <p class="text-xs text-app-text-3">Loading…</p>
  {:else if recentProjects.length > 0}
    <RecentProjectList
      projects={recentProjects}
      onSelect={handleSelectRecent}
      onRemove={handleRemoveRecent}
    />
  {:else}
    <p class="text-app-text-4 text-sm">No recent projects yet</p>
    <p class="text-xs text-app-text-3 mt-1">
      Create a project or open an existing folder to get started
    </p>
  {/if}
</div>
