import { writable } from 'svelte/store';
import type { OpenApiSource } from '$lib/services/tauri-commands';

export const openApiSources = writable<OpenApiSource[]>([]);
export const driftedRequestIds = writable<Set<string>>(new Set());
