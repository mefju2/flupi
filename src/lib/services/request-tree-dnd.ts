import type { RequestTreeNode } from '$lib/services/tauri-commands';

export type DndItem = {
  id: string;
  name: string;
  method: string;
  node: RequestTreeNode & { type: 'Request' };
};

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
  if (tree.some((n) => n.type === 'Request' && n.type === 'Request' && (n as { id: string }).id === requestId)) {
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
