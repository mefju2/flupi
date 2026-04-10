<script lang="ts">
  import { relaunch } from "@tauri-apps/plugin-process";
  import { pendingUpdate } from "$lib/stores/updates";

  let downloading = $state(false);
  let downloaded = $state(0);
  let contentLength = $state(0);
  let dismissed = $state(false);
  let error = $state<string | null>(null);

  const progress = $derived(
    contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0,
  );

  async function install() {
    if (!$pendingUpdate) return;
    downloading = true;
    downloaded = 0;
    contentLength = 0;
    error = null;
    try {
      await $pendingUpdate.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            break;
        }
      });
      // On Windows the app exits automatically here.
      await relaunch();
    } catch (e) {
      downloading = false;
      error = "Install failed. Please try again.";
    }
  }
</script>

{#if $pendingUpdate && !dismissed}
  <div
    class="shrink-0 border-b border-app-border bg-app-panel px-4 py-2 flex items-center gap-3 text-xs"
  >
    {#if downloading}
      <div class="flex-1 flex items-center gap-3">
        <span class="text-app-text-3 shrink-0">
          Downloading {$pendingUpdate.version}…
        </span>
        <div
          class="flex-1 max-w-48 h-1 rounded-full bg-app-card overflow-hidden"
        >
          <div
            class="h-full bg-cyan-500 transition-all duration-200 rounded-full"
            style="width: {progress}%"
          ></div>
        </div>
        {#if contentLength > 0}
          <span class="text-app-text-4 font-mono shrink-0">{progress}%</span>
        {/if}
      </div>
    {:else}
      <span class="w-1.5 h-1.5 rounded-full bg-cyan-500 shrink-0"></span>
      <span class="text-app-text-2">
        Flupi <span class="font-mono text-app-text"
          >{$pendingUpdate.version}</span
        > is available
      </span>
      {#if $pendingUpdate.body}
        <span class="text-app-text-4 hidden sm:inline truncate max-w-xs">
          · {$pendingUpdate.body}
        </span>
      {/if}
      <div class="ml-auto flex items-center gap-2 shrink-0">
        {#if error}
          <span class="text-red-400">{error}</span>
        {/if}
        <button
          onclick={install}
          class="px-3 py-1 rounded bg-cyan-500 text-zinc-900 font-medium hover:bg-cyan-400 transition-colors"
        >
          Install &amp; Restart
        </button>
        <button
          onclick={() => (dismissed = true)}
          aria-label="Dismiss"
          class="text-app-text-4 hover:text-app-text-3 px-1 py-1 rounded transition-colors"
        >
          ×
        </button>
      </div>
    {/if}
  </div>
{/if}
