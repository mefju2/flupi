<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { project } from "$lib/stores/project";
  import {
    requestTree,
    activeRequestId,
    activeRequest,
    activeCollectionFolder,
    activeCollection,
  } from "$lib/stores/requests";
  import {
    loadRequestTree,
    getRequest,
    getCollection,
    createRequest,
    deleteRequest,
    renameRequest,
    duplicateRequest,
    createCollection,
    deleteCollection,
    renameCollection,
    moveRequest,
    type RequestTreeNode,
  } from "$lib/services/tauri-commands";
  import {
    ROOT_KEY,
    rebuildDndItems,
    findContainerKey,
    containerKeyToArg,
    argToContainerKey,
    buildContextMenuItems,
    type DndItem,
    type DndItems,
  } from "$lib/services/request-tree-dnd";
  import { lastResponse, lastError } from "$lib/stores/execution";
  import { activeScenario } from "$lib/stores/scenarios";
  import { dndzone } from "svelte-dnd-action";
  import TreeNode from "$lib/components/shared/TreeNode.svelte";
  import ContextMenu from "$lib/components/shared/ContextMenu.svelte";

  let dndItems = $state<DndItems>({});
  $effect(() => {
    dndItems = rebuildDndItems($requestTree);
  });
  let collectionExpanded = $state<Record<string, boolean>>({});
  let toast = $state<string | null>(null);
  let contextMenu: {
    x: number;
    y: number;
    items: { label: string; action: () => void; danger?: boolean }[];
  } | null = $state(null);
  let pendingInput = $state<{
    type:
      | "rename-request"
      | "rename-collection"
      | "new-request"
      | "new-collection";
    id: string;
    value: string;
  } | null>(null);
  let pendingDelete = $state<{
    type: "request" | "collection";
    id: string;
    label: string;
  } | null>(null);

  onMount(() => {
    const handler = () => {
      pendingInput = { type: "new-request", id: "", value: "New Request" };
    };
    window.addEventListener("flupi:new-request", handler);

    const renameHandler = () => {
      const reqId = $activeRequestId;
      const colFolder = $activeCollectionFolder;
      if (reqId) {
        // Find name from tree
        function findRequestName(
          nodes: RequestTreeNode[],
          id: string,
        ): string | null {
          for (const n of nodes) {
            if (n.type === "Request" && n.id === id) return n.name;
            if (n.type === "Collection" || n.type === "Folder") {
              const found = findRequestName(n.children, id);
              if (found) return found;
            }
          }
          return null;
        }
        const name = findRequestName($requestTree, reqId) ?? reqId;
        pendingInput = { type: "rename-request", id: reqId, value: name };
      } else if (colFolder) {
        const col = $requestTree.find(
          (n): n is RequestTreeNode & { type: "Collection" } =>
            n.type === "Collection" && n.folder_name === colFolder,
        );
        if (col)
          pendingInput = {
            type: "rename-collection",
            id: colFolder,
            value: col.name,
          };
      }
    };
    window.addEventListener("flupi:rename-active", renameHandler);

    return () => {
      window.removeEventListener("flupi:new-request", handler);
      window.removeEventListener("flupi:rename-active", renameHandler);
    };
  });

  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  function showToast(msg: string) {
    if (toastTimer) clearTimeout(toastTimer);
    toast = msg;
    toastTimer = setTimeout(() => {
      toast = null;
      toastTimer = null;
    }, 3000);
  }
  function focusOnMount(el: HTMLElement) {
    el.focus();
    if (el instanceof HTMLInputElement) el.select();
  }

  async function reload() {
    if (!$project.path) return;
    try {
      requestTree.set(await loadRequestTree($project.path));
    } catch (e) {
      console.error("Failed to load request tree:", e);
    }
  }

  // Trees are loaded by +layout.svelte on project open; reload() is called
  // explicitly after create/rename/delete/move mutations.

  function handleConsider(e: CustomEvent<{ items: DndItem[] }>, key: string) {
    dndItems = { ...dndItems, [key]: e.detail.items };
  }

  async function handleFinalize(
    e: CustomEvent<{ items: DndItem[]; info: { trigger: string; id: string } }>,
    key: string,
  ) {
    const id = e.detail.info.id;
    dndItems = { ...dndItems, [key]: e.detail.items };
    if (!$project.path) return;
    const originalKey = argToContainerKey(
      findContainerKey($requestTree, id) ?? null,
    );
    if (originalKey !== key) {
      const target = containerKeyToArg(key);
      try {
        const newId = await moveRequest($project.path, id, target);
        await reload();
        // If the moved request was active, refresh it under its new ID so
        // req.collection and req.id reflect the new location
        if ($activeRequestId === id) {
          await selectRequest(newId);
        }
        showToast(
          `Request will now inherit auth and headers from ${target ?? "root"}.`,
        );
      } catch (err) {
        console.error("Failed to move request:", err);
        await reload();
      }
    }
  }

  async function selectRequest(id: string) {
    if (!$project.path) return;
    try {
      const req = await getRequest($project.path, id);
      lastResponse.set(null);
      lastError.set(null);
      activeCollectionFolder.set(null);
      activeRequestId.set(id);
      activeRequest.set(req);
      if (req.collection) {
        const col = await getCollection($project.path, req.collection);
        activeCollection.set(col);
      } else {
        activeCollection.set(null);
      }
    } catch (e) {
      console.error("Failed to load request:", e);
    }
  }

  async function selectCollection(folderName: string) {
    if (!$project.path) return;
    try {
      const data = await getCollection($project.path, folderName);
      activeRequestId.set(null);
      activeRequest.set(null);
      activeCollectionFolder.set(folderName);
      activeCollection.set(data);
    } catch (e) {
      console.error("Failed to load collection:", e);
    }
  }

  function openContextMenu(e: MouseEvent, node: RequestTreeNode) {
    const items = buildContextMenuItems(
      node,
      (col) => {
        pendingInput = {
          type: "new-request",
          id: col ?? "",
          value: "New Request",
        };
      },
      (id) => {
        pendingInput = { type: "rename-request", id, value: node.name };
      },
      (id) => {
        pendingDelete = { type: "request", id, label: node.name };
      },
      async (id) => {
        if (!$project.path) return;
        try {
          await duplicateRequest($project.path, id);
          await reload();
        } catch (e) {
          console.error(e);
        }
      },
      (folder) => {
        pendingInput = {
          type: "rename-collection",
          id: folder,
          value: node.name,
        };
      },
      (folder) => {
        pendingDelete = { type: "collection", id: folder, label: node.name };
      },
    );
    contextMenu = { x: e.clientX, y: e.clientY, items };
  }

  async function confirmDelete() {
    if (!pendingDelete || !$project.path) return;
    const { type, id } = pendingDelete;
    pendingDelete = null;
    try {
      if (type === "request") {
        await deleteRequest($project.path, id);
        if ($activeRequestId === id) {
          activeRequestId.set(null);
          activeRequest.set(null);
        }
      } else {
        await deleteCollection($project.path, id);
      }
      await reload();
    } catch (e) {
      console.error(e);
    }
  }

  async function confirmPendingInput() {
    if (!pendingInput || !$project.path) return;
    const { type, id, value } = pendingInput;
    const trimmed = value.trim();
    if (!trimmed) {
      pendingInput = null;
      return;
    }
    pendingInput = null;
    try {
      if (type === "new-request")
        await createRequest($project.path, id || null, trimmed);
      else if (type === "new-collection")
        await createCollection($project.path, trimmed);
      else if (type === "rename-request") {
        const newId = await renameRequest($project.path, id, trimmed);
        // Sync any open scenario that references the old request ID in memory
        const cur = $activeScenario;
        if (cur && cur.steps.some((s) => s.requestId === id)) {
          activeScenario.set({
            ...cur,
            steps: cur.steps.map((s) =>
              s.requestId === id ? { ...s, requestId: newId } : s,
            ),
          });
        }
      } else if (type === "rename-collection") {
        const newSlug = await renameCollection($project.path, id, trimmed);
        await reload();
        if ($activeCollectionFolder === id) {
          const updated = await getCollection($project.path, newSlug);
          activeCollectionFolder.set(newSlug);
          activeCollection.set(updated);
        }
        return;
      }
      await reload();
    } catch (e) {
      console.error(e);
    }
  }

  function handlePendingInputKey(e: KeyboardEvent) {
    if (e.key === "Enter") confirmPendingInput();
    if (e.key === "Escape") pendingInput = null;
  }

  function makeInlineEdit(item: DndItem) {
    if (pendingInput?.type !== "rename-request" || pendingInput.id !== item.id)
      return null;
    return {
      value: pendingInput.value,
      onChange: (v: string) => {
        if (pendingInput) pendingInput = { ...pendingInput, value: v };
      },
      onConfirm: confirmPendingInput,
      onCancel: () => {
        pendingInput = null;
      },
    };
  }

  function makeInlineEditCollection(folderName: string) {
    if (
      pendingInput?.type !== "rename-collection" ||
      pendingInput.id !== folderName
    )
      return null;
    return {
      value: pendingInput.value,
      onChange: (v: string) => {
        if (pendingInput) pendingInput = { ...pendingInput, value: v };
      },
      onConfirm: confirmPendingInput,
      onCancel: () => {
        pendingInput = null;
      },
    };
  }

  let collections = $derived(
    $requestTree.filter(
      (n): n is RequestTreeNode & { type: "Collection" } =>
        n.type === "Collection",
    ),
  );
</script>

<div class="flex flex-col h-full bg-app-panel">
  <div
    class="px-3 py-2 text-xs text-app-text-3 uppercase tracking-wider border-b border-app-border"
  >
    Requests
  </div>

  <div class="flex-1 overflow-y-auto px-1 py-1">
    {#each collections as collection (collection.folder_name)}
      {@const key = collection.folder_name}
      {@const items = dndItems[key] ?? []}
      <TreeNode
        node={{ ...collection, children: [] }}
        activeRequestId={$activeRequestId}
        activeCollectionFolder={$activeCollectionFolder}
        onSelect={selectRequest}
        onSelectCollection={selectCollection}
        onContextMenu={openContextMenu}
        expanded={collectionExpanded[key] ?? true}
        onToggleExpanded={() => {
          collectionExpanded[key] = !(collectionExpanded[key] ?? true);
        }}
        inlineEdit={makeInlineEditCollection(collection.folder_name)}
      />
      {#if collectionExpanded[key] ?? true}
        <div
          class="ml-3 border-l border-app-border pl-1 min-h-1"
          use:dndzone={{ items, type: "request", flipDurationMs: 150 }}
          onconsider={(e: CustomEvent<{ items: DndItem[] }>) =>
            handleConsider(e, key)}
          onfinalize={(
            e: CustomEvent<{
              items: DndItem[];
              info: { trigger: string; id: string };
            }>,
          ) => handleFinalize(e, key)}
        >
          {#each items as item (item.id)}
            <TreeNode
              node={item.node}
              activeRequestId={$activeRequestId}
              onSelect={selectRequest}
              onContextMenu={openContextMenu}
              showDragHandle={true}
              inlineEdit={makeInlineEdit(item)}
            />
          {/each}
          {#if items.length === 0}<p
              class="px-2 py-1 text-xs text-app-text-4 italic"
            >
              Empty
            </p>{/if}
        </div>
      {/if}
    {/each}

    {#if (dndItems[ROOT_KEY] ?? []).length > 0}
      {@const rootItems = dndItems[ROOT_KEY] ?? []}
      <div
        class="min-h-1"
        use:dndzone={{ items: rootItems, type: "request", flipDurationMs: 150 }}
        onconsider={(e: CustomEvent<{ items: DndItem[] }>) =>
          handleConsider(e, ROOT_KEY)}
        onfinalize={(
          e: CustomEvent<{
            items: DndItem[];
            info: { trigger: string; id: string };
          }>,
        ) => handleFinalize(e, ROOT_KEY)}
      >
        {#each rootItems as item (item.id)}
          <TreeNode
            node={item.node}
            activeRequestId={$activeRequestId}
            onSelect={selectRequest}
            onContextMenu={openContextMenu}
            showDragHandle={true}
            inlineEdit={makeInlineEdit(item)}
          />
        {/each}
      </div>
    {/if}

    {#if $requestTree.length === 0}
      <p class="px-3 py-4 text-xs text-app-text-4">
        No requests yet. Create a request or import from OpenAPI.
      </p>
    {/if}
  </div>

  {#if toast}
    <div
      class="px-3 py-2 text-xs text-cyan-400 bg-app-card border-t border-app-border-2"
      transition:fade={{ duration: 150 }}
    >
      {toast}
    </div>
  {/if}

  {#if pendingDelete}
    <div class="px-3 py-2 border-t border-app-border bg-app-card">
      <p class="text-xs text-app-text-2 mb-2">
        Delete <span class="text-app-text font-mono">{pendingDelete.label}</span
        >?
      </p>
      <div class="flex gap-2">
        <button
          class="text-xs text-red-400 hover:text-red-300 transition-colors"
          onclick={confirmDelete}>Delete</button
        >
        <button
          class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
          onclick={() => (pendingDelete = null)}>Cancel</button
        >
      </div>
    </div>
  {/if}

  {#if pendingInput && pendingInput.type.startsWith("new")}
    <div class="px-3 py-2 border-t border-app-border">
      <p class="text-xs text-app-text-3 mb-1">Create:</p>
      <input
        class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono focus:outline-none focus:border-cyan-600"
        bind:value={pendingInput.value}
        onkeydown={handlePendingInputKey}
        use:focusOnMount
      />
      <div class="flex gap-2 mt-1">
        <button class="text-xs text-cyan-500" onclick={confirmPendingInput}
          >OK</button
        >
        <button
          class="text-xs text-app-text-3"
          onclick={() => (pendingInput = null)}>Cancel</button
        >
      </div>
    </div>
  {/if}

  <div class="border-t border-app-border px-3 py-2">
    <button
      class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
      onclick={() => {
        pendingInput = {
          type: "new-collection",
          id: "",
          value: "New Collection",
        };
      }}>+ New Collection</button
    >
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
