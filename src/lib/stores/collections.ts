import { writable } from 'svelte/store';

// Active collection folder for context (e.g. "new request" knows which collection)
export const activeCollectionFolder = writable<string | null>(null);
