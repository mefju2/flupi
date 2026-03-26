import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light' | 'system';

export const theme = writable<Theme>('dark');

theme.subscribe((value) => {
  if (typeof document === 'undefined') return;
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  if (value === 'dark' || (value === 'system' && prefersDark)) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
});
