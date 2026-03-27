<script lang="ts">
  import type { ScenarioStep, RequestTreeNode } from '$lib/services/tauri-commands';
  import { getRequest } from '$lib/services/tauri-commands';
  import { project } from '$lib/stores/project';
  import OverridesPanel from './OverridesPanel.svelte';
  import ExtractionsPanel from './ExtractionsPanel.svelte';

  interface Props {
    step: ScenarioStep;
    requestTree: RequestTreeNode[];
    index: number;
    onUpdate: (step: ScenarioStep) => void;
    onDelete: () => void;
  }

  let { step, requestTree, index, onUpdate, onDelete }: Props = $props();

  let expanded = $state(false);
  let requestSchema = $state<unknown>(null);
  let responseSchema = $state<unknown>(null);

  $effect(() => {
    const id = step.requestId;
    const path = $project?.path;
    if (!id || !path) return;
    getRequest(path, id).then((r) => {
      requestSchema = r.templateRef?.requestSchema ?? null;
      responseSchema = r.templateRef?.responseSchema ?? null;
    }).catch(() => {
      requestSchema = null;
      responseSchema = null;
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
  let methodColors: Record<string, string> = {
    GET: 'text-green-400', POST: 'text-cyan-400', PUT: 'text-yellow-400',
    PATCH: 'text-orange-400', DELETE: 'text-red-400',
  };
  let methodColor = $derived(requestInfo ? (methodColors[requestInfo.method] ?? 'text-zinc-400') : 'text-zinc-600');
</script>

<div class="border border-zinc-800 rounded bg-zinc-900 mb-2">
  <div class="flex items-center gap-2 px-3 py-2 cursor-pointer select-none" role="button" tabindex="0"
    onclick={() => expanded = !expanded}
    onkeydown={(e) => e.key === 'Enter' && (expanded = !expanded)}
  >
    <span class="drag-handle text-zinc-600 hover:text-zinc-400 cursor-grab active:cursor-grabbing text-xs shrink-0">⠿</span>
    <span class="text-xs text-zinc-500 w-5 shrink-0">{index + 1}</span>

    <div class="flex-1 flex items-center gap-2 min-w-0">
      <span class="text-sm text-zinc-200 truncate">{step.name || 'Unnamed Step'}</span>
      {#if !requestInfo}
        <span class="text-xs text-amber-400 bg-amber-950/40 border border-amber-800/60 rounded px-1.5 py-0.5 shrink-0">
          Request not found
        </span>
      {:else}
        <span class="font-mono text-xs {methodColor} shrink-0">{requestInfo.method}</span>
        <span class="font-mono text-xs text-zinc-500 truncate">{requestInfo.name}</span>
      {/if}
    </div>

    <div class="flex items-center gap-2 shrink-0">
      <button
        class="text-zinc-600 hover:text-red-400 transition-colors text-base"
        onclick={(e) => { e.stopPropagation(); onDelete(); }}
        aria-label="Delete step"
      >×</button>
      <span class="text-zinc-500 text-xs">{expanded ? '▾' : '▸'}</span>
    </div>
  </div>

  {#if expanded}
    <div class="border-t border-zinc-800 px-3 py-3 space-y-4">
      <div class="flex gap-3">
        <div class="flex-1">
          <label class="block text-xs text-zinc-500 mb-1">Step Name</label>
          <input
            class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 focus:outline-none focus:border-zinc-500"
            value={step.name}
            oninput={(e) => onUpdate({ ...step, name: e.currentTarget.value })}
            placeholder="Step name"
          />
        </div>
        <div class="flex-1">
          <label class="block text-xs text-zinc-500 mb-1">Request ID</label>
          <input
            class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono focus:outline-none focus:border-zinc-500"
            value={step.requestId}
            oninput={(e) => onUpdate({ ...step, requestId: e.currentTarget.value })}
            placeholder="request-id"
          />
        </div>
      </div>

      <div>
        <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Overrides</p>
        <OverridesPanel
          overrides={step.overrides}
          {requestSchema}
          onUpdate={(overrides) => onUpdate({ ...step, overrides })}
        />
      </div>

      <div>
        <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Extractions</p>
        <ExtractionsPanel
          extractions={step.extract}
          {responseSchema}
          onUpdate={(extract) => onUpdate({ ...step, extract })}
        />
      </div>
    </div>
  {/if}
</div>
