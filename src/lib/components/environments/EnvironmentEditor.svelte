<script lang="ts">
  import { environments, selectedEnvironmentFile, type EnvironmentEntry } from '$lib/stores/environment';
  import { saveEnvironment, saveSecrets } from '$lib/services/tauri-commands';
  import { createDebouncedSave } from '$lib/services/debounced-save';
  import { project } from '$lib/stores/project';
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';
  import { fade } from 'svelte/transition';

  interface Row {
    key: string;
    value: string;
    isSecret?: boolean;
  }

  let currentEntry = $derived<EnvironmentEntry | undefined>(
    $environments.find((e) => e.fileName === $selectedEnvironmentFile)
  );

  let rows = $state<Row[]>([]);
  let syncedFileName = $state<string | undefined>(undefined);
  let savedRecently = $state(false);

  $effect(() => {
    const fileName = currentEntry?.fileName;
    if (fileName === syncedFileName) return;
    syncedFileName = fileName;
    if (!currentEntry) { rows = []; return; }
    rows = [
      ...Object.entries(currentEntry.environment.variables).map(([key, value]) => ({ key, value, isSecret: false as const })),
      ...currentEntry.environment.secrets.map((key) => ({ key, value: currentEntry!.secrets[key] ?? '', isSecret: true as const })),
    ];
  });

  const debouncedSave = createDebouncedSave(async () => {
    if (!currentEntry || !$project.path) return;
    const variables: Record<string, string> = {};
    const secretKeys: string[] = [];
    const secretValues: Record<string, string> = {};

    for (const row of rows) {
      if (!row.key) continue;
      if (row.isSecret) {
        secretKeys.push(row.key);
        secretValues[row.key] = row.value;
      } else {
        variables[row.key] = row.value;
      }
    }

    const updatedEnv = {
      ...currentEntry.environment,
      variables,
      secrets: secretKeys,
    };

    await saveEnvironment($project.path, currentEntry.fileName, updatedEnv);
    await saveSecrets($project.path, currentEntry.fileName, secretValues);
    savedRecently = true;
    setTimeout(() => (savedRecently = false), 2000);
  });

  function handleUpdate(updatedRows: Row[]) {
    rows = updatedRows;
    if (!currentEntry) return;
    const variables: Record<string, string> = {};
    const secretKeys: string[] = [];
    const secretValues: Record<string, string> = {};

    for (const row of updatedRows) {
      if (!row.key) continue;
      if (row.isSecret) {
        secretKeys.push(row.key);
        secretValues[row.key] = row.value;
      } else {
        variables[row.key] = row.value;
      }
    }

    environments.update((list) =>
      list.map((e) =>
        e.fileName === currentEntry!.fileName
          ? {
              ...e,
              environment: { ...e.environment, variables, secrets: secretKeys },
              secrets: secretValues,
            }
          : e
      )
    );

    debouncedSave.trigger();
  }
</script>

<div class="bg-app-bg p-6 h-full">
  {#if currentEntry}
    <div class="flex items-center gap-3 mb-4">
      <h2 class="text-app-text text-base font-semibold">{currentEntry.environment.name}</h2>
      {#if savedRecently}
        <span class="text-xs text-green-400" transition:fade={{ duration: 150 }}>Saved</span>
      {/if}
    </div>
    <KeyValueTable
      rows={rows}
      showSecretToggle={true}
      onUpdate={handleUpdate}
    />
  {:else}
    <p class="text-app-text-4 text-sm">No environment selected.</p>
  {/if}
</div>
