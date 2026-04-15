<script lang="ts">
  interface Props {
    message: string;
    detail?: string;
    confirmLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    message,
    detail,
    confirmLabel = "Confirm",
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) onCancel();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onCancel();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  role="none"
  onclick={handleBackdrop}
>
  <div class="bg-app-panel border border-app-border-2 rounded-lg p-5 w-72 flex flex-col gap-4 shadow-lg">
    <div class="flex flex-col gap-1">
      <p class="text-sm font-semibold text-app-text">{message}</p>
      {#if detail}
        <p class="text-xs text-app-text-3">{detail}</p>
      {/if}
    </div>
    <div class="flex gap-2 justify-end">
      <button
        class="px-3 py-1.5 text-xs bg-app-card hover:bg-app-hover text-app-text-2 rounded transition-colors"
        onclick={onCancel}
      >
        Cancel
      </button>
      <button
        class="px-3 py-1.5 text-xs bg-red-700 hover:bg-red-600 text-white rounded transition-colors"
        onclick={onConfirm}
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
