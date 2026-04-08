<script lang="ts">
  import FunctionList from "$lib/components/functions/FunctionList.svelte";
  import FunctionEditor from "$lib/components/functions/FunctionEditor.svelte";
  import ResizablePanel from "$lib/components/shared/ResizablePanel.svelte";
</script>

<div class="flex h-full">
  <ResizablePanel
    defaultWidth={224}
    minWidth={140}
    maxWidth={420}
    side="right"
    storageKey="flupi:sidebar-functions"
  >
    <FunctionList />
  </ResizablePanel>
  <div class="flex-1 min-w-0">
    <FunctionEditor />
  </div>
</div>
