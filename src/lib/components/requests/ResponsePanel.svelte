<script lang="ts">
  import { onDestroy } from 'svelte';
  import { lastResponse, isExecuting, lastError } from '$lib/stores/execution';

  const COLLAPSED_STORAGE_KEY = 'flupi:response-panel-collapsed';

  let headersOpen = $state(false);
  let collapsed = $state(localStorage.getItem(COLLAPSED_STORAGE_KEY) === 'true');
  let panelHeight = $state(240);

  const MIN_HEIGHT = 100;
  const MAX_HEIGHT = 800;

  // Persist collapsed state to localStorage
  $effect(() => {
    localStorage.setItem(COLLAPSED_STORAGE_KEY, String(collapsed));
  });

  function statusClass(status: number): string {
    if (status >= 200 && status < 300) return 'bg-emerald-900 text-emerald-300 border-emerald-700';
    if (status >= 400 && status < 500) return 'bg-yellow-900 text-yellow-300 border-yellow-700';
    if (status >= 500) return 'bg-red-900 text-red-300 border-red-700';
    return 'bg-app-card text-app-text-2 border-app-border-2';
  }

  const MAX_BODY_BYTES = 1_048_576; // 1MB

  function formatBody(raw: string): string {
    if (raw.length > MAX_BODY_BYTES) {
      return raw.slice(0, MAX_BODY_BYTES) + '\n\n[Response truncated at 1MB]';
    }
    try {
      return JSON.stringify(JSON.parse(raw), null, 2);
    } catch {
      return raw;
    }
  }

  const formattedBody = $derived($lastResponse ? formatBody($lastResponse.body) : '');

  // ── Resize logic ──────────────────────────────────────────────────────────
  let startY = 0;
  let startHeight = 0;

  function onMouseMove(e: MouseEvent) {
    const delta = startY - e.clientY;
    panelHeight = Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, startHeight + delta));
  }

  function onMouseUp() {
    window.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('mouseup', onMouseUp);
  }

  function startResize(e: MouseEvent) {
    e.preventDefault();
    startY = e.clientY;
    startHeight = panelHeight;
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  onDestroy(() => {
    window.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('mouseup', onMouseUp);
  });
</script>

<div
  class="border-t border-app-border bg-app-bg flex flex-col"
  style="height: {collapsed ? 'auto' : panelHeight + 'px'}; flex-shrink: 0;"
>
  <!-- Resize handle -->
  {#if !collapsed}
    <button
      class="h-1 w-full cursor-row-resize hover:bg-app-hover transition-colors shrink-0 focus:outline-none"
      onmousedown={startResize}
      aria-label="Drag to resize response panel"
    ></button>
  {/if}

  <!-- Header bar -->
  <div class="flex items-center gap-2 px-3 h-8 shrink-0 border-b border-app-border select-none">
    <span class="text-xs text-app-text-3 font-medium">Response</span>
    {#if $lastResponse}
      <span class="text-xs font-mono px-1.5 py-px rounded border {statusClass($lastResponse.status)}">
        {$lastResponse.status} {$lastResponse.statusText}
      </span>
      <span class="text-xs text-app-text-4">{$lastResponse.durationMs}ms</span>
    {:else if $isExecuting}
      <span class="text-xs text-app-text-4">Sending…</span>
    {/if}
    <button
      class="ml-auto text-app-text-4 hover:text-app-text transition-colors p-1 rounded hover:bg-app-hover"
      onclick={() => (collapsed = !collapsed)}
      aria-label={collapsed ? 'Expand response panel' : 'Collapse response panel'}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        style="transform: rotate({collapsed ? '180deg' : '0deg'}); transition: transform 150ms ease;"
      >
        <polyline points="6 9 12 15 18 9"></polyline>
      </svg>
    </button>
  </div>

  <!-- Content -->
  {#if !collapsed}
    <div class="flex-1 overflow-y-auto">
      {#if $isExecuting}
        <div class="p-4 text-sm text-app-text-3">Sending...</div>
      {:else if !$lastResponse && $lastError}
        <div class="p-4 text-sm text-red-400"><span class="font-mono">Error: {$lastError}</span></div>
      {:else if !$lastResponse}
        <div class="p-4 text-sm text-app-text-4">Ready to send — press Ctrl+Enter or click Send</div>
      {:else}
        <div class="p-4 space-y-3">
          <div>
            <button
              class="text-xs text-app-text-3 hover:text-app-text hover:bg-app-card rounded px-2 transition-colors flex items-center gap-1 mb-1 -mx-2"
              onclick={() => (headersOpen = !headersOpen)}
            >
              <span>{headersOpen ? '▾' : '▸'}</span>
              Headers ({Object.keys($lastResponse.headers).length})
            </button>
            {#if headersOpen}
              <div class="space-y-0.5">
                {#each Object.entries($lastResponse.headers) as [k, v]}
                  <div class="flex gap-2 text-xs font-mono">
                    <span class="text-app-text-3 shrink-0">{k}:</span>
                    <span class="text-app-text-2 break-all">{v}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <div>
            <p class="text-xs text-app-text-3 mb-1">Body</p>
            <pre class="text-xs font-mono text-app-text bg-app-panel border border-app-border p-3 overflow-auto whitespace-pre-wrap break-all">{formattedBody}</pre>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
