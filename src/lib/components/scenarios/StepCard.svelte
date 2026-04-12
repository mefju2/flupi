<script lang="ts" module>
  const schemaCache = new Map<string, { requestSchema: unknown; responseSchema: unknown; requestPath: string | null }>();
</script>

<script lang="ts">
  import type { RequestStep, RequestTreeNode } from '$lib/services/tauri-commands';
  import { getRequest } from '$lib/services/tauri-commands';
  import { project } from '$lib/stores/project';
  import { getMethodColor } from '$lib/utils/format';
  import OverridesPanel from './OverridesPanel.svelte';
  import ExtractionsPanel from '$lib/components/shared/ExtractionsPanel.svelte';
  import RequestPicker from './RequestPicker.svelte';
  import SectionHeader from '$lib/components/shared/SectionHeader.svelte';

  interface VarMeta {
    name: string;
    kind: 'input' | 'local';
    description?: string;
    defaultValue?: string;
  }

  interface Props {
    step: RequestStep;
    requestTree: RequestTreeNode[];
    index: number;
    extractedVars?: VarMeta[];
    onUpdate: (step: RequestStep) => void;
    onDelete: () => void;
    onMoveUp?: () => void;
    onMoveDown?: () => void;
    onInputEdit?: (name: string, value: string) => void;
  }

  let { step, requestTree, index, extractedVars = [], onUpdate, onDelete, onMoveUp, onMoveDown, onInputEdit }: Props = $props();

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

  let statusInput = $state('');

  function isValidStatusPattern(p: string) {
    return p.length === 3 && /^[\d*]{3}$/.test(p);
  }

  function addStatus() {
    const code = statusInput.trim();
    if (!isValidStatusPattern(code)) return;
    const current = step.expectedStatus ?? [];
    if (!current.includes(code)) {
      onUpdate({ ...step, expectedStatus: [...current, code] });
    }
    statusInput = '';
  }

  function removeStatus(code: string) {
    onUpdate({ ...step, expectedStatus: (step.expectedStatus ?? []).filter((c) => c !== code) });
  }
</script>

<div class="border border-app-border rounded bg-app-panel mb-2 {expanded ? 'border-l-2 border-l-cyan-500' : 'border-l-2 border-l-transparent'}">
  <div class="flex items-center gap-2 px-3 py-2 cursor-pointer select-none" role="button" tabindex="0"
    onclick={() => expanded = !expanded}
    onkeydown={(e) => e.key === 'Enter' && (expanded = !expanded)}
  >
    <div class="flex flex-col gap-0.5 shrink-0" onclick={(e) => e.stopPropagation()} role="none">
      <button
        type="button"
        aria-label="Move step up"
        class="text-app-text-4 hover:text-app-text-3 transition-colors bg-transparent border-0 p-0 leading-none text-[10px] {onMoveUp ? '' : 'opacity-40 cursor-not-allowed'}"
        disabled={!onMoveUp}
        onclick={() => onMoveUp?.()}
      >▲</button>
      <button
        type="button"
        aria-label="Move step down"
        class="text-app-text-4 hover:text-app-text-3 transition-colors bg-transparent border-0 p-0 leading-none text-[10px] {onMoveDown ? '' : 'opacity-40 cursor-not-allowed'}"
        disabled={!onMoveDown}
        onclick={() => onMoveDown?.()}
      >▼</button>
    </div>
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
          <label for="step-name" class="block text-xs text-app-text-3 mb-1">Step Name</label>
          <input
            id="step-name"
            class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text focus:outline-none focus:border-app-border-2"
            value={step.name}
            oninput={(e) => onUpdate({ ...step, name: e.currentTarget.value })}
            placeholder="Step name"
          />
        </div>
        <div class="flex-1">
          <label for="request-picker" class="block text-xs text-app-text-3 mb-1">Request</label>
          <RequestPicker
            id="request-picker"
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
          onInputEdit={onInputEdit}
          onUpdate={(overrides) => onUpdate({ ...step, overrides })}
        />
      </div>

      <div>
        <SectionHeader class="mb-2">Extractions</SectionHeader>
        <ExtractionsPanel
          extractions={step.extract}
          {responseSchema}
          mode="scenario"
          onUpdate={(extract) => onUpdate({ ...step, extract })}
        />
      </div>

      <div>
        <SectionHeader class="mb-2">Expected Status</SectionHeader>
        <div class="flex flex-wrap gap-1 mb-2 min-h-[1.5rem]">
          {#if !step.expectedStatus?.length}
            <span class="text-xs text-app-text-4 italic">empty — any 2xx passes</span>
          {:else}
            {#each step.expectedStatus as code}
              <span class="font-mono text-xs bg-app-card border border-app-border-2 rounded px-1.5 py-0.5 flex items-center gap-1">
                {code}
                <button
                  type="button"
                  class="text-app-text-4 hover:text-red-400 leading-none"
                  onclick={() => removeStatus(code)}
                  aria-label="Remove {code}"
                >×</button>
              </span>
            {/each}
          {/if}
        </div>
        <div class="flex gap-2">
          <input
            class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text focus:outline-none focus:border-app-border-2"
            value={statusInput}
            oninput={(e) => (statusInput = e.currentTarget.value)}
            onkeydown={(e) => e.key === 'Enter' && addStatus()}
            placeholder="e.g. 200, 40*, 2**"
            maxlength="3"
          />
          <button
            type="button"
            class="px-3 py-1 text-xs bg-app-card border border-app-border-2 rounded text-app-text-2 hover:text-app-text transition-colors"
            onclick={addStatus}
          >Add</button>
        </div>
      </div>
    </div>
  {/if}
</div>
