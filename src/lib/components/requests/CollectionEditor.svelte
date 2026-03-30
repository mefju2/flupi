<script lang="ts">
  import { untrack } from 'svelte';
  import { project } from '$lib/stores/project';
  import { saveCollection, type CollectionData, type AuthConfig } from '$lib/services/tauri-commands';
  import { createDebouncedSave } from '$lib/services/debounced-save';
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';
  import AuthTab from './AuthTab.svelte';

  interface Props {
    folderName: string;
    collection: CollectionData;
    onUpdate?: (updated: CollectionData) => void;
  }

  let { folderName, collection, onUpdate }: Props = $props();

  // Local mutable copy — reset when switching to a different collection (folderName changes)
  let local = $state<CollectionData>({ ...collection, headers: { ...collection.headers } });

  $effect(() => {
    folderName; // track folderName as the reset signal
    local = untrack(() => ({ ...collection, headers: { ...collection.headers } }));
  });

  const debouncedSave = createDebouncedSave(async () => {
    const path = $project.path;
    if (!path) return;
    await saveCollection(path, folderName, local);
    onUpdate?.(local);
  });

  function patch(changes: Partial<CollectionData>) {
    local = { ...local, ...changes };
    debouncedSave.trigger();
  }

  let headerRows = $derived(Object.entries(local.headers).map(([key, value]) => ({ id: key, key, value })));

  function handleHeadersUpdate(rows: { key: string; value: string }[]) {
    const result: Record<string, string> = {};
    for (const row of rows) {
      if (row.key) result[row.key] = row.value;
    }
    patch({ headers: result });
  }

  function handleAuthUpdate(auth: AuthConfig) {
    patch({ auth });
  }
</script>

<div class="p-4 space-y-5 text-sm">
  <!-- Collection Name -->
  <div>
    <label for="collection-name" class="text-xs text-app-text-3 block mb-1">Collection Name</label>
    <input
      id="collection-name"
      class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm text-app-text placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
      value={local.name}
      placeholder="My Collection"
      oninput={(e) => patch({ name: e.currentTarget.value })}
    />
  </div>

  <!-- Base URL -->
  <div>
    <label for="collection-base-url" class="text-xs text-app-text-3 block mb-1">Base URL</label>
    <input
      id="collection-base-url"
      class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
      value={local.baseUrl ?? ''}
      placeholder="https://api.example.com"
      oninput={(e) => patch({ baseUrl: e.currentTarget.value })}
    />
    <p class="text-xs text-app-text-4 mt-1">Prepended to all request paths in this collection.</p>
  </div>

  <!-- Default Auth -->
  <div>
    <span class="text-xs text-app-text-3 block mb-1">Default Auth</span>
    <div class="bg-app-panel border border-app-border rounded">
      <AuthTab
        auth={local.auth}
        onUpdate={handleAuthUpdate}
      />
    </div>
  </div>

  <!-- Default Headers -->
  <div>
    <span class="text-xs text-app-text-3 block mb-2">Default Headers</span>
    <p class="text-xs text-app-text-4 mb-2">Applied to all requests — request-level headers override these.</p>
    <KeyValueTable rows={headerRows} onUpdate={handleHeadersUpdate} />
  </div>
</div>
