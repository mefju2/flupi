<script lang="ts">
  import { ChevronDown, Check, GitBranch } from 'lucide-svelte';
  import type { BranchInfo } from '$lib/services/tauri-commands';

  interface Props {
    currentBranch: string;
    branches: BranchInfo[];
    isSwitching: boolean;
    onswitch: (branch: string) => void;
    onopen: () => void;
  }

  let { currentBranch, branches, isSwitching, onswitch, onopen }: Props = $props();

  let open = $state(false);
  let filter = $state('');

  const localBranches = $derived(
    branches.filter((b) => !b.isRemote && b.name.toLowerCase().includes(filter.toLowerCase()))
  );

  const remoteBranches = $derived(
    branches.filter((b) => b.isRemote && b.name.toLowerCase().includes(filter.toLowerCase()))
  );

  function toggle() {
    open = !open;
    if (open) {
      filter = '';
      onopen();
    }
  }

  function select(name: string) {
    open = false;
    if (name !== currentBranch) onswitch(name);
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') open = false; }} />

<div class="relative">
  <button
    class="flex items-center gap-1 px-2 py-1 rounded text-xs font-mono
           text-app-text-2 hover:bg-app-card hover:text-app-text transition-colors
           disabled:opacity-50"
    onclick={toggle}
    disabled={isSwitching}
    aria-label="Switch branch"
  >
    <GitBranch size={12} class="shrink-0" />
    <span class="max-w-[140px] truncate">{currentBranch}</span>
    <ChevronDown size={10} class="shrink-0 transition-transform {open ? 'rotate-180' : ''}" />
  </button>

  {#if open}
    <!-- Backdrop -->
    <button
      class="fixed inset-0 z-10 cursor-default"
      onclick={() => (open = false)}
      tabindex="-1"
      aria-hidden="true"
    ></button>

    <div
      class="absolute left-0 top-full mt-1 z-20 w-64 rounded border border-app-border
             bg-app-bg shadow-lg flex flex-col"
    >
      <div class="p-1.5 border-b border-app-border">
        <input
          class="w-full bg-app-card rounded px-2 py-1 text-xs text-app-text
                 placeholder:text-app-text-3 focus:outline-none"
          placeholder="Filter branches…"
          bind:value={filter}
          autofocus
        />
      </div>

      <div class="overflow-y-auto max-h-56 flex flex-col">
        {#if localBranches.length > 0}
          <p class="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-app-text-3">
            Local
          </p>
          {#each localBranches as branch}
            <button
              class="flex items-center gap-2 px-2 py-1.5 text-xs text-left
                     hover:bg-app-card transition-colors
                     {branch.isCurrent ? 'text-app-text' : 'text-app-text-2'}"
              onclick={() => select(branch.name)}
            >
              {#if branch.isCurrent}
                <Check size={12} class="text-cyan-400 shrink-0" />
              {:else}
                <span class="w-3 shrink-0"></span>
              {/if}
              <span class="font-mono truncate">{branch.name}</span>
            </button>
          {/each}
        {/if}

        {#if remoteBranches.length > 0}
          <p
            class="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-app-text-3
                   {localBranches.length > 0 ? 'mt-1' : ''}"
          >
            Remote
          </p>
          {#each remoteBranches as branch}
            <button
              class="flex items-center gap-2 px-2 py-1.5 text-xs text-left
                     text-app-text-3 hover:bg-app-card hover:text-app-text-2 transition-colors"
              onclick={() => select(branch.name)}
            >
              <span class="w-3 shrink-0"></span>
              <span class="font-mono truncate">{branch.name}</span>
            </button>
          {/each}
        {/if}

        {#if localBranches.length === 0 && remoteBranches.length === 0}
          <p class="px-2 py-3 text-xs text-center text-app-text-3">No branches found</p>
        {/if}
      </div>
    </div>
  {/if}
</div>
