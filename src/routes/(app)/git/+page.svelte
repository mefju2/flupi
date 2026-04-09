<script lang="ts">
  import ResizablePanel from '$lib/components/shared/ResizablePanel.svelte';
  import GitStatusPanel from '$lib/components/git/GitStatusPanel.svelte';
  import GitDiffPanel from '$lib/components/git/GitDiffPanel.svelte';
</script>

<div class="flex h-full">
  <ResizablePanel defaultWidth={280} minWidth={180} maxWidth={480} side="right" storageKey="flupi:sidebar-git">
    <GitStatusPanel />
  </ResizablePanel>
  <div class="flex-1 h-full overflow-hidden">
    <GitDiffPanel />
  </div>
</div>
