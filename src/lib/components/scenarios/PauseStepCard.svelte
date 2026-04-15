<script lang="ts">
  import type { PauseStep } from '$lib/services/tauri-commands';

  interface Props {
    step: PauseStep;
    index: number;
    onUpdate: (step: PauseStep) => void;
    onDelete: () => void;
    onMoveUp?: () => void;
    onMoveDown?: () => void;
  }

  let { step, index, onUpdate, onDelete, onMoveUp, onMoveDown }: Props = $props();

  let expanded = $state(false);
</script>

<div class="border border-app-border rounded bg-app-panel mb-2 {expanded ? 'border-l-2 border-l-app-border-2' : 'border-l-2 border-l-transparent'}">
  <div
    class="flex items-center gap-2 px-3 py-2 cursor-pointer select-none"
    role="button"
    tabindex="0"
    onclick={() => expanded = !expanded}
    onkeydown={(e) => e.key === 'Enter' && (expanded = !expanded)}
  >
    <div class="flex flex-col gap-0.5 shrink-0" onclick={(e) => e.stopPropagation()} role="none">
      <button
        type="button"
        aria-label="Move step up"
        class="text-app-text-4 hover:text-app-text-3 transition-colors bg-transparent border-0 p-0 leading-none text-[10px] {onMoveUp ? '' : 'opacity-40 cursor-not-allowed'}"
        disabled={!onMoveUp}
        onclick={() => onMoveUp?.()}
      >▲</button>
      <button
        type="button"
        aria-label="Move step down"
        class="text-app-text-4 hover:text-app-text-3 transition-colors bg-transparent border-0 p-0 leading-none text-[10px] {onMoveDown ? '' : 'opacity-40 cursor-not-allowed'}"
        disabled={!onMoveDown}
        onclick={() => onMoveDown?.()}
      >▼</button>
    </div>

    <span class="text-xs text-app-text-3 w-5 shrink-0">{index + 1}</span>

    <div class="flex-1 flex items-center gap-2 min-w-0">
      <span class="font-mono text-sm text-app-text truncate">{step.name || 'Pause'}</span>
      <span class="text-xs text-app-text-4 shrink-0">⏸</span>
    </div>

    <div class="flex items-center gap-2 shrink-0">
      <button
        type="button"
        class="text-app-text-4 hover:text-red-400 transition-colors text-base"
        onclick={(e) => { e.stopPropagation(); onDelete(); }}
        aria-label="Delete pause step"
      >×</button>
    </div>
  </div>

  {#if expanded}
    <div class="px-3 pb-3 border-t border-app-border space-y-3 pt-3">
      <div class="flex items-center gap-3">
        <label for="pause-name-{step.id}" class="text-xs text-app-text-3 w-16 shrink-0">Name</label>
        <input
          id="pause-name-{step.id}"
          class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
          value={step.name}
          oninput={(e) => onUpdate({ ...step, name: e.currentTarget.value })}
          placeholder="Pause"
        />
      </div>
    </div>
  {/if}
</div>
