<script lang="ts">
  import RequestTree from '$lib/components/requests/RequestTree.svelte';
  import RequestEditor from '$lib/components/requests/RequestEditor.svelte';
  import { activeRequest } from '$lib/stores/requests';
</script>

<div class="flex h-full">
  <div class="w-64 shrink-0 border-r border-zinc-800 overflow-y-auto">
    <RequestTree />
  </div>
  <div class="flex-1 overflow-hidden">
    {#if $activeRequest}
      <RequestEditor />
    {:else}
      <p class="p-6 text-zinc-500 text-sm">Pick a request from the sidebar to get started</p>
    {/if}
  </div>
</div>
