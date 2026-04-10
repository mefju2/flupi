<script lang="ts">
  import { GitBranch } from "lucide-svelte";
  import { gitPageState } from "$lib/stores/git";
  import { project } from "$lib/stores/project";
  import { gitFileDiff, type GitFileDiff } from "$lib/services/tauri-commands";
  import GitFileDiffView from "./GitFileDiff.svelte";

  let diffContent = $state<GitFileDiff | null>(null);
  let isLoadingDiff = $state(false);
  let diffError = $state<string | null>(null);

  let loadSeq = 0;

  $effect(() => {
    const sel = $gitPageState.selectedFile;
    const path = $project.path;
    if (!sel || !path) {
      diffContent = null;
      return;
    }
    const seq = ++loadSeq;
    loadDiff(path, sel.path, seq);
  });

  async function loadDiff(projectPath: string, filePath: string, seq: number) {
    isLoadingDiff = true;
    diffError = null;
    try {
      const result = await gitFileDiff(projectPath, filePath);
      if (seq !== loadSeq) return;
      diffContent = result;
    } catch (e) {
      if (seq !== loadSeq) return;
      diffError = String(e);
      diffContent = null;
    } finally {
      if (seq === loadSeq) isLoadingDiff = false;
    }
  }
</script>

{#if !$gitPageState.selectedFile}
  <div
    class="flex flex-col items-center justify-center h-full gap-3 text-app-text-3"
  >
    <GitBranch size={28} class="opacity-30" />
    <p class="text-sm">Select a file to view its diff</p>
  </div>
{:else if diffError}
  <div class="p-4">
    <div
      class="px-3 py-2 rounded bg-red-500/10 border border-red-500/30 text-red-400 text-xs font-mono"
    >
      {diffError}
    </div>
  </div>
{:else}
  <GitFileDiffView
    filePath={$gitPageState.selectedFile.path}
    lines={diffContent?.lines ?? []}
    isNewFile={diffContent?.isNewFile ?? false}
    isLoading={isLoadingDiff}
  />
{/if}
