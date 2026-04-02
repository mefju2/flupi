<script lang="ts">
  import type { BodyConfig } from '$lib/services/tauri-commands';
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';
  import JsonEditor from '$lib/components/shared/JsonEditor.svelte';
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';

  interface Props {
    body: BodyConfig | undefined;
    onUpdate: (body: BodyConfig) => void;
    requestSchema?: unknown;
  }

  let { body, onUpdate, requestSchema = undefined }: Props = $props();

  let bodyType = $derived(body?.type ?? 'none');

  function setType(type: BodyConfig['type']) {
    if (type === 'none') onUpdate({ type: 'none' });
    else if (type === 'json') onUpdate({ type: 'json', content: '' });
    else if (type === 'form') onUpdate({ type: 'form', content: {}, disabledFields: [] });
    else if (type === 'raw') onUpdate({ type: 'raw', content: '' });
  }

  function formRows(b: BodyConfig | undefined) {
    if (b?.type !== 'form') return [];
    const disabled = b.disabledFields ?? [];
    return Object.entries(b.content).map(([key, value]) => ({
      id: key,
      key,
      value,
      enabled: !disabled.includes(key),
    }));
  }

  const jsonPlaceholder = '{ "key": "value" }';
</script>

<div class="flex flex-col h-full">
  <!-- Type switcher bar -->
  <div class="flex items-center gap-2 px-4 py-3 border-b border-app-border shrink-0">
    <span class="text-xs text-app-text-3">Body</span>
    <div class="flex border border-app-border-2 rounded overflow-hidden">
      {#each ['none', 'json', 'form', 'raw'] as t}
        <button
          class="text-xs px-2 py-0.5 transition-colors {bodyType === t ? 'bg-app-hover text-app-text' : 'text-app-text-3 hover:text-app-text-2 hover:bg-app-card'}"
          onclick={() => setType(t as BodyConfig['type'])}
        >{t === 'form' ? 'Form' : t.charAt(0).toUpperCase() + t.slice(1)}</button>
      {/each}
    </div>
  </div>

  <!-- Content area -->
  {#if body?.type === 'none' || !body}
    <div class="flex-1 overflow-y-auto px-4 py-3">
      <p class="text-sm text-app-text-3">No request body.</p>
    </div>
  {:else if body.type === 'json'}
    <div class="flex-1 overflow-hidden">
      <JsonEditor
        value={typeof body.content === 'string' ? body.content : JSON.stringify(body.content, null, 2)}
        onChange={(v) => onUpdate({ type: 'json', content: v })}
        placeholder={jsonPlaceholder}
        schema={requestSchema}
      />
    </div>
  {:else if body.type === 'form'}
    <div class="flex-1 overflow-y-auto px-4 py-3 space-y-3">
      <p class="text-xs text-app-text-4">Encoded as <span class="font-mono">application/x-www-form-urlencoded</span></p>
      <KeyValueTable
        rows={formRows(body)}
        showEnabled={true}
        onUpdate={(rows) => {
          const c: Record<string, string> = {};
          const disabled: string[] = [];
          for (const r of rows) {
            if (r.key) {
              c[r.key] = r.value;
              if (r.enabled === false) disabled.push(r.key);
            }
          }
          onUpdate({ type: 'form', content: c, disabledFields: disabled });
        }}
      />
    </div>
  {:else if body.type === 'raw'}
    <div class="flex-1 overflow-y-auto px-4 py-3">
      <VariableAutocomplete
        value={body.content}
        placeholder="Raw body content..."
        multiline={true}
        onChange={(v) => onUpdate({ type: 'raw', content: v })}
      />
    </div>
  {/if}
</div>
