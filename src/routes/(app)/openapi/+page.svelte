<script lang="ts">
  import { onMount } from 'svelte';
  import { project } from '$lib/stores/project';
  import { openApiSources, driftedIdsBySource } from '$lib/stores/openapi';
  import { listOpenApiSources, refreshSource } from '$lib/services/tauri-commands';
  import SourceList from '$lib/components/openapi/SourceList.svelte';
  import SourceDetail from '$lib/components/openapi/SourceDetail.svelte';
  import AddSourceForm from '$lib/components/openapi/AddSourceForm.svelte';
  import ImportWizard from '$lib/components/openapi/ImportWizard.svelte';

  let showAddForm = $state(false);
  let wizardSourceId = $state<string | null>(null);
  let newlyAddedSourceId = $state<string | null>(null);
  let selectedSourceId = $state<string | null>(null);
  let newlyAddedTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    if (!$project.path) return;
    try { openApiSources.set(await listOpenApiSources($project.path)); }
    catch (e) { console.error('Failed to load OpenAPI sources:', e); }
  });

  onMount(() => {
    const onNewSource = () => { showAddForm = true; };
    const onSyncAll = async () => {
      const projectPath = $project.path;
      if (!projectPath) return;
      const sources = $openApiSources;
      await Promise.all(
        sources.map(async (source) => {
          try {
            const drifted = await refreshSource(projectPath, source.id);
            driftedIdsBySource.update((prev) => {
              const next = new Map(prev);
              next.set(source.id, drifted);
              return next;
            });
          } catch {
            // individual source failures don't block others
          }
        }),
      );
    };
    window.addEventListener('flupi:new-openapi-source', onNewSource);
    window.addEventListener('flupi:sync-all', onSyncAll);
    return () => {
      window.removeEventListener('flupi:new-openapi-source', onNewSource);
      window.removeEventListener('flupi:sync-all', onSyncAll);
      if (newlyAddedTimer) {
        clearTimeout(newlyAddedTimer);
        newlyAddedTimer = null;
      }
    };
  });

  function handleAddSource() { showAddForm = true; }
  function handleCloseAddForm() { showAddForm = false; }
  function handleSourceAdded(sourceId: string) {
    newlyAddedSourceId = sourceId;
    selectedSourceId = sourceId;
    if (newlyAddedTimer) clearTimeout(newlyAddedTimer);
    newlyAddedTimer = setTimeout(() => {
      newlyAddedSourceId = null;
      newlyAddedTimer = null;
    }, 2000);
  }
  function handleOpenImport(sourceId: string) { wizardSourceId = sourceId; }
  function handleCloseWizard() { wizardSourceId = null; }
</script>

<div class="flex flex-col h-full bg-app-bg overflow-hidden">
  <div class="px-6 py-4 border-b border-app-border shrink-0">
    <h1 class="text-base font-semibold text-app-text">OpenAPI Sources</h1>
    <p class="text-xs text-app-text-3 mt-0.5">Import API operations from OpenAPI 3.0 specifications.</p>
  </div>

  <div class="flex flex-1 min-h-0">
    <!-- Left column: source list -->
    <div class="w-80 shrink-0 border-r border-app-border overflow-y-auto p-4 flex flex-col gap-4">
      {#if showAddForm}
        <AddSourceForm onClose={handleCloseAddForm} onAdded={handleSourceAdded} />
      {/if}
      <SourceList
        onAddSource={handleAddSource}
        onImport={handleOpenImport}
        onSelectSource={(id) => { selectedSourceId = id; }}
        selectedSourceId={selectedSourceId}
        addedSourceId={newlyAddedSourceId}
      />
    </div>

    <!-- Right column: detail panel -->
    <div class="flex-1 min-w-0 overflow-y-auto">
      {#if selectedSourceId}
        <SourceDetail sourceId={selectedSourceId} />
      {:else}
        <div class="flex items-center justify-center h-full">
          <p class="text-app-text-4 text-sm">Select a source to view details and imported requests.</p>
        </div>
      {/if}
    </div>
  </div>
</div>

{#if wizardSourceId !== null}
  <ImportWizard sourceId={wizardSourceId} onClose={handleCloseWizard} />
{/if}
