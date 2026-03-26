import { writable } from 'svelte/store';

export interface ProjectState {
  isOpen: boolean;
  path: string | null;
  name: string | null;
}

export const project = writable<ProjectState>({
  isOpen: false,
  path: null,
  name: null,
});
