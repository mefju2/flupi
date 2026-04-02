import { writable, derived } from 'svelte/store';
import type { OpenApiSource } from '$lib/services/tauri-commands';

export const openApiSources = writable<OpenApiSource[]>([]);

// Maps sourceId → request IDs that are drifted for that source.
// Updated atomically on each sync so a re-sync clears stale IDs for that source.
export const driftedIdsBySource = writable<Map<string, string[]>>(new Map());

// Flat set of all drifted request IDs — consumed by TreeNode and DriftPanel.
export const driftedRequestIds = derived(
  driftedIdsBySource,
  ($map) => new Set<string>([...$map.values()].flat()),
);
