<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import type { RequestTreeNode } from '$lib/services/tauri-commands';
  import { getRequest } from '$lib/services/tauri-commands';
  import { requestTree, activeRequestId, activeRequest } from '$lib/stores/requests';
  import { project } from '$lib/stores/project';
  import { searchOpen } from '$lib/stores/ui';

  interface RequestOption {
    id: string;
    name: string;
    method: string;
    collectionPath: string;
  }

  const METHOD_COLORS: Record<string, string> = {
    GET: 'text-green-400',
    POST: 'text-cyan-400',
    PUT: 'text-yellow-400',
    PATCH: 'text-orange-400',
    DELETE: 'text-red-400',
  };

  let search = $state('');
  let activeIndex = $state(0);
  let searchEl = $state<HTMLInputElement | null>(null);

  function flattenTree(nodes: RequestTreeNode[], path: string): RequestOption[] {
    const result: RequestOption[] = [];
    for (const node of nodes) {
      if (node.type === 'Request') {
        result.push({ id: node.id, name: node.name, method: node.method, collectionPath: path });
      } else if (node.children) {
        const childPath = path ? `${path} / ${node.name}` : node.name;
        result.push(...flattenTree(node.children, childPath));
      }
    }
    return result;
  }

  const allRequests = $derived(flattenTree($requestTree, ''));

  const filtered = $derived(
    search.trim()
      ? allRequests.filter(
          (r) =>
            r.name.toLowerCase().includes(search.toLowerCase()) ||
            r.method.toLowerCase().includes(search.toLowerCase()) ||
            r.collectionPath.toLowerCase().includes(search.toLowerCase()),
        )
      : allRequests,
  );

  function close() {
    searchOpen.set(false);
    search = '';
    activeIndex = 0;
  }

  async function select(id: string) {
    const projectPath = $project?.path;
    if (!projectPath) return;
    activeRequestId.set(id);
    const data = await getRequest(projectPath, id);
    activeRequest.set(data);
    await goto('/requests');
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, filtered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, 0);
    } else if (e.key === 'Enter' && filtered[activeIndex]) {
      e.preventDefault();
      select(filtered[activeIndex].id);
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
    class="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-zinc-950/80"
    onclick={handleBackdropClick}
  >
    <div class="w-full max-w-[560px] mx-4 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl">
      <div class="p-3 border-b border-zinc-800">
        <input
          bind:this={searchEl}
          class="w-full bg-zinc-800 rounded px-3 py-2.5 text-base text-zinc-100 placeholder:text-zinc-500 focus:outline-none"
          placeholder="Search requests…"
          bind:value={search}
          oninput={() => (activeIndex = 0)}
          onkeydown={handleKeydown}
        />
      </div>

      <ul class="max-h-80 overflow-y-auto py-1">
        {#if filtered.length === 0}
          <li class="px-4 py-3 text-sm text-zinc-500 italic">No requests found</li>
        {:else}
          {#each filtered as req, idx}
            <li>
              <button
                type="button"
                class="w-full text-left px-4 py-2 flex items-center gap-3 {idx === activeIndex
                  ? 'bg-zinc-800'
                  : 'hover:bg-zinc-800'}"
                onclick={() => select(req.id)}
                onmouseenter={() => (activeIndex = idx)}
              >
                <span
                  class="font-mono text-xs {METHOD_COLORS[req.method] ??
                    'text-zinc-400'} w-16 shrink-0">{req.method}</span
                >
                <span class="text-sm text-zinc-100 truncate flex-1">{req.name}</span>
                {#if req.collectionPath}
                  <span class="text-xs text-zinc-500 shrink-0 truncate max-w-[160px]"
                    >{req.collectionPath}</span
                  >
                {/if}
              </button>
            </li>
          {/each}
        {/if}
      </ul>
    </div>
  </div>
{/if}
