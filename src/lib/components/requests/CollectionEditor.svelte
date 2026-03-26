<script lang="ts">
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

  // Local mutable copy
  let local = $state<CollectionData>({ ...collection, headers: { ...collection.headers } });

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

  let headerRows = $derived(Object.entries(local.headers).map(([key, value]) => ({ key, value })));

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
    <label class="text-xs text-zinc-400 block mb-1">Collection Name</label>
    <input
      class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
      value={local.name}
      placeholder="My Collection"
      oninput={(e) => patch({ name: e.currentTarget.value })}
    />
  </div>

  <!-- Base URL -->
  <div>
    <label class="text-xs text-zinc-400 block mb-1">Base URL</label>
    <input
      class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
      value={local.baseUrl ?? ''}
      placeholder="https://api.example.com"
      oninput={(e) => patch({ baseUrl: e.currentTarget.value })}
    />
    <p class="text-xs text-zinc-600 mt-1">Prepended to all request paths in this collection.</p>
  </div>

  <!-- Default Auth -->
  <div>
    <label class="text-xs text-zinc-400 block mb-1">Default Auth</label>
    <div class="bg-zinc-900 border border-zinc-800 rounded">
      <AuthTab
        auth={local.auth}
        onUpdate={handleAuthUpdate}
      />
    </div>
  </div>

  <!-- Default Headers -->
  <div>
    <label class="text-xs text-zinc-400 block mb-2">Default Headers</label>
    <p class="text-xs text-zinc-600 mb-2">Applied to all requests — request-level headers override these.</p>
    <KeyValueTable rows={headerRows} onUpdate={handleHeadersUpdate} />
  </div>
</div>
