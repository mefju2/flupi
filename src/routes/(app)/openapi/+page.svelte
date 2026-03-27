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

  onMount(async () => {
    if (!$project.path) return;
    try { openApiSources.set(await listOpenApiSources($project.path)); }
    catch (e) { console.error('Failed to load OpenAPI sources:', e); }
  });

  function handleAddSource() { showAddForm = true; }
  function handleCloseAddForm() { showAddForm = false; }
  function handleSourceAdded(sourceId: string) {
    newlyAddedSourceId = sourceId;
    setTimeout(() => { newlyAddedSourceId = null; }, 2000);
  }
  function handleOpenImport(sourceId: string) { wizardSourceId = sourceId; }
  function handleCloseWizard() { wizardSourceId = null; }
</script>

<div class="flex flex-col h-full bg-zinc-950 overflow-y-auto">
  <div class="px-6 py-4 border-b border-zinc-800">
    <h1 class="text-base font-semibold text-zinc-100">OpenAPI Sources</h1>
    <p class="text-xs text-zinc-500 mt-0.5">Import API operations from OpenAPI 3.0 specifications.</p>
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
