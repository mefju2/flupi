<script lang="ts">
  interface Props {
    onSave: () => void;
    onDiscard: () => void;
    onCancel: () => void;
  }

  let { onSave, onDiscard, onCancel }: Props = $props();

  const titleId = `unsaved-title-${Math.random().toString(36).slice(2, 8)}`;
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onCancel()} />

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  role="dialog"
  aria-modal="true"
  aria-labelledby={titleId}
>
  <!-- Click outside = cancel -->
  <div class="absolute inset-0" onclick={onCancel} role="presentation"></div>

  <div class="relative bg-app-card border border-app-border rounded-md px-6 py-5 w-72 shadow-xl">
    <p id={titleId} class="text-sm font-medium text-app-text mb-1">Unsaved changes</p>
    <p class="text-xs text-app-text-3 mb-5">Save before leaving, or discard your changes.</p>
    <div class="flex gap-2 justify-end">
      <button
        class="px-3 py-1 text-xs bg-transparent border border-app-border-2 text-app-text-3 hover:text-app-text rounded transition-colors"
        onclick={onCancel}
      >Cancel</button>
      <button
        class="px-3 py-1 text-xs bg-transparent border border-app-border-2 text-red-400 hover:text-red-300 rounded transition-colors"
        onclick={onDiscard}
      >Discard</button>
      <button
        class="px-3 py-1 text-xs text-zinc-900 bg-cyan-400 hover:bg-cyan-300 rounded font-medium transition-colors"
        onclick={onSave}
      >Save</button>
    </div>
  </div>
</div>
