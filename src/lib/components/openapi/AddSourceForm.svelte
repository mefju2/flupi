<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { project } from '$lib/stores/project';
  import { openApiSources } from '$lib/stores/openapi';
  import { addOpenApiSource, listOpenApiSources, type OpenApiSource } from '$lib/services/tauri-commands';

  interface Props {
    onClose: () => void;
    onAdded?: (sourceId: string) => void;
  }

  let { onClose, onAdded }: Props = $props();

  let tab = $state<'url' | 'file'>('url');
  let name = $state('');
  let url = $state('');
  let filePath = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);

  function slugify(s: string): string {
    return s.toLowerCase().trim().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
  }

  let generatedId = $derived(slugify(name) || crypto.randomUUID().slice(0, 8));

  async function handleBrowse() {
    const result = await open({
      filters: [{ name: 'OpenAPI', extensions: ['json', 'yaml', 'yml'] }],
    });
    if (result && typeof result === 'string') {
      filePath = result;
      if (!name) {
        const parts = filePath.split('/');
        name = parts[parts.length - 1].replace(/\.(json|yaml|yml)$/, '');
      }
    }
  }

  async function handleAdd() {
    if (!$project.path) return;
    if (!name.trim()) { error = 'Name is required'; return; }
    if (tab === 'url' && !url.trim()) { error = 'URL is required'; return; }
    if (tab === 'file' && !filePath.trim()) { error = 'File path is required'; return; }

    loading = true;
    error = null;
    try {
      const source: OpenApiSource = tab === 'url'
        ? { type: 'url', id: generatedId, name: name.trim(), url: url.trim(), lastFetchedAt: null, lastHash: null }
        : { type: 'file', id: generatedId, name: name.trim(), path: filePath.trim(), lastFetchedAt: null, lastHash: null };

      await addOpenApiSource($project.path, source);
      openApiSources.set(await listOpenApiSources($project.path));
      onAdded?.(generatedId);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="bg-app-panel border border-app-border-2 rounded-lg p-4 flex flex-col gap-4">
  <div class="flex items-center justify-between">
    <span class="text-sm font-semibold text-app-text">Add OpenAPI Source</span>
    <button class="text-app-text-3 hover:text-app-text-2 text-xs" onclick={onClose}>✕</button>
  </div>

  <!-- Tab selector -->
  <div class="flex gap-1 bg-app-card rounded p-0.5 w-fit">
    <button
      class="px-3 py-1 text-xs rounded transition-colors {tab === 'url' ? 'bg-cyan-600 text-white' : 'bg-app-card text-app-text-3 hover:bg-app-hover'}"
      onclick={() => (tab = 'url')}
    >URL</button>
    <button
      class="px-3 py-1 text-xs rounded transition-colors {tab === 'file' ? 'bg-cyan-600 text-white' : 'bg-app-card text-app-text-3 hover:bg-app-hover'}"
      onclick={() => (tab = 'file')}
    >File</button>
  </div>

  <!-- Name field -->
  <div class="flex flex-col gap-1">
    <label class="text-xs text-app-text-3">Name</label>
    <input
      class="bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm text-app-text focus:outline-none focus:border-cyan-600"
      bind:value={name}
      placeholder="My API"
    />
    <p
      class="text-xs text-app-text-4 font-mono"
      title="Auto-generated from name. Used internally to identify this source."
    >ID: {generatedId}</p>
  </div>

  <!-- URL or file input -->
  {#if tab === 'url'}
    <div class="flex flex-col gap-1">
      <label class="text-xs text-app-text-3">URL</label>
      <input
        class="bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm font-mono text-app-text focus:outline-none focus:border-cyan-600"
        bind:value={url}
        placeholder="https://api.example.com/openapi.json"
      />
    </div>
  {:else}
    <div class="flex flex-col gap-1">
      <label class="text-xs text-app-text-3">File</label>
      <div class="flex gap-2">
        <input
          class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm font-mono text-app-text focus:outline-none focus:border-cyan-600"
          bind:value={filePath}
          placeholder="/path/to/openapi.yaml"
          readonly
        />
        <button
          class="px-3 py-1.5 text-xs bg-app-card hover:bg-app-hover border border-app-border-2 text-app-text-2 rounded transition-colors"
          onclick={handleBrowse}
        >Browse…</button>
      </div>
    </div>
  {/if}

  {#if error}
    <p class="text-xs text-red-400">{error}</p>
  {/if}

  <div class="flex gap-2 justify-end">
    <button
      class="px-3 py-1.5 text-xs bg-app-card hover:bg-app-hover text-app-text-2 rounded transition-colors"
      onclick={onClose}
    >Cancel</button>
    <button
      class="px-3 py-1.5 text-xs bg-cyan-600 hover:bg-cyan-500 text-white rounded transition-colors disabled:opacity-50"
      disabled={loading}
      onclick={handleAdd}
    >{loading ? 'Adding…' : 'Add Source'}</button>
  </div>
</div>
