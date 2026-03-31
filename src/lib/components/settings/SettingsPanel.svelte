<script lang="ts">
  import { onMount } from 'svelte';
  import SavedIndicator from '$lib/components/shared/SavedIndicator.svelte';
  import { project } from '$lib/stores/project';
  import { theme, type Theme } from '$lib/stores/ui';
  import { getPreferences, savePreferences } from '$lib/services/tauri-commands';

  let timeoutMs = $state(30000);
  let savedTimeout = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    const prefs = await getPreferences();
    timeoutMs = prefs.defaultTimeoutMs;
    theme.set(prefs.theme as Theme);
  });

  async function handleThemeChange(value: Theme) {
    theme.set(value);
    await savePreferences({ theme: value, defaultTimeoutMs: timeoutMs });
  }

  function handleTimeoutInput(e: Event) {
    const raw = (e.target as HTMLInputElement).valueAsNumber;
    if (!Number.isFinite(raw) || raw <= 0) return;
    timeoutMs = raw;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      await savePreferences({ theme: $theme, defaultTimeoutMs: timeoutMs });
      savedTimeout = true;
      setTimeout(() => (savedTimeout = false), 2000);
    }, 300);
  }

  const themeOptions: { value: Theme; label: string }[] = [
    { value: 'dark', label: 'Dark' },
    { value: 'light', label: 'Light' },
    { value: 'system', label: 'System' },
  ];
</script>

<div class="max-w-xl mx-auto p-6 flex flex-col gap-6">
  <h1 class="text-lg font-semibold text-app-text mb-6">Settings</h1>

  <section>
    <h2 class="text-sm font-semibold text-app-text-2 mb-4">Project</h2>
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-sm text-app-text-2">Name</label>
        <div class="font-mono text-sm text-app-text-3 py-1">
          {$project.name ?? '—'}
        </div>
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm text-app-text-2">Folder</label>
        <div class="font-mono text-sm text-app-text-3 py-1 truncate">
          {$project.path ?? '—'}
        </div>
      </div>
    </div>
  </section>

  <div class="border-t border-app-border"></div>

  <section>
    <h2 class="text-sm font-semibold text-app-text-2 mb-4">App</h2>
    <div class="flex flex-col gap-5">
      <div class="flex flex-col gap-2">
        <label class="text-sm text-app-text-2">Theme</label>
        <div class="flex gap-2">
          {#each themeOptions as opt}
            <button
              type="button"
              class="px-4 py-1.5 rounded text-sm transition-colors {$theme === opt.value
                ? 'bg-cyan-500 text-zinc-900 font-medium'
                : 'bg-app-card text-app-text-2 hover:bg-app-hover'}"
              onclick={() => handleThemeChange(opt.value)}
            >
              {opt.label}
            </button>
          {/each}
        </div>
        <p class="text-xs text-app-text-3 mt-1">Follows your operating system appearance preference.</p>
      </div>

      <div class="flex flex-col gap-2">
        <label for="timeout" class="text-sm text-app-text-2">Default request timeout (ms)</label>
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
        <p class="text-xs text-app-text-3 mt-1">Applied to requests without an explicit timeout configured.</p>
      </div>
    </div>
  </section>
</div>
