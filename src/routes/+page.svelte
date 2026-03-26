<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import RecentProjectList from '$lib/components/project-picker/RecentProjectList.svelte';
  import {
    getRecentProjects,
    createProject,
    openProject,
    addRecentProject,
    pickFolder,
  } from '$lib/services/tauri-commands';
  import { project } from '$lib/stores/project';
  import type { RecentProject } from '$lib/services/tauri-commands';

  let recentProjects: RecentProject[] = $state([]);
  let error = $state('');

  onMount(async () => {
    const data = await getRecentProjects();
    recentProjects = data.projects;
  });

  async function openAndNavigate(path: string, name: string) {
    project.set({ isOpen: true, path, name });
    await addRecentProject(name, path);
    goto('/requests');
  }

  async function handleNewProject() {
    const path = await pickFolder();
    if (!path) return;
    try {
      await createProject(path);
      await openAndNavigate(path, path.split('/').pop() || path);
    } catch (e) {
      error = String(e);
    }
  }

  async function handleOpenFolder() {
    const path = await pickFolder();
    if (!path) return;
    try {
      const state = await openProject(path);
      if (state === 'empty') {
        if (!confirm("This folder doesn't appear to be a Flupi project. Initialize it?")) return;
        await createProject(path);
      }
      await openAndNavigate(path, path.split('/').pop() || path);
    } catch (e) {
      error = String(e);
    }
  }

  async function handleSelectRecent(p: RecentProject) {
    try {
      const state = await openProject(p.path);
      if (state === 'empty') {
        if (!confirm("This folder doesn't appear to be a Flupi project. Initialize it?")) return;
        await createProject(p.path);
      }
      await openAndNavigate(p.path, p.name);
    } catch (e) {
      error = String(e);
      recentProjects = recentProjects.filter((rp) => rp.path !== p.path);
    }
  }
</script>

<div class="flex flex-col items-center justify-center h-screen bg-zinc-950 text-zinc-100">
  <div class="mb-8 text-center">
    <h1 class="text-2xl font-semibold tracking-tight mb-1">Flupi</h1>
    <p class="text-sm text-zinc-500">API test scenarios, powered by Git</p>
  </div>

  {#if error}
    <p class="text-red-400 text-sm mb-4 max-w-sm text-center">{error}</p>
  {/if}

  <div class="flex gap-3 mb-10">
    <button
      class="px-4 py-2 bg-cyan-500 hover:bg-cyan-400 text-zinc-950 font-medium text-sm rounded transition-colors"
      onclick={handleNewProject}
    >
      New Project
    </button>
    <button
      class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 font-medium text-sm rounded transition-colors"
      onclick={handleOpenFolder}
    >
      Open Folder
    </button>
  </div>

  {#if recentProjects.length > 0}
    <RecentProjectList projects={recentProjects} onSelect={handleSelectRecent} />
  {:else}
    <p class="text-zinc-600 text-sm">No recent projects</p>
  {/if}
</div>
