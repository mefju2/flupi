<script lang="ts">
  import { onMount } from 'svelte';
  import { project } from '$lib/stores/project';
  import { requestTree, activeRequestId, activeRequest } from '$lib/stores/requests';
  import {
    loadRequestTree, getRequest, createRequest, deleteRequest, renameRequest,
    duplicateRequest, createCollection, deleteCollection, renameCollection,
    moveRequest, type RequestTreeNode,
  } from '$lib/services/tauri-commands';
  import {
    ROOT_KEY, rebuildDndItems, findContainerKey, containerKeyToArg,
    argToContainerKey, buildContextMenuItems,
    type DndItem, type DndItems,
  } from '$lib/services/request-tree-dnd';
  import { dndzone } from 'svelte-dnd-action';
  import TreeNode from '$lib/components/shared/TreeNode.svelte';
  import ContextMenu from '$lib/components/shared/ContextMenu.svelte';

  let dndItems = $state<DndItems>({});
  let toast = $state<string | null>(null);
  let contextMenu: { x: number; y: number; items: { label: string; action: () => void; danger?: boolean }[] } | null = $state(null);
  let pendingInput = $state<{
    type: 'rename-request' | 'rename-collection' | 'new-request' | 'new-collection';
    id: string; value: string;
  } | null>(null);

  function showToast(msg: string) { toast = msg; setTimeout(() => (toast = null), 3000); }
  function focusOnMount(el: HTMLElement) { el.focus(); if (el instanceof HTMLInputElement) el.select(); }

  async function reload() {
    if (!$project.path) return;
    try { requestTree.set(await loadRequestTree($project.path)); dndItems = rebuildDndItems($requestTree); }
    catch (e) { console.error('Failed to load request tree:', e); }
  }

  onMount(reload);

  function handleConsider(e: CustomEvent<{ items: DndItem[] }>, key: string) {
    dndItems = { ...dndItems, [key]: e.detail.items };
  }

  async function handleFinalize(e: CustomEvent<{ items: DndItem[]; info: { trigger: string; id: string } }>, key: string) {
    const id = e.detail.info.id;
    dndItems = { ...dndItems, [key]: e.detail.items };
    if (!$project.path) return;
    const originalKey = argToContainerKey(findContainerKey($requestTree, id) ?? null);
    if (originalKey !== key) {
      const target = containerKeyToArg(key);
      try {
        await moveRequest($project.path, id, target);
        await reload();
        showToast(`Request will now inherit auth and headers from ${target ?? 'root'}.`);
      } catch (err) { console.error('Failed to move request:', err); await reload(); }
    }
  }

  async function selectRequest(id: string) {
    if (!$project.path) return;
    activeRequestId.set(id);
    try { activeRequest.set(await getRequest($project.path, id)); }
    catch (e) { console.error('Failed to load request:', e); }
  }

  function openContextMenu(e: MouseEvent, node: RequestTreeNode) {
    const items = buildContextMenuItems(node,
      (col) => { pendingInput = { type: 'new-request', id: col ?? '', value: 'New Request' }; },
      (id) => { pendingInput = { type: 'rename-request', id, value: '' }; },
      async (id) => {
        if (!$project.path || !confirm('Delete this request?')) return;
        try { await deleteRequest($project.path, id); if ($activeRequestId === id) { activeRequestId.set(null); activeRequest.set(null); } await reload(); }
        catch (e) { console.error(e); }
      },
      async (id) => { if (!$project.path) return; try { await duplicateRequest($project.path, id); await reload(); } catch (e) { console.error(e); } },
      (folder) => { pendingInput = { type: 'rename-collection', id: folder, value: '' }; },
      async (folder) => {
        if (!$project.path || !confirm('Delete this collection and all its requests?')) return;
        try { await deleteCollection($project.path, folder); await reload(); } catch (e) { console.error(e); }
      }
    );
    contextMenu = { x: e.clientX, y: e.clientY, items };
  }

  async function confirmPendingInput() {
    if (!pendingInput || !$project.path) return;
    const { type, id, value } = pendingInput;
    const trimmed = value.trim();
    if (!trimmed) { pendingInput = null; return; }
    pendingInput = null;
    try {
      if (type === 'new-request') await createRequest($project.path, id || null, trimmed);
      else if (type === 'new-collection') await createCollection($project.path, trimmed);
      else if (type === 'rename-request') await renameRequest($project.path, id, trimmed);
      else if (type === 'rename-collection') await renameCollection($project.path, id, trimmed);
      await reload();
    } catch (e) { console.error(e); }
  }

  function handlePendingInputKey(e: KeyboardEvent) {
    if (e.key === 'Enter') confirmPendingInput();
    if (e.key === 'Escape') pendingInput = null;
  }

  let collections = $derived(
    $requestTree.filter((n): n is RequestTreeNode & { type: 'Collection' } => n.type === 'Collection')
  );
</script>

<div class="flex flex-col h-full bg-zinc-900">
  <div class="px-3 py-2 text-xs text-zinc-500 uppercase tracking-wider border-b border-zinc-800">
    Requests
  </div>

  <div class="flex-1 overflow-y-auto px-1 py-1">
    {#each collections as collection (collection.folder_name)}
      {@const key = collection.folder_name}
      {@const items = dndItems[key] ?? []}
      <TreeNode node={{ ...collection, children: [] }} activeRequestId={$activeRequestId} onSelect={selectRequest} onContextMenu={openContextMenu} />
      <div class="ml-3 border-l border-zinc-800 pl-1 min-h-1"
        use:dndzone={{ items, type: 'request', flipDurationMs: 150 }}
        onconsider={(e: CustomEvent<{ items: DndItem[] }>) => handleConsider(e, key)}
        onfinalize={(e: CustomEvent<{ items: DndItem[]; info: { trigger: string; id: string } }>) => handleFinalize(e, key)}
      >
        {#each items as item (item.id)}
          <TreeNode node={item.node} activeRequestId={$activeRequestId} onSelect={selectRequest} onContextMenu={openContextMenu} showDragHandle={true} />
        {/each}
        {#if items.length === 0}<p class="px-2 py-1 text-xs text-zinc-600 italic">Empty</p>{/if}
      </div>
    {/each}

    {#if (dndItems[ROOT_KEY] ?? []).length > 0}
      {@const rootItems = dndItems[ROOT_KEY] ?? []}
      <div class="min-h-1"
        use:dndzone={{ items: rootItems, type: 'request', flipDurationMs: 150 }}
        onconsider={(e: CustomEvent<{ items: DndItem[] }>) => handleConsider(e, ROOT_KEY)}
        onfinalize={(e: CustomEvent<{ items: DndItem[]; info: { trigger: string; id: string } }>) => handleFinalize(e, ROOT_KEY)}
      >
        {#each rootItems as item (item.id)}
          <TreeNode node={item.node} activeRequestId={$activeRequestId} onSelect={selectRequest} onContextMenu={openContextMenu} showDragHandle={true} />
        {/each}
      </div>
    {/if}

    {#if $requestTree.length === 0}
      <p class="px-3 py-4 text-xs text-zinc-600">No requests yet. Create a request or import from OpenAPI.</p>
    {/if}
  </div>

  {#if toast}
    <div class="px-3 py-2 text-xs text-cyan-400 bg-zinc-800 border-t border-zinc-700 animate-pulse">{toast}</div>
  {/if}

  {#if pendingInput}
    <div class="px-3 py-2 border-t border-zinc-800">
      <p class="text-xs text-zinc-500 mb-1">{pendingInput.type.startsWith('new') ? 'Create:' : 'Rename to:'}</p>
      <input class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono focus:outline-none focus:border-cyan-600"
        bind:value={pendingInput.value} onkeydown={handlePendingInputKey} use:focusOnMount />
      <div class="flex gap-2 mt-1">
        <button class="text-xs text-cyan-500" onclick={confirmPendingInput}>OK</button>
        <button class="text-xs text-zinc-500" onclick={() => pendingInput = null}>Cancel</button>
      </div>
    </div>
  {/if}

  <div class="border-t border-zinc-800 px-3 py-2">
    <button class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
      onclick={() => { pendingInput = { type: 'new-collection', id: '', value: 'New Collection' }; }}>+ New Collection</button>
  </div>
</div>

{#if contextMenu}
  <ContextMenu x={contextMenu.x} y={contextMenu.y} items={contextMenu.items} onClose={() => (contextMenu = null)} />
{/if}
