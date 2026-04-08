<script lang="ts">
  import EnvironmentList from "$lib/components/environments/EnvironmentList.svelte";
  import EnvironmentEditor from "$lib/components/environments/EnvironmentEditor.svelte";
  import ResizablePanel from "$lib/components/shared/ResizablePanel.svelte";
</script>

<div class="flex h-full">
  <ResizablePanel
    defaultWidth={224}
    minWidth={140}
    maxWidth={420}
    side="right"
    storageKey="flupi:sidebar-environments"
  >
    <EnvironmentList />
  </ResizablePanel>
  <div class="flex-1 overflow-auto">
    <EnvironmentEditor />
  </div>
</div>
