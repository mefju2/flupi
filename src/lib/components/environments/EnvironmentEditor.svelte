<script lang="ts">
  import { environments, activeEnvironment, type EnvironmentEntry } from '$lib/stores/environment';
  import { saveEnvironment, saveSecrets } from '$lib/services/tauri-commands';
  import { createDebouncedSave } from '$lib/services/debounced-save';
  import { project } from '$lib/stores/project';
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';

  interface Row {
    key: string;
    value: string;
    isSecret?: boolean;
  }

  let currentEntry = $derived<EnvironmentEntry | undefined>(
    $environments.find((e) => e.fileName === $activeEnvironment)
  );

  let rows = $derived.by<Row[]>(() => {
    if (!currentEntry) return [];
    const regular = Object.entries(currentEntry.environment.variables).map(([key, value]) => ({
      key,
      value,
      isSecret: false,
    }));
    const secrets = currentEntry.environment.secrets.map((key) => ({
      key,
      value: currentEntry!.secrets[key] ?? '',
      isSecret: true,
    }));
    return [...regular, ...secrets];
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
  });

  function handleUpdate(updatedRows: Row[]) {
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

<div class="bg-zinc-950 p-6 h-full">
  {#if currentEntry}
    <h2 class="text-zinc-100 text-base font-semibold mb-4">{currentEntry.environment.name}</h2>
    <KeyValueTable
      rows={rows}
      showSecretToggle={true}
      onUpdate={handleUpdate}
    />
  {:else}
    <p class="text-zinc-600 text-sm">No environment selected.</p>
  {/if}
</div>
