<script lang="ts" module>
  const schemaCache = new Map<string, { requestSchema: unknown; responseSchema: unknown; requestPath: string | null }>();
</script>

<script lang="ts">
  import type { ScenarioStep, RequestTreeNode } from '$lib/services/tauri-commands';
  import { getRequest } from '$lib/services/tauri-commands';
  import { project } from '$lib/stores/project';
  import { getMethodColor } from '$lib/utils/format';
  import OverridesPanel from './OverridesPanel.svelte';
  import ExtractionsPanel from '$lib/components/shared/ExtractionsPanel.svelte';
  import RequestPicker from './RequestPicker.svelte';
  import SectionHeader from '$lib/components/shared/SectionHeader.svelte';

  interface Props {
    step: ScenarioStep;
    requestTree: RequestTreeNode[];
    index: number;
    extractedVars?: string[];
    onUpdate: (step: ScenarioStep) => void;
    onDelete: () => void;
  }

  let { step, requestTree, index, extractedVars = [], onUpdate, onDelete }: Props = $props();

  let expanded = $state(false);
  let requestSchema = $state<unknown>(null);
  let responseSchema = $state<unknown>(null);
  let requestPath = $state<string | null>(null);

  $effect(() => {
    const id = step.requestId;
    const path = $project?.path;
    if (!id || !path) return;

    const cacheKey = `${path}:${id}`;
    if (schemaCache.has(cacheKey)) {
      const cached = schemaCache.get(cacheKey)!;
      requestSchema = cached.requestSchema;
      responseSchema = cached.responseSchema;
      requestPath = cached.requestPath;
      return;
    }

    getRequest(path, id).then((r) => {
      const schemas = {
        requestSchema: r.templateRef?.requestSchema ?? null,
        responseSchema: r.templateRef?.responseSchema ?? null,
        requestPath: r.path,
      };
      schemaCache.set(cacheKey, schemas);
      requestSchema = schemas.requestSchema;
      responseSchema = schemas.responseSchema;
      requestPath = schemas.requestPath;
    }).catch(() => {
      requestSchema = null;
      responseSchema = null;
      requestPath = null;
    });
  });

  function findRequest(
    nodes: RequestTreeNode[],
    id: string,
  ): { method: string; name: string; path: string } | null {
    for (const node of nodes) {
      if (node.type === 'Request' && node.id === id) return { method: node.method, name: node.name, path: '' };
      if ((node.type === 'Collection' || node.type === 'Folder') && node.children) {
        const found = findRequest(node.children, id);
        if (found) return found;
      }
    }
    return null;
  }

  let requestInfo = $derived(findRequest(requestTree, step.requestId));
  let methodColor = $derived(requestInfo ? getMethodColor(requestInfo.method) : 'text-app-text-4');
</script>

<div class="border border-app-border rounded bg-app-panel mb-2 {expanded ? 'border-l-2 border-l-cyan-500' : 'border-l-2 border-l-transparent'}">
  <div class="flex items-center gap-2 px-3 py-2 cursor-pointer select-none" role="button" tabindex="0"
    onclick={() => expanded = !expanded}
    onkeydown={(e) => e.key === 'Enter' && (expanded = !expanded)}
  >
    <span class="drag-handle text-app-text-4 hover:text-app-text-3 cursor-grab active:cursor-grabbing text-xs shrink-0">⠿</span>
    <span class="text-xs text-app-text-3 w-5 shrink-0">{index + 1}</span>

    <div class="flex-1 flex items-center gap-2 min-w-0">
      <span class="font-mono text-sm text-app-text truncate">{step.name || 'Unnamed Step'}</span>
      {#if !requestInfo}
        <span class="text-xs text-amber-400 bg-amber-950/40 border border-amber-800/60 rounded px-1.5 py-0.5 shrink-0">
          Request not found
        </span>
      {:else}
        <span class="font-mono text-xs {methodColor} shrink-0">{requestInfo.method}</span>
        <span class="font-mono text-xs text-app-text-3 truncate">{requestInfo.name}</span>
      {/if}
    </div>

    <div class="flex items-center gap-2 shrink-0">
      <button
        class="text-app-text-4 hover:text-red-400 transition-colors text-base"
        onclick={(e) => { e.stopPropagation(); onDelete(); }}
        aria-label="Delete step"
      >×</button>
      <span class="text-app-text-3 text-xs">{expanded ? '▾' : '▸'}</span>
    </div>
  </div>

  {#if expanded}
    <div class="border-t border-app-border px-3 py-3 space-y-4">
      <div class="flex gap-3">
        <div class="flex-1">
          <label class="block text-xs text-app-text-3 mb-1">Step Name</label>
          <input
            class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text focus:outline-none focus:border-app-border-2"
            value={step.name}
            oninput={(e) => onUpdate({ ...step, name: e.currentTarget.value })}
            placeholder="Step name"
          />
        </div>
        <div class="flex-1">
          <label class="block text-xs text-app-text-3 mb-1">Request</label>
          <RequestPicker
            {requestTree}
            value={step.requestId}
            onChange={(id) => onUpdate({ ...step, requestId: id })}
          />
        </div>
      </div>

      <div>
        <SectionHeader class="mb-2">Overrides</SectionHeader>
        <OverridesPanel
          overrides={step.overrides}
          {requestSchema}
          requestPath={requestPath ?? undefined}
          {extractedVars}
          onUpdate={(overrides) => onUpdate({ ...step, overrides })}
        />
      </div>

      <div>
        <SectionHeader class="mb-2">Extractions</SectionHeader>
        <ExtractionsPanel
          extractions={step.extract}
          {responseSchema}
          onUpdate={(extract) => onUpdate({ ...step, extract })}
        />
      </div>
    </div>
  {/if}
</div>
