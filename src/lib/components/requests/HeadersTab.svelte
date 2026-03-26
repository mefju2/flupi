<script lang="ts">
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';

  interface Row {
    key: string;
    value: string;
  }

  interface Props {
    headers: Record<string, string>;
    onUpdate: (headers: Record<string, string>) => void;
  }

  let { headers, onUpdate }: Props = $props();

  let rows = $derived(Object.entries(headers).map(([key, value]) => ({ key, value })));

  function handleUpdate(updated: Row[]) {
    const result: Record<string, string> = {};
    for (const row of updated) {
      if (row.key) result[row.key] = row.value;
    }
    onUpdate(result);
  }
</script>

<div class="p-4">
  <div class="mb-3 flex items-center gap-2 py-1.5 px-2 bg-zinc-900 border border-zinc-800 rounded">
    <span class="text-xs text-zinc-500 italic">Collection headers will be visible here once a collection is selected.</span>
  </div>
  <KeyValueTable rows={rows} onUpdate={handleUpdate} />
</div>
