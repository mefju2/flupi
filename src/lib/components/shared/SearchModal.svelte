<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import type { RequestTreeNode, ScenarioTreeNode } from '$lib/services/tauri-commands';
  import { getRequest, getScenario } from '$lib/services/tauri-commands';
  import { requestTree, activeRequestId, activeRequest } from '$lib/stores/requests';
  import { scenarioTree, activeScenarioId, activeScenario } from '$lib/stores/scenarios';
  import { functions, selectedFunctionName } from '$lib/stores/functions';
  import { project } from '$lib/stores/project';
  import { searchOpen } from '$lib/stores/ui';
  import { getMethodColor } from '$lib/utils/format';

  type RequestResult = { kind: 'request'; id: string; name: string; method: string; collectionPath: string };
  type ScenarioResult = { kind: 'scenario'; id: string; name: string; groupPath: string };
  type FunctionResult = { kind: 'function'; name: string };
  type SearchResult = RequestResult | ScenarioResult | FunctionResult;

  let search = $state('');
  let activeIndex = $state(0);
  let searchEl = $state<HTMLInputElement | null>(null);

  function flattenRequests(nodes: RequestTreeNode[], path: string): RequestResult[] {
    const result: RequestResult[] = [];
    for (const node of nodes) {
      if (node.type === 'Request') {
        result.push({ kind: 'request', id: node.id, name: node.name, method: node.method, collectionPath: path });
      } else if (node.children) {
        const childPath = path ? `${path} / ${node.name}` : node.name;
        result.push(...flattenRequests(node.children, childPath));
      }
    }
    return result;
  }

  function flattenScenarios(nodes: ScenarioTreeNode[], groupPath: string): ScenarioResult[] {
    const result: ScenarioResult[] = [];
    for (const node of nodes) {
      if (node.type === 'Scenario') {
        result.push({ kind: 'scenario', id: node.id, name: node.name, groupPath });
      } else if (node.type === 'Group') {
        result.push(...flattenScenarios(node.children, node.name));
      }
    }
    return result;
  }

  const allRequests = $derived(flattenRequests($requestTree, ''));
  const allScenarios = $derived(flattenScenarios($scenarioTree, ''));
  const allFunctions = $derived($functions.map((f): FunctionResult => ({ kind: 'function', name: f.name })));

  const q = $derived(search.trim().toLowerCase());

  const filteredRequests = $derived(
    q
      ? allRequests.filter(
          (r) =>
            r.name.toLowerCase().includes(q) ||
            r.method.toLowerCase().includes(q) ||
            r.collectionPath.toLowerCase().includes(q),
        )
      : allRequests,
  );

  const filteredScenarios = $derived(
    q
      ? allScenarios.filter(
          (s) => s.name.toLowerCase().includes(q) || s.groupPath.toLowerCase().includes(q),
        )
      : allScenarios,
  );

  const filteredFunctions = $derived(
    q ? allFunctions.filter((f) => f.name.toLowerCase().includes(q)) : allFunctions,
  );

  const allFiltered = $derived<SearchResult[]>([
    ...filteredRequests,
    ...filteredScenarios,
    ...filteredFunctions,
  ]);

  function close() {
    searchOpen.set(false);
    search = '';
    activeIndex = 0;
  }

  async function select(result: SearchResult) {
    const projectPath = $project?.path;
    if (!projectPath) return;
    if (result.kind === 'request') {
      activeRequestId.set(result.id);
      activeRequest.set(await getRequest(projectPath, result.id));
      await goto('/requests');
    } else if (result.kind === 'scenario') {
      activeScenarioId.set(result.id);
      activeScenario.set(await getScenario(projectPath, result.id));
      await goto('/scenarios');
    } else if (result.kind === 'function') {
      selectedFunctionName.set(result.name);
      await goto('/functions');
    }
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, allFiltered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, 0);
    } else if (e.key === 'Enter' && allFiltered[activeIndex]) {
      e.preventDefault();
      select(allFiltered[activeIndex]);
    } else if (e.key === 'Escape') {
      close();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) close();
  }

  $effect(() => {
    if ($searchOpen) {
      search = '';
      activeIndex = 0;
      setTimeout(() => searchEl?.focus(), 0);
    }
  });

  onMount(() => {
    function onGlobalKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape' && $searchOpen) close();
    }
    window.addEventListener('keydown', onGlobalKeydown);
    return () => window.removeEventListener('keydown', onGlobalKeydown);
  });
</script>

{#if $searchOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-app-bg/60 backdrop-blur-sm"
    onclick={handleBackdropClick}
  >
    <div class="w-full max-w-[560px] mx-4 bg-app-panel border border-app-border-2 rounded-lg shadow-xl">
      <div class="p-3 border-b border-app-border">
        <input
          bind:this={searchEl}
          class="w-full bg-app-card rounded px-3 py-2.5 text-base text-app-text placeholder:text-app-text-3 focus:outline-none"
          placeholder="Search everything…"
          bind:value={search}
          oninput={() => (activeIndex = 0)}
          onkeydown={handleKeydown}
        />
        <div class="flex gap-3 mt-2 px-1 text-xs text-app-text-4">
          <span>↑↓ Navigate</span>
          <span>↵ Select</span>
          <span>Esc Close</span>
        </div>
      </div>

      <ul class="max-h-80 overflow-y-auto py-1">
        {#if allFiltered.length === 0}
          <li class="px-4 py-3 text-sm text-app-text-3 italic">No results</li>
        {:else}
          {#if filteredRequests.length > 0}
            <li class="px-4 py-1.5 text-xs font-medium text-app-text-4 uppercase tracking-wide">Requests</li>
            {#each filteredRequests as req, i}
              <li>
                <button
                  type="button"
                  class="w-full text-left px-4 py-2 flex items-center gap-3 {i === activeIndex ? 'bg-app-card' : 'hover:bg-app-card'}"
                  onclick={() => select(req)}
                  onmouseenter={() => (activeIndex = i)}
                >
                  <span class="font-mono text-xs {getMethodColor(req.method)} w-16 shrink-0">{req.method}</span>
                  <span class="text-sm text-app-text truncate flex-1">{req.name}</span>
                  {#if req.collectionPath}
                    <span class="text-xs text-app-text-3 shrink-0 truncate max-w-[160px]">{req.collectionPath}</span>
                  {/if}
                </button>
              </li>
            {/each}
          {/if}

          {#if filteredScenarios.length > 0}
            <li class="px-4 py-1.5 text-xs font-medium text-app-text-4 uppercase tracking-wide {filteredRequests.length > 0 ? 'border-t border-app-border mt-1' : ''}">Scenarios</li>
            {#each filteredScenarios as scenario, i}
              {@const idx = filteredRequests.length + i}
              <li>
                <button
                  type="button"
                  class="w-full text-left px-4 py-2 flex items-center gap-3 {idx === activeIndex ? 'bg-app-card' : 'hover:bg-app-card'}"
                  onclick={() => select(scenario)}
                  onmouseenter={() => (activeIndex = idx)}
                >
                  <span class="text-sm text-app-text truncate flex-1">{scenario.name}</span>
                  {#if scenario.groupPath}
                    <span class="text-xs text-app-text-3 shrink-0 truncate max-w-[160px]">{scenario.groupPath}</span>
                  {/if}
                </button>
              </li>
            {/each}
          {/if}

          {#if filteredFunctions.length > 0}
            <li class="px-4 py-1.5 text-xs font-medium text-app-text-4 uppercase tracking-wide {filteredRequests.length > 0 || filteredScenarios.length > 0 ? 'border-t border-app-border mt-1' : ''}">Functions</li>
            {#each filteredFunctions as fn, i}
              {@const idx = filteredRequests.length + filteredScenarios.length + i}
              <li>
                <button
                  type="button"
                  class="w-full text-left px-4 py-2 flex items-center gap-3 {idx === activeIndex ? 'bg-app-card' : 'hover:bg-app-card'}"
                  onclick={() => select(fn)}
                  onmouseenter={() => (activeIndex = idx)}
                >
                  <span class="font-mono text-sm text-app-text truncate flex-1">{fn.name}</span>
                </button>
              </li>
            {/each}
          {/if}
        {/if}
      </ul>
    </div>
  </div>
{/if}
