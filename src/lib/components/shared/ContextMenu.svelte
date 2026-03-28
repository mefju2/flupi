<script lang="ts">
  import { onMount } from 'svelte';

  interface MenuItem {
    label: string;
    action: () => void;
    danger?: boolean;
  }

  interface Props {
    items: MenuItem[];
    x: number;
    y: number;
    onClose: () => void;
  }

  let { items, x, y, onClose }: Props = $props();

  let menuEl: HTMLElement | undefined = $state();

  onMount(() => {
    function handleClick(e: MouseEvent) {
      if (menuEl && !menuEl.contains(e.target as Node)) {
        onClose();
      }
    }
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    document.addEventListener('mousedown', handleClick);
    document.addEventListener('keydown', handleKey);
    return () => {
      document.removeEventListener('mousedown', handleClick);
      document.removeEventListener('keydown', handleKey);
    };
  });
</script>

<div
  bind:this={menuEl}
  class="fixed z-50 min-w-40 bg-app-panel border border-app-border-2 rounded shadow-lg py-1"
  style="left: {x}px; top: {y}px;"
  role="menu"
>
  {#each items as item}
    <button
      class="w-full text-left px-3 py-1.5 text-sm
        {item.danger ? 'text-red-400 hover:bg-red-950/40' : 'text-app-text-2 hover:bg-app-card'}
        transition-colors"
      role="menuitem"
      onclick={() => { item.action(); onClose(); }}
    >
      {item.label}
    </button>
  {/each}
</div>
