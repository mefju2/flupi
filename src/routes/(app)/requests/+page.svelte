<script lang="ts">
  import RequestTree from '$lib/components/requests/RequestTree.svelte';
  import { activeRequestId } from '$lib/stores/requests';
</script>

<div class="flex h-full">
  <div class="w-64 shrink-0 border-r border-zinc-800 overflow-y-auto">
    <RequestTree />
  </div>
  <div class="flex-1 overflow-hidden">
    {#if $activeRequestId}
      <p class="p-6 text-zinc-500 text-sm">Request editor coming soon...</p>
    {:else}
      <p class="p-6 text-zinc-500 text-sm">Select a request to edit it.</p>
    {/if}
  </div>
</div>
