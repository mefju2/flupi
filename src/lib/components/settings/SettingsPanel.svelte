<script lang="ts">
  import { onMount } from 'svelte';
  import { project } from '$lib/stores/project';
  import { theme, type Theme } from '$lib/stores/ui';
  import { getPreferences, savePreferences } from '$lib/services/tauri-commands';

  let timeoutMs = $state(30000);
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
    }, 300);
  }

  const themeOptions: { value: Theme; label: string }[] = [
    { value: 'dark', label: 'Dark' },
    { value: 'light', label: 'Light' },
    { value: 'system', label: 'System' },
  ];
</script>

<div class="max-w-xl mx-auto p-8 flex flex-col gap-8">
  <section>
    <h2 class="text-xs font-semibold uppercase tracking-widest text-zinc-500 mb-4">Project</h2>
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-xs text-zinc-400">Name</label>
        <div class="px-3 py-2 rounded bg-zinc-800 text-sm text-zinc-300 font-mono select-all">
          {$project.name ?? '—'}
        </div>
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-xs text-zinc-400">Folder</label>
        <div class="px-3 py-2 rounded bg-zinc-800 text-sm text-zinc-300 font-mono select-all truncate">
          {$project.path ?? '—'}
        </div>
      </div>
    </div>
  </section>

  <div class="border-t border-zinc-800"></div>

  <section>
    <h2 class="text-xs font-semibold uppercase tracking-widest text-zinc-500 mb-4">App</h2>
    <div class="flex flex-col gap-5">
      <div class="flex flex-col gap-2">
        <label class="text-xs text-zinc-400">Theme</label>
        <div class="flex gap-2">
          {#each themeOptions as opt}
            <button
              type="button"
              class="px-4 py-1.5 rounded text-sm transition-colors {$theme === opt.value
                ? 'bg-cyan-500 text-zinc-900 font-medium'
                : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700'}"
              onclick={() => handleThemeChange(opt.value)}
            >
              {opt.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="flex flex-col gap-2">
        <label for="timeout" class="text-xs text-zinc-400">Default request timeout (ms)</label>
        <input
          id="timeout"
          type="number"
          min="100"
          step="500"
          value={timeoutMs}
          oninput={handleTimeoutInput}
          class="w-40 px-3 py-2 rounded bg-zinc-800 text-sm text-zinc-200 font-mono
                 border border-transparent focus:border-cyan-500 focus:outline-none transition-colors"
        />
      </div>
    </div>
  </section>
</div>
