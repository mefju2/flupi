<script lang="ts">
  import { onDestroy } from "svelte";

  interface Props {
    defaultWidth: number;
    minWidth: number;
    maxWidth: number;
    side: "left" | "right";
    storageKey?: string;
    children: import("svelte").Snippet;
  }

  let { defaultWidth, minWidth, maxWidth, side, storageKey, children }: Props =
    $props();

  function readStoredWidth(): number {
    if (!storageKey) return defaultWidth;
    const stored = localStorage.getItem(storageKey);
    if (stored) {
      const n = Number(stored);
      if (!isNaN(n)) return Math.min(maxWidth, Math.max(minWidth, n));
    }
    return defaultWidth;
  }

  let panelWidth = $state(readStoredWidth());

  $effect(() => {
    if (storageKey) {
      localStorage.setItem(storageKey, String(panelWidth));
    }
  });

  // ── Resize logic ──────────────────────────────────────────────────────────
  let startX = 0;
  let startWidth = 0;

  function onMouseMove(e: MouseEvent) {
    // side='right': handle is on the right edge, drag right → wider
    // side='left': handle is on the left edge, drag left → wider
    const delta = side === "right" ? e.clientX - startX : startX - e.clientX;
    panelWidth = Math.min(maxWidth, Math.max(minWidth, startWidth + delta));
  }

  function onMouseUp() {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }

  function startResize(e: MouseEvent) {
    e.preventDefault();
    startX = e.clientX;
    startWidth = panelWidth;
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  onDestroy(() => {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  });
</script>

<div class="flex h-full shrink-0" style="width: {panelWidth}px">
  {#if side === "left"}
    <button
      class="w-1 cursor-col-resize shrink-0 border-l border-app-border hover:bg-app-hover transition-colors focus:outline-none"
      onmousedown={startResize}
      aria-label="Drag to resize panel"
    ></button>
  {/if}

  <div class="flex-1 overflow-y-auto min-w-0">
    {@render children()}
  </div>

  {#if side === "right"}
    <button
      class="w-1 cursor-col-resize shrink-0 border-r border-app-border hover:bg-app-hover transition-colors focus:outline-none"
      onmousedown={startResize}
      aria-label="Drag to resize panel"
    ></button>
  {/if}
</div>
