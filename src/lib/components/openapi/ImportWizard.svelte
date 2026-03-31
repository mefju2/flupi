<script lang="ts">
  import { project } from '$lib/stores/project';
  import { openApiSources } from '$lib/stores/openapi';
  import { fetchOperations, importOperations, type ImportableOperation } from '$lib/services/tauri-commands';

  interface Props {
    sourceId?: string;
    onClose: () => void;
    onImported?: (requestIds: string[]) => void;
  }

  let { sourceId = $bindable(), onClose, onImported }: Props = $props();

  let step = $state<1 | 2 | 3>(1);
  let selectedSourceId = $state(sourceId ?? '');
  let operations = $state<ImportableOperation[]>([]);
  let selectedIds = $state<Set<string>>(new Set());
  let collectionFolder = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let importedCount = $state<number | null>(null);

  // If sourceId is pre-selected, fetch operations immediately
  $effect(() => {
    if (sourceId && $project.path) {
      goToStep2();
    }
  });

  let groupedOps = $derived(() => {
    const map = new Map<string, ImportableOperation[]>();
    for (const op of operations) {
      const tag = op.tag || 'default';
      if (!map.has(tag)) map.set(tag, []);
      map.get(tag)!.push(op);
    }
    return map;
  });

  function methodClass(method: string): string {
    const m = method.toUpperCase();
    if (m === 'GET') return 'bg-green-900 text-green-300';
    if (m === 'POST') return 'bg-blue-900 text-blue-300';
    if (m === 'PUT') return 'bg-yellow-900 text-yellow-300';
    if (m === 'DELETE') return 'bg-red-900 text-red-300';
    if (m === 'PATCH') return 'bg-orange-900 text-orange-300';
    return 'bg-app-card text-app-text-3';
  }

  async function goToStep2() {
    if (!$project.path || !selectedSourceId) return;
    loading = true; error = null;
    try {
      operations = await fetchOperations($project.path, selectedSourceId);
      step = 2;
    } catch (e) { error = String(e); }
    finally { loading = false; }
  }

  function toggleOp(opId: string) {
    const next = new Set(selectedIds);
    next.has(opId) ? next.delete(opId) : next.add(opId);
    selectedIds = next;
  }

  function selectAllInTag(tag: string) {
    const ops = groupedOps().get(tag) ?? [];
    const next = new Set(selectedIds);
    ops.forEach((op) => next.add(op.operationId));
    selectedIds = next;
  }

  function deselectAllInTag(tag: string) {
    const ops = groupedOps().get(tag) ?? [];
    const next = new Set(selectedIds);
    ops.forEach((op) => next.delete(op.operationId));
    selectedIds = next;
  }

  async function handleImport() {
    if (!$project.path || selectedIds.size === 0 || !collectionFolder.trim()) return;
    loading = true; error = null;
    try {
      const ids = await importOperations($project.path, selectedSourceId, [...selectedIds], collectionFolder.trim());
      importedCount = ids.length;
      onImported?.(ids);
    } catch (e) { error = String(e); }
    finally { loading = false; }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div class="fixed inset-0 bg-black/40 z-40" role="presentation" onclick={onClose}></div>

<!-- Modal -->
<div class="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
  <div class="pointer-events-auto bg-app-panel border border-app-border-2 rounded-xl shadow-2xl w-full max-w-2xl max-h-[80vh] flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-app-border">
      <span class="text-sm font-semibold text-app-text">Import from OpenAPI</span>
      <button class="text-app-text-3 hover:text-app-text-2 text-xs" onclick={onClose}>✕</button>
    </div>

    <!-- Step indicator -->
    <div class="flex items-center gap-2 px-6 py-3 border-b border-app-border text-xs">
      {#each ['Select Source', 'Choose Operations', 'Collection Folder'] as label, i}
        <span class="{step === i + 1 ? 'text-cyan-400 font-medium' : 'text-app-text-4'}">{label}</span>
        {#if i < 2}<span class="text-app-text-4">→</span>{/if}
      {/each}
    </div>

    <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-4">
      {#if importedCount !== null}
        <div class="text-center py-8">
          <p class="text-2xl text-cyan-400 font-mono">{importedCount}</p>
          <p class="text-sm text-app-text-3 mt-1">requests imported successfully</p>
          <button class="mt-4 px-4 py-2 text-sm bg-cyan-600 hover:bg-cyan-500 text-white rounded transition-colors" onclick={onClose}>Done</button>
        </div>
      {:else if step === 1}
        <!-- Step 1: Select source -->
        <div class="flex flex-col gap-2">
          <label class="text-xs text-app-text-3">Select a source</label>
          <select class="bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm text-app-text focus:outline-none focus:border-cyan-600" bind:value={selectedSourceId}>
            <option value="">-- Choose a source --</option>
            {#each $openApiSources as src}
              <option value={src.id}>{src.name}</option>
            {/each}
          </select>
        </div>
        {#if error}<p class="text-xs text-red-400">{error}</p>{/if}
        <button
          class="self-end px-3 py-1.5 text-xs bg-cyan-600 hover:bg-cyan-500 text-white rounded transition-colors disabled:opacity-50"
          disabled={!selectedSourceId || loading}
          onclick={goToStep2}
        >{loading ? 'Loading…' : 'Next →'}</button>

      {:else if step === 2}
        <!-- Step 2: Select operations -->
        <div class="flex items-center justify-between">
          <span class="text-xs text-app-text-3">{selectedIds.size} selected</span>
        </div>
        {#each [...groupedOps().entries()] as [tag, ops]}
          <div class="flex flex-col gap-1">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold text-app-text-2 uppercase tracking-wide">{tag}</span>
              <button class="text-xs text-cyan-500 hover:text-cyan-400" onclick={() => selectAllInTag(tag)}>All</button>
              <button class="text-xs text-app-text-3 hover:text-app-text-3" onclick={() => deselectAllInTag(tag)}>None</button>
            </div>
            {#each ops as op}
              <label class="flex items-center gap-2 px-2 py-1 rounded hover:bg-app-card cursor-pointer">
                <input type="checkbox" class="accent-cyan-500" checked={selectedIds.has(op.operationId)} onchange={() => toggleOp(op.operationId)} />
                <span class="font-mono text-xs px-1.5 py-0.5 rounded {methodClass(op.method)}">{op.method}</span>
                <span class="font-mono text-xs text-app-text-2 truncate">{op.path}</span>
                {#if op.summary}
                  <span class="text-xs text-app-text-3 truncate">{op.summary}</span>
                {/if}
              </label>
            {/each}
          </div>
        {/each}
        {#if operations.length === 0}
          <p class="text-sm text-app-text-4">No operations found.</p>
        {/if}
        <div class="flex items-center justify-between mt-2">
          <button
            class="px-3 py-1.5 text-xs border border-app-border-2 text-app-text-3 rounded transition-colors hover:text-app-text-2"
            onclick={() => (step = 1)}
          >← Back</button>
          <button
            class="px-3 py-1.5 text-xs bg-cyan-600 hover:bg-cyan-500 text-white rounded transition-colors disabled:opacity-50"
            disabled={selectedIds.size === 0}
            onclick={() => (step = 3)}
          >Next →</button>
        </div>

      {:else if step === 3}
        <!-- Step 3: Collection folder -->
        <div class="flex flex-col gap-2">
          <label class="text-xs text-app-text-3">Collection folder name</label>
          <input
            class="bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm font-mono text-app-text focus:outline-none focus:border-cyan-600"
            bind:value={collectionFolder}
            placeholder="my-api"
          />
          <p class="text-xs text-app-text-3 mt-1">Lowercase letters, numbers, and hyphens (e.g. users-service)</p>
          <p class="text-xs text-app-text-4">{selectedIds.size} operations will be imported</p>
        </div>
        {#if error}<p class="text-xs text-red-400">{error}</p>{/if}
        <div class="flex items-center justify-between mt-2">
          <button
            class="px-3 py-1.5 text-xs border border-app-border-2 text-app-text-3 rounded transition-colors hover:text-app-text-2"
            onclick={() => (step = 2)}
          >← Back</button>
          <button
            class="px-3 py-1.5 text-xs bg-cyan-600 hover:bg-cyan-500 text-white rounded transition-colors disabled:opacity-50"
            disabled={!collectionFolder.trim() || loading}
            onclick={handleImport}
          >{loading ? 'Importing…' : 'Import'}</button>
        </div>
      {/if}
    </div>
  </div>
</div>
