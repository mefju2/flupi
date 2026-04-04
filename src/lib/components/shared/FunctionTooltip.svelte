<script lang="ts">
  import type { ScriptFunction } from '$lib/services/tauri-commands';
  import { onMount, tick } from 'svelte';

  interface Props {
    fn: ScriptFunction;
    anchorEl: HTMLElement;
    onclose: () => void;
    onmouseenter?: () => void;
    onmouseleave?: () => void;
  }

  let { fn, anchorEl, onclose, onmouseenter, onmouseleave }: Props = $props();

  let tooltipEl = $state<HTMLElement | null>(null);
  let style = $state('position: fixed; opacity: 0; top: 0; left: 0;');

  const callSignature = $derived(
    fn.params && fn.params.length > 0
      ? `{{$${fn.name}(${fn.params.map((p) => p.name).join(', ')})}}`
      : `{{$${fn.name}()}}`
  );

  onMount(() => {
    tick().then(() => {
      const rect = anchorEl.getBoundingClientRect();
      const tipHeight = tooltipEl?.offsetHeight ?? 100;
      const tipWidth = tooltipEl?.offsetWidth ?? 220;

      let top: number;
      if (rect.top - tipHeight - 6 > 0) {
        top = rect.top - tipHeight - 6;
      } else {
        top = rect.bottom + 6;
      }
      const left = Math.max(8, Math.min(rect.left, window.innerWidth - tipWidth - 8));
      style = `position: fixed; top: ${top}px; left: ${left}px;`;
    });

    let skipFirst = true;
    function handleClickOutside(e: PointerEvent) {
      if (skipFirst) { skipFirst = false; return; }
      if (tooltipEl && !tooltipEl.contains(e.target as Node)) {
        onclose();
      }
    }

    document.addEventListener('pointerdown', handleClickOutside);
    return () => document.removeEventListener('pointerdown', handleClickOutside);
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={tooltipEl}
  {style}
  role="tooltip"
  class="min-w-48 bg-app-panel border border-app-border-2 rounded shadow-lg p-3 text-sm z-50"
  {onmouseenter}
  {onmouseleave}
>
  <div class="flex items-center justify-between mb-2">
    <span class="font-mono text-cyan-400 text-xs truncate">{callSignature}</span>
    <button
      class="text-app-text-4 hover:text-app-text leading-none ml-2 shrink-0"
      onclick={onclose}
      aria-label="Close"
    >×</button>
  </div>

  {#if fn.params && fn.params.length > 0}
    <ul class="space-y-1">
      {#each fn.params as param}
        <li class="flex items-center gap-2">
          <span class="font-mono text-app-text text-xs">{param.name}</span>
          <span class="text-xs px-1.5 py-0.5 rounded bg-app-card text-app-text-3">{param.param_type}</span>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="text-app-text-4 text-xs italic">No parameters</p>
  {/if}
</div>
