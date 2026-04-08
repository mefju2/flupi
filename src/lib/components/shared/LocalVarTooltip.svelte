<script lang="ts">
  import { onMount, tick } from 'svelte';

  interface Props {
    varName: string;
    anchorEl: HTMLElement;
    onclose: () => void;
    onmouseenter?: () => void;
    onmouseleave?: () => void;
  }

  let { varName, anchorEl, onclose, onmouseenter, onmouseleave }: Props = $props();

  let tooltipEl = $state<HTMLElement | null>(null);
  let style = $state('position: fixed; opacity: 0; top: 0; left: 0;');

  onMount(() => {
    tick().then(() => {
      const rect = anchorEl.getBoundingClientRect();
      const tipHeight = tooltipEl?.offsetHeight ?? 72;
      const tipWidth = tooltipEl?.offsetWidth ?? 220;

      const top =
        rect.top - tipHeight - 6 > 0 ? rect.top - tipHeight - 6 : rect.bottom + 6;
      const left = Math.max(8, Math.min(rect.left, window.innerWidth - tipWidth - 8));
      style = `position: fixed; top: ${top}px; left: ${left}px;`;
    });
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={tooltipEl}
  {style}
  class="w-56 bg-app-panel border border-app-border-2 rounded shadow-lg p-3 text-sm z-50 pointer-events-none"
  {onmouseenter}
  {onmouseleave}
>
  <div class="flex items-center justify-between mb-1.5">
    <span class="font-mono text-app-text-3 text-xs">{`{{${varName}}}`}</span>
    <span class="text-xs px-1.5 py-0.5 rounded bg-violet-500/10 text-violet-400">
      Scenario Variable
    </span>
  </div>
  <p class="text-app-text-4 text-xs">Extracted at runtime by a preceding step.</p>
</div>
