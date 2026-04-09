<script lang="ts">
  import { GitBranch, Download, ArrowDown, AlertTriangle } from 'lucide-svelte';
  import type { GitStatus } from '$lib/services/tauri-commands';

  interface Props {
    status: GitStatus;
    isFetching: boolean;
    isPulling: boolean;
    conflictError: string | null;
    error: string | null;
    onfetch: () => void;
    onpull: () => void;
  }

  let { status, isFetching, isPulling, conflictError, error, onfetch, onpull }: Props = $props();
</script>

<!-- Branch -->
<section class="flex flex-col gap-1 shrink-0">
  <div class="flex items-center gap-1.5">
    <GitBranch size={13} class="text-app-text-3 shrink-0" />
    <span class="font-mono text-xs text-app-text truncate">{status.branch}</span>
    {#if status.ahead > 0 || status.behind > 0}
      <span class="font-mono text-xs text-app-text-2 ml-auto shrink-0">
        {#if status.ahead > 0}↑{status.ahead}{/if}{#if status.behind > 0}↓{status.behind}{/if}
      </span>
    {/if}
  </div>
  {#if status.upstream}
    <p class="text-xs text-app-text-3 truncate pl-4">{status.upstream}</p>
  {/if}
</section>

<!-- Actions -->
<section class="flex gap-2 shrink-0">
  <button
    class="flex items-center gap-1 px-2.5 py-1.5 rounded text-xs bg-app-card text-app-text-2
           hover:bg-app-hover border border-app-border transition-colors disabled:opacity-50"
    onclick={onfetch}
    disabled={isFetching || isPulling}
  >
    <Download size={12} class={isFetching ? 'animate-spin' : ''} />
    {isFetching ? 'Fetching…' : 'Fetch'}
  </button>
  <button
    class="flex items-center gap-1 px-2.5 py-1.5 rounded text-xs bg-cyan-500 text-zinc-900
           font-medium hover:bg-cyan-400 transition-colors disabled:opacity-50"
    onclick={onpull}
    disabled={isPulling || isFetching || !status.upstream || status.behind === 0}
  >
    <ArrowDown size={12} class={isPulling ? 'animate-spin' : ''} />
    {isPulling ? 'Pulling…' : 'Pull'}
  </button>
</section>

{#if conflictError}
  <div class="flex items-start gap-1.5 px-2.5 py-2 rounded bg-yellow-500/10 border border-yellow-500/30 text-yellow-300 text-xs shrink-0">
    <AlertTriangle size={13} class="shrink-0 mt-0.5" />
    <p>{conflictError}</p>
  </div>
{/if}

{#if error}
  <div class="px-2.5 py-2 rounded bg-red-500/10 border border-red-500/30 text-red-400 text-xs font-mono shrink-0">
    {error}
  </div>
{/if}
