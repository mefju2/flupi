import type { RequestTreeNode } from '$lib/services/tauri-commands';

export type DndItem = {
  id: string;
  name: string;
  method: string;
  node: RequestTreeNode & { type: 'Request' };
};

export type DndItems = Record<string, DndItem[]>;

export const ROOT_KEY = '__root__';

export function containerKeyToArg(key: string): string | null {
  return key === ROOT_KEY ? null : key;
}

export function argToContainerKey(arg: string | null): string {
  return arg === null ? ROOT_KEY : arg;
}

/** Extract direct Request children from a Collection (by folder_name) or root (null). */
export function buildDndItems(
  tree: RequestTreeNode[],
  containerKey: string | null
): DndItem[] {
  if (containerKey === null) {
    // Root-level requests (no collection)
    return tree
      .filter((n): n is RequestTreeNode & { type: 'Request' } => n.type === 'Request')
      .map((n) => ({ id: n.id, name: n.name, method: n.method, node: n }));
  }
  const collection = tree.find(
    (n): n is RequestTreeNode & { type: 'Collection' } =>
      n.type === 'Collection' && n.folder_name === containerKey
  );
  if (!collection) return [];
  return collection.children
    .filter((n): n is RequestTreeNode & { type: 'Request' } => n.type === 'Request')
    .map((n) => ({ id: n.id, name: n.name, method: n.method, node: n }));
}

/** Return folder_name keys for all collections in the tree. */
export function getCollectionKeys(tree: RequestTreeNode[]): string[] {
  return tree
    .filter((n): n is RequestTreeNode & { type: 'Collection' } => n.type === 'Collection')
    .map((n) => n.folder_name);
}

/** Check whether the tree has any root-level requests. */
export function hasRootRequests(tree: RequestTreeNode[]): boolean {
  return tree.some((n) => n.type === 'Request');
}

/** Find which collection (or null for root) a request currently lives in. */
export function findContainerKey(
  tree: RequestTreeNode[],
  requestId: string
): string | null | undefined {
  // Check root
  if (tree.some((n) => n.type === 'Request' && (n as { id: string }).id === requestId)) {
    return null;
  }
  for (const n of tree) {
    if (n.type === 'Collection') {
      if (n.children.some((c) => c.type === 'Request' && (c as { id: string }).id === requestId)) {
        return n.folder_name;
      }
    }
  }
  return undefined; // not found
}

/** Build the full dndItems map from a tree snapshot. */
export function rebuildDndItems(tree: RequestTreeNode[]): DndItems {
  const next: DndItems = {};
  for (const key of getCollectionKeys(tree)) {
    next[key] = buildDndItems(tree, key);
  }
  next[ROOT_KEY] = hasRootRequests(tree) ? buildDndItems(tree, null) : [];
  return next;
}

export type DndHandlers = {
  handleConsider: (e: CustomEvent<{ items: DndItem[] }>, containerKey: string) => void;
  handleFinalize: (
    e: CustomEvent<{ items: DndItem[]; info: { trigger: string; id: string } }>,
    containerKey: string
  ) => Promise<void>;
};

/**
 * Create DnD consider/finalize handlers.
 * `getDndItems` / `setDndItems` are get/set accessors for the reactive dndItems state.
 * `getTree` returns the current request tree snapshot.
 */
export function createDndHandlers(
  projectPath: string,
  getTree: () => RequestTreeNode[],
  getDndItems: () => DndItems,
  setDndItems: (v: DndItems) => void,
  moveRequestFn: (path: string, id: string, target: string | null) => Promise<void>,
  reload: () => Promise<void>,
  showToast: (msg: string) => void
): DndHandlers {
  function handleConsider(e: CustomEvent<{ items: DndItem[] }>, containerKey: string) {
    setDndItems({ ...getDndItems(), [containerKey]: e.detail.items });
  }

  async function handleFinalize(
    e: CustomEvent<{ items: DndItem[]; info: { trigger: string; id: string } }>,
    containerKey: string
  ) {
    const droppedId = e.detail.info.id;
    setDndItems({ ...getDndItems(), [containerKey]: e.detail.items });

    const originalKey = argToContainerKey(findContainerKey(getTree(), droppedId) ?? null);

    if (originalKey !== containerKey) {
      const targetArg = containerKeyToArg(containerKey);
      try {
        await moveRequestFn(projectPath, droppedId, targetArg);
        await reload();
        const targetLabel = targetArg === null ? 'root' : targetArg;
        showToast(`Request will now inherit auth and headers from ${targetLabel}.`);
      } catch (err) {
        console.error('Failed to move request:', err);
        await reload();
      }
    }
  }

  return { handleConsider, handleFinalize };
}

export type ContextMenuItem = { label: string; action: () => void; danger?: boolean };

/** Build context menu items for a tree node. Callbacks are provided by the caller. */
export function buildContextMenuItems(
  node: RequestTreeNode,
  onNewRequest: (collectionFolder: string | null) => void,
  onRenameRequest: (id: string) => void,
  onDeleteRequest: (id: string) => void,
  onDuplicateRequest: (id: string) => void,
  onRenameCollection: (folder: string) => void,
  onDeleteCollection: (folder: string) => void
): ContextMenuItem[] {
  if (node.type === 'Collection') {
    return [
      { label: 'New Request', action: () => onNewRequest(node.folder_name) },
      { label: 'Rename', action: () => onRenameCollection(node.folder_name) },
      { label: 'Delete Collection', action: () => onDeleteCollection(node.folder_name), danger: true },
    ];
  }
  if (node.type === 'Folder') {
    return [{ label: 'New Request', action: () => onNewRequest(null) }];
  }
  return [
    { label: 'Rename', action: () => onRenameRequest(node.id) },
    { label: 'Duplicate', action: () => onDuplicateRequest(node.id) },
    { label: 'Delete', action: () => onDeleteRequest(node.id), danger: true },
  ];
}
