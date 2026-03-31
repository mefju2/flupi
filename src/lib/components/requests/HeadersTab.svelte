<script lang="ts">
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';

  interface Row {
    id: string;
    key: string;
    value: string;
    enabled?: boolean;
  }

  interface Props {
    headers: Record<string, string>;
    disabledHeaders?: string[];
    collectionHeaders?: Record<string, string>;
    disabledCollectionHeaders?: string[];
    onUpdate: (headers: Record<string, string>, disabledHeaders: string[]) => void;
    onDisabledCollectionHeadersChange?: (disabled: string[]) => void;
  }

  let {
    headers,
    disabledHeaders = [],
    collectionHeaders,
    disabledCollectionHeaders = [],
    onUpdate,
    onDisabledCollectionHeadersChange,
  }: Props = $props();

  let rows = $derived(
    Object.entries(headers).map(([key, value]) => ({
      id: key,
      key,
      value,
      enabled: !disabledHeaders.includes(key),
    }))
  );

  let collectionRows = $derived(
    collectionHeaders
      ? Object.entries(collectionHeaders).map(([key, value]) => ({ key, value, enabled: !disabledCollectionHeaders.includes(key) }))
      : []
  );

  function handleUpdate(updated: Row[]) {
    const result: Record<string, string> = {};
    const disabled: string[] = [];
    for (const row of updated) {
      if (row.key) {
        result[row.key] = row.value;
        if (row.enabled === false) disabled.push(row.key);
      }
    }
    onUpdate(result, disabled);
  }

  function toggleCollectionHeader(key: string, enabled: boolean) {
    const next = enabled
      ? disabledCollectionHeaders.filter((k) => k !== key)
      : [...disabledCollectionHeaders, key];
    onDisabledCollectionHeadersChange?.(next);
  }
</script>

<div class="p-4 space-y-4">
  {#if collectionRows.length > 0}
    <div>
      <p class="text-xs text-app-text-3 mb-2">Inherited from collection</p>
      <div class="space-y-1">
        {#each collectionRows as row (row.key)}
          <div class="flex gap-2 items-center {!row.enabled ? 'opacity-40' : ''}">
            <input
              type="checkbox"
              checked={row.enabled}
              onchange={(e) => toggleCollectionHeader(row.key, e.currentTarget.checked)}
              class="accent-cyan-500 shrink-0"
              aria-label="Enable inherited header"
            />
            <span class="flex-1 bg-app-panel border border-app-border rounded px-2 py-1 text-sm font-mono text-app-text-3">{row.key}</span>
            <span class="flex-1 bg-app-panel border border-app-border rounded px-2 py-1 text-sm font-mono text-app-text-3">{row.value}</span>
            <div class="w-6"></div>
          </div>
        {/each}
      </div>
    </div>
    <div class="border-t border-app-border"></div>
  {/if}

  <KeyValueTable rows={rows} showEnabled={true} onUpdate={handleUpdate} />
</div>
