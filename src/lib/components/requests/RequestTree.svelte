<script lang="ts">
  import { onMount } from 'svelte';
  import { project } from '$lib/stores/project';
  import { requestTree, activeRequestId, activeRequest } from '$lib/stores/requests';
  import {
    loadRequestTree,
    getRequest,
    createRequest,
    deleteRequest,
    renameRequest,
    duplicateRequest,
    createCollection,
    deleteCollection,
    renameCollection,
    type RequestTreeNode,
  } from '$lib/services/tauri-commands';
  import TreeNode from '$lib/components/shared/TreeNode.svelte';
  import ContextMenu from '$lib/components/shared/ContextMenu.svelte';

  let contextMenu: { x: number; y: number; items: { label: string; action: () => void; danger?: boolean }[] } | null = $state(null);

  async function reload() {
    if (!$project.path) return;
    try {
      requestTree.set(await loadRequestTree($project.path));
    } catch (e) {
      console.error('Failed to load request tree:', e);
    }
  }

  onMount(reload);

  async function selectRequest(id: string) {
    if (!$project.path) return;
    activeRequestId.set(id);
    try {
      activeRequest.set(await getRequest($project.path, id));
    } catch (e) {
      console.error('Failed to load request:', e);
    }
  }

  function buildContextMenuItems(node: RequestTreeNode) {
    if (node.type === 'Collection') {
      return [
        { label: 'New Request', action: () => handleNewRequest(node.folder_name) },
        { label: 'Rename', action: () => handleRenameCollection(node.folder_name) },
        { label: 'Delete Collection', action: () => handleDeleteCollection(node.folder_name), danger: true },
      ];
    }
    if (node.type === 'Folder') {
      return [
        { label: 'New Request', action: () => handleNewRequest(null) },
      ];
    }
    // Request node
    return [
      { label: 'Rename', action: () => handleRenameRequest(node.id) },
      { label: 'Duplicate', action: () => handleDuplicateRequest(node.id) },
      { label: 'Delete', action: () => handleDeleteRequest(node.id), danger: true },
    ];
  }

  function openContextMenu(e: MouseEvent, node: RequestTreeNode) {
    contextMenu = { x: e.clientX, y: e.clientY, items: buildContextMenuItems(node) };
  }

  async function handleNewRequest(collectionFolder: string | null) {
    if (!$project.path) return;
    const name = prompt('Request name:');
    if (!name?.trim()) return;
    try {
      await createRequest($project.path, collectionFolder, name.trim());
      await reload();
    } catch (e) { console.error(e); }
  }

  async function handleRenameRequest(id: string) {
    if (!$project.path) return;
    const name = prompt('New name:');
    if (!name?.trim()) return;
    try {
      await renameRequest($project.path, id, name.trim());
      await reload();
    } catch (e) { console.error(e); }
  }

  async function handleDeleteRequest(id: string) {
    if (!$project.path || !confirm('Delete this request?')) return;
    try {
      await deleteRequest($project.path, id);
      if ($activeRequestId === id) { activeRequestId.set(null); activeRequest.set(null); }
      await reload();
    } catch (e) { console.error(e); }
  }

  async function handleDuplicateRequest(id: string) {
    if (!$project.path) return;
    try {
      await duplicateRequest($project.path, id);
      await reload();
    } catch (e) { console.error(e); }
  }

  async function handleNewCollection() {
    if (!$project.path) return;
    const name = prompt('Collection name:');
    if (!name?.trim()) return;
    try {
      await createCollection($project.path, name.trim());
      await reload();
    } catch (e) { console.error(e); }
  }

  async function handleRenameCollection(folderName: string) {
    if (!$project.path) return;
    const name = prompt('New name:');
    if (!name?.trim()) return;
    try {
      await renameCollection($project.path, folderName, name.trim());
      await reload();
    } catch (e) { console.error(e); }
  }

  async function handleDeleteCollection(folderName: string) {
    if (!$project.path || !confirm('Delete this collection and all its requests?')) return;
    try {
      await deleteCollection($project.path, folderName);
      await reload();
    } catch (e) { console.error(e); }
  }
</script>

<div class="flex flex-col h-full bg-zinc-900">
  <div class="px-3 py-2 text-xs text-zinc-500 uppercase tracking-wider border-b border-zinc-800">
    Requests
  </div>

  <div class="flex-1 overflow-y-auto px-1 py-1">
    {#each $requestTree as node}
      <TreeNode
        {node}
        activeRequestId={$activeRequestId}
        onSelect={selectRequest}
        onContextMenu={openContextMenu}
      />
    {/each}

    {#if $requestTree.length === 0}
      <p class="px-3 py-4 text-xs text-zinc-600">No collections yet.</p>
    {/if}
  </div>

  <div class="border-t border-zinc-800 px-3 py-2">
    <button
      class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
      onclick={handleNewCollection}
    >+ New Collection</button>
  </div>
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={contextMenu.items}
    onClose={() => (contextMenu = null)}
  />
{/if}
