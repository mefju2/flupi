<script lang="ts">
  import { ChevronDown, ChevronRight, Plus, Minus } from "lucide-svelte";
  import type { GitStatus } from "$lib/services/tauri-commands";
  import type { GitFileStatus } from "$lib/stores/git";
  import GitFileTree, { type GitFileEntry } from "./GitFileTree.svelte";

  interface Props {
    status: GitStatus;
    selectedFile: { path: string; status: GitFileStatus } | null;
    onselect: (path: string, status: GitFileStatus) => void;
    onstage: (path: string) => void;
    onunstage: (path: string) => void;
    onstageall: () => void;
    onunstageall: () => void;
  }

  let {
    status,
    selectedFile,
    onselect,
    onstage,
    onunstage,
    onstageall,
    onunstageall,
  }: Props = $props();

  let stagedOpen = $state(true);
  let unstagedOpen = $state(true);

  const stagedFiles = $derived<GitFileEntry[]>(
    status.staged.map((path) => ({ path, status: "staged" as const })),
  );

  const unstagedFiles = $derived<GitFileEntry[]>([
    ...status.modified.map((path) => ({ path, status: "modified" as const })),
    ...status.deleted.map((path) => ({ path, status: "deleted" as const })),
    ...status.untracked.map((path) => ({ path, status: "untracked" as const })),
  ]);

  const isEmpty = $derived(
    stagedFiles.length === 0 && unstagedFiles.length === 0,
  );

  function handleAction(path: string, fileStatus: GitFileStatus) {
    if (fileStatus === "staged") onunstage(path);
    else onstage(path);
  }
</script>

{#if isEmpty}
  <p class="text-xs text-app-text-3 px-1">Working tree is clean.</p>
{:else}
  <div class="flex flex-col gap-3">
    <!-- Staged section -->
    <section class="flex flex-col gap-1">
      <div class="flex items-center gap-1">
        <button
          class="flex items-center gap-1 text-xs font-semibold text-app-text-3 uppercase
                 tracking-wider px-1 hover:text-app-text-2 transition-colors flex-1 text-left"
          onclick={() => (stagedOpen = !stagedOpen)}
        >
          {#if stagedOpen}<ChevronDown size={12} />{:else}<ChevronRight
              size={12}
            />{/if}
          Staged ({stagedFiles.length})
        </button>
        {#if stagedFiles.length > 0}
          <button
            class="flex items-center gap-0.5 px-1.5 py-0.5 text-xs rounded
                   text-app-text-3 hover:bg-app-card hover:text-app-text-2 transition-colors"
            onclick={onunstageall}
            title="Unstage all"
            aria-label="Unstage all files"
          >
            <Minus size={10} /> All
          </button>
        {/if}
      </div>
      {#if stagedOpen}
        {#if stagedFiles.length > 0}
          <GitFileTree
            files={stagedFiles}
            {selectedFile}
            {onselect}
            onaction={handleAction}
          />
        {:else}
          <p class="text-xs text-app-text-3 px-3 italic">No staged changes</p>
        {/if}
      {/if}
    </section>

    <!-- Unstaged section -->
    <section class="flex flex-col gap-1">
      <div class="flex items-center gap-1">
        <button
          class="flex items-center gap-1 text-xs font-semibold text-app-text-3 uppercase
                 tracking-wider px-1 hover:text-app-text-2 transition-colors flex-1 text-left"
          onclick={() => (unstagedOpen = !unstagedOpen)}
        >
          {#if unstagedOpen}<ChevronDown size={12} />{:else}<ChevronRight
              size={12}
            />{/if}
          Unstaged ({unstagedFiles.length})
        </button>
        {#if unstagedFiles.length > 0}
          <button
            class="flex items-center gap-0.5 px-1.5 py-0.5 text-xs rounded
                   text-app-text-3 hover:bg-app-card hover:text-app-text-2 transition-colors"
            onclick={onstageall}
            title="Stage all"
            aria-label="Stage all files"
          >
            <Plus size={10} /> All
          </button>
        {/if}
      </div>
      {#if unstagedOpen}
        {#if unstagedFiles.length > 0}
          <GitFileTree
            files={unstagedFiles}
            {selectedFile}
            {onselect}
            onaction={handleAction}
          />
        {:else}
          <p class="text-xs text-app-text-3 px-3 italic">No unstaged changes</p>
        {/if}
      {/if}
    </section>
  </div>
{/if}
