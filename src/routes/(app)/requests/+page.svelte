<script lang="ts">
  import RequestTree from '$lib/components/requests/RequestTree.svelte';
  import RequestEditor from '$lib/components/requests/RequestEditor.svelte';
  import CollectionEditor from '$lib/components/requests/CollectionEditor.svelte';
  import { activeRequest, activeCollectionFolder, activeCollection, requestTree } from '$lib/stores/requests';
</script>

<div class="flex h-full">
  <div class="w-64 shrink-0 border-r border-app-border overflow-y-auto">
    <RequestTree />
  </div>
  <div class="flex-1 overflow-hidden">
    {#if $activeCollectionFolder && $activeCollection}
      <div class="h-full overflow-y-auto">
        <CollectionEditor
          folderName={$activeCollectionFolder}
          collection={$activeCollection}
          onUpdate={(updated) => {
            activeCollection.set(updated);
            requestTree.update(tree =>
              tree.map(n =>
                n.type === 'Collection' && n.folder_name === $activeCollectionFolder
                  ? { ...n, name: updated.name }
                  : n
              )
            );
          }}
        />
      </div>
    {:else if $activeRequest}
      <RequestEditor />
    {:else}
      <p class="p-6 text-app-text-3 text-sm">Pick a request or collection from the sidebar.</p>
    {/if}
  </div>
</div>
