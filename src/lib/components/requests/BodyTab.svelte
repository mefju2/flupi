<script lang="ts">
  import type { BodyConfig } from '$lib/services/tauri-commands';
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';
  import JsonEditor from '$lib/components/shared/JsonEditor.svelte';
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';

  interface Props {
    body: BodyConfig | undefined;
    onUpdate: (body: BodyConfig) => void;
  }

  let { body, onUpdate }: Props = $props();

  let bodyType = $derived(body?.type ?? 'none');

  function setType(type: BodyConfig['type']) {
    if (type === 'none') onUpdate({ type: 'none' });
    else if (type === 'json') onUpdate({ type: 'json', content: '' });
    else if (type === 'form') onUpdate({ type: 'form', content: {} });
    else if (type === 'raw') onUpdate({ type: 'raw', content: '' });
  }

  function formRows(b: BodyConfig | undefined) {
    if (b?.type !== 'form') return [];
    return Object.entries(b.content).map(([key, value]) => ({ key, value }));
  }

  const jsonPlaceholder = '{ "key": "value" }';
</script>

<div class="p-4 space-y-3">
  <div class="flex items-center gap-2">
    <span class="text-xs text-zinc-400">Body</span>
    {#each ['none', 'json', 'form', 'raw'] as t}
      <button
        class="text-xs px-2 py-0.5 rounded transition-colors {bodyType === t ? 'bg-zinc-700 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}"
        onclick={() => setType(t as BodyConfig['type'])}
      >{t.charAt(0).toUpperCase() + t.slice(1)}</button>
    {/each}
  </div>

  {#if body?.type === 'none' || !body}
    <p class="text-sm text-zinc-500">No request body.</p>
  {:else if body.type === 'json'}
    <JsonEditor
      value={typeof body.content === 'string' ? body.content : JSON.stringify(body.content, null, 2)}
      onChange={(v) => onUpdate({ type: 'json', content: v })}
      placeholder={jsonPlaceholder}
    />
  {:else if body.type === 'form'}
    <KeyValueTable
      rows={formRows(body)}
      onUpdate={(rows) => {
        const c: Record<string, string> = {};
        for (const r of rows) { if (r.key) c[r.key] = r.value; }
        onUpdate({ type: 'form', content: c });
      }}
    />
  {:else if body.type === 'raw'}
    <VariableAutocomplete
      value={body.content}
      placeholder="Raw body content..."
      multiline={true}
      onChange={(v) => onUpdate({ type: 'raw', content: v })}
    />
  {/if}
</div>
