<script lang="ts">
  import { onMount } from "svelte";
  import SavedIndicator from "$lib/components/shared/SavedIndicator.svelte";
  import { project } from "$lib/stores/project";
  import { theme, type Theme } from "$lib/stores/ui";
  import {
    getPreferences,
    savePreferences,
  } from "$lib/services/tauri-commands";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { addRecentProject } from "$lib/services/tauri-commands";
  import { createDebouncedSave } from "$lib/services/debounced-save";
  import { gitAutoRefreshMs } from "$lib/stores/git";
  import {
    checkForUpdates,
    updateChecking,
    pendingUpdate,
  } from "$lib/stores/updates";

  let timeoutMs = $state(30000);
  let savedTimeout = $state(false);
  let gitRefreshSec = $state(30);
  let savedGitRefresh = $state(false);
  let projectName = $state($project.name ?? "");
  let savedProjectName = $state(false);

  const debouncedSaveName = createDebouncedSave(async () => {
    if (!$project.path || !projectName.trim()) return;
    await addRecentProject(projectName.trim(), $project.path);
    project.update((p) => ({ ...p, name: projectName.trim() }));
    savedProjectName = true;
    setTimeout(() => {
      savedProjectName = false;
    }, 2000);
  }, 400);

  function handleProjectNameInput(e: Event) {
    projectName = (e.target as HTMLInputElement).value;
    debouncedSaveName.trigger();
  }

  const debouncedSave = createDebouncedSave(async () => {
    await savePreferences({
      theme: $theme,
      defaultTimeoutMs: timeoutMs,
      gitAutoRefreshMs: gitRefreshSec * 1000,
    });
    gitAutoRefreshMs.set(gitRefreshSec * 1000);
    savedTimeout = true;
    savedGitRefresh = true;
    setTimeout(() => {
      savedTimeout = false;
      savedGitRefresh = false;
    }, 2000);
  }, 300);

  onMount(async () => {
    const prefs = await getPreferences();
    timeoutMs = prefs.defaultTimeoutMs;
    gitRefreshSec = Math.round((prefs.gitAutoRefreshMs ?? 30000) / 1000);
    theme.set(prefs.theme as Theme);
  });

  async function handleThemeChange(value: Theme) {
    theme.set(value);
    await savePreferences({
      theme: value,
      defaultTimeoutMs: timeoutMs,
      gitAutoRefreshMs: gitRefreshSec * 1000,
    });
  }

  function handleTimeoutInput(e: Event) {
    const raw = (e.target as HTMLInputElement).valueAsNumber;
    if (!Number.isFinite(raw) || raw <= 0) return;
    timeoutMs = raw;
    debouncedSave.trigger();
  }

  function handleGitRefreshInput(e: Event) {
    const raw = (e.target as HTMLInputElement).valueAsNumber;
    if (!Number.isFinite(raw) || raw <= 0 || raw < 5) return;
    gitRefreshSec = raw;
    debouncedSave.trigger();
  }

  const themeOptions: { value: Theme; label: string }[] = [
    { value: "dark", label: "Dark" },
    { value: "light", label: "Light" },
    { value: "system", label: "System" },
  ];
</script>

<div class="max-w-xl mx-auto p-6 flex flex-col gap-6">
  <h1 class="text-lg font-semibold text-app-text mb-6">Settings</h1>

  <section>
    <h2 class="text-sm font-semibold text-app-text-2 mb-4">Project</h2>
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1">
        <label for="project-name" class="text-sm text-app-text-2">Name</label>
        <div class="flex items-center">
          <input
            id="project-name"
            type="text"
            value={projectName}
            oninput={handleProjectNameInput}
            class="w-60 px-3 py-2 rounded bg-app-card text-sm text-app-text font-mono
                   border border-transparent focus:border-cyan-500 focus:outline-none transition-colors"
          />
          <SavedIndicator visible={savedProjectName} class="ml-2" />
        </div>
      </div>
      <div class="flex flex-col gap-1">
        <p class="text-sm text-app-text-2">Folder</p>
        <div class="flex items-center gap-2">
          <div class="font-mono text-sm text-app-text-3 py-1 truncate flex-1">
            {$project.path ?? "—"}
          </div>
          {#if $project.path}
            <button
              type="button"
              onclick={() => revealItemInDir($project.path!)}
              title="Open in file explorer"
              class="shrink-0 px-2 py-1 rounded text-xs bg-app-card text-app-text-2 hover:bg-app-hover transition-colors"
            >
              Open
            </button>
          {/if}
        </div>
      </div>
    </div>
  </section>

  <div class="border-t border-app-border"></div>

  <section>
    <h2 class="text-sm font-semibold text-app-text-2 mb-4">App</h2>
    <div class="flex flex-col gap-5">
      <div class="flex flex-col gap-2">
        <p class="text-sm text-app-text-2">Theme</p>
        <div class="flex gap-2">
          {#each themeOptions as opt}
            <button
              type="button"
              class="px-4 py-1.5 rounded text-sm transition-colors {$theme ===
              opt.value
                ? 'bg-cyan-500 text-zinc-900 font-medium'
                : 'bg-app-card text-app-text-2 hover:bg-app-hover'}"
              onclick={() => handleThemeChange(opt.value)}
            >
              {opt.label}
            </button>
          {/each}
        </div>
        <p class="text-xs text-app-text-3 mt-1">
          Follows your operating system appearance preference.
        </p>
      </div>

      <div class="flex flex-col gap-2">
        <label for="timeout" class="text-sm text-app-text-2"
          >Default request timeout (ms)</label
        >
        <div class="flex items-center">
          <input
            id="timeout"
            type="number"
            min="100"
            step="500"
            value={timeoutMs}
            oninput={handleTimeoutInput}
            class="w-40 px-3 py-2 rounded bg-app-card text-sm text-app-text font-mono
                   border border-transparent focus:border-cyan-500 focus:outline-none transition-colors"
          />
          <SavedIndicator visible={savedTimeout} class="ml-2" />
        </div>
        <p class="text-xs text-app-text-3 mt-1">
          Applied to requests without an explicit timeout configured.
        </p>
      </div>
    </div>
  </section>

  <div class="border-t border-app-border"></div>

  <section>
    <h2 class="text-sm font-semibold text-app-text-2 mb-4">Git</h2>
    <div class="flex flex-col gap-5">
      <div class="flex flex-col gap-2">
        <label for="git-refresh" class="text-sm text-app-text-2"
          >Auto-refresh interval (seconds)</label
        >
        <div class="flex items-center">
          <input
            id="git-refresh"
            type="number"
            min="5"
            step="5"
            value={gitRefreshSec}
            oninput={handleGitRefreshInput}
            class="w-40 px-3 py-2 rounded bg-app-card text-sm text-app-text font-mono
                   border border-transparent focus:border-cyan-500 focus:outline-none transition-colors"
          />
          <SavedIndicator visible={savedGitRefresh} class="ml-2" />
        </div>
        <p class="text-xs text-app-text-3 mt-1">
          How often the Git status page refreshes automatically.
        </p>
      </div>
    </div>
  </section>

  <div class="border-t border-app-border"></div>

  <section>
    <h2 class="text-sm font-semibold text-app-text-2 mb-4">Updates</h2>
    <div class="flex flex-col gap-3">
      <div class="flex items-center gap-3">
        <button
          type="button"
          onclick={checkForUpdates}
          disabled={$updateChecking}
          class="px-4 py-2 rounded text-sm bg-app-card text-app-text-2 hover:bg-app-hover
                 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {$updateChecking ? "Checking…" : "Check for Updates"}
        </button>
        {#if $pendingUpdate}
          <span class="text-xs text-cyan-400">
            {$pendingUpdate.version} available — see the banner above
          </span>
        {:else if !$updateChecking}
          <span class="text-xs text-app-text-4">You're on v0.1.7</span>
        {/if}
      </div>
      <p class="text-xs text-app-text-3">
        Current version: <span class="font-mono">0.1.7</span>
      </p>
    </div>
  </section>
</div>
