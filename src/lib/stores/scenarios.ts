import { writable } from 'svelte/store';
import type { ScenarioTreeNode, ScenarioData } from '$lib/services/tauri-commands';

export const scenarioTree = writable<ScenarioTreeNode[]>([]);
export const activeScenarioId = writable<string | null>(null);
export const activeScenario = writable<ScenarioData | null>(null);
