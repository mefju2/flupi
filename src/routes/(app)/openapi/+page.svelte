<script lang="ts">
  import { onMount } from 'svelte';
  import { project } from '$lib/stores/project';
  import { openApiSources } from '$lib/stores/openapi';
  import { listOpenApiSources } from '$lib/services/tauri-commands';
  import SourceList from '$lib/components/openapi/SourceList.svelte';
  import AddSourceForm from '$lib/components/openapi/AddSourceForm.svelte';
  import ImportWizard from '$lib/components/openapi/ImportWizard.svelte';

  let showAddForm = $state(false);
  let wizardSourceId = $state<string | null>(null);
  let newlyAddedSourceId = $state<string | null>(null);
  let newlyAddedTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    if (!$project.path) return;
    try { openApiSources.set(await listOpenApiSources($project.path)); }
    catch (e) { console.error('Failed to load OpenAPI sources:', e); }
  });

  function handleAddSource() { showAddForm = true; }
  function handleCloseAddForm() { showAddForm = false; }
  function handleSourceAdded(sourceId: string) {
    newlyAddedSourceId = sourceId;
    if (newlyAddedTimer) clearTimeout(newlyAddedTimer);
    newlyAddedTimer = setTimeout(() => {
      newlyAddedSourceId = null;
      newlyAddedTimer = null;
    }, 2000);
  }
  function handleOpenImport(sourceId: string) { wizardSourceId = sourceId; }
  function handleCloseWizard() { wizardSourceId = null; }
</script>

<div class="flex flex-col h-full bg-app-bg overflow-y-auto">
  <div class="px-6 py-4 border-b border-app-border">
    <h1 class="text-base font-semibold text-app-text">OpenAPI Sources</h1>
    <p class="text-xs text-app-text-3 mt-0.5">Import API operations from OpenAPI 3.0 specifications.</p>
  </div>

  <div class="p-6 flex flex-col gap-4 max-w-2xl">
    {#if showAddForm}
      <AddSourceForm onClose={handleCloseAddForm} onAdded={handleSourceAdded} />
    {/if}

    <SourceList onAddSource={handleAddSource} onImport={handleOpenImport} addedSourceId={newlyAddedSourceId} />
  </div>
</div>

{#if wizardSourceId !== null}
  <ImportWizard sourceId={wizardSourceId} onClose={handleCloseWizard} />
{/if}
