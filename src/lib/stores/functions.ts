import { writable } from 'svelte/store';
import type { ScriptFunction } from '$lib/services/tauri-commands';

export const functions = writable<ScriptFunction[]>([]);
export const selectedFunctionName = writable<string | null>(null);
