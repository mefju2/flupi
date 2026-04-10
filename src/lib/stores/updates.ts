import { writable } from 'svelte/store';
import { check, type Update } from '@tauri-apps/plugin-updater';

export const pendingUpdate = writable<Update | null>(null);
export const updateChecking = writable(false);

export async function checkForUpdates(): Promise<void> {
  updateChecking.set(true);
  try {
    const result = await check();
    pendingUpdate.set(result);
  } catch {
    // Network error or no update manifest — silently ignore on background checks.
  } finally {
    updateChecking.set(false);
  }
}
