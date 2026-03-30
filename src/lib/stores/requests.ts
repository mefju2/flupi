import { writable } from 'svelte/store';
import type { RequestTreeNode, RequestData, CollectionData } from '$lib/services/tauri-commands';

export const requestTree = writable<RequestTreeNode[]>([]);
export const activeRequestId = writable<string | null>(null);
export const activeRequest = writable<RequestData | null>(null);
export const activeCollectionFolder = writable<string | null>(null);
export const activeCollection = writable<CollectionData | null>(null);
