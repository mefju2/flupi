<script lang="ts">
  import type { RequestTreeNode } from '$lib/services/tauri-commands';
  import { getMethodColor } from '$lib/utils/format';

  interface RequestOption {
    id: string;
    name: string;
    method: string;
    collectionPath: string;
  }

  interface Props {
    requestTree: RequestTreeNode[];
    value: string;
    onChange: (id: string) => void;
  }

  let { requestTree, value, onChange }: Props = $props();

  let open = $state(false);
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

  const allRequests = $derived(flattenTree(requestTree, ''));

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

  const selected = $derived(allRequests.find((r) => r.id === value) ?? null);

  function openPicker() {
    open = true;
    search = '';
    activeIndex = 0;
    setTimeout(() => searchEl?.focus(), 0);
  }

  function select(id: string) {
    onChange(id);
    open = false;
    search = '';
  }

  function handleSearchBlur() {
    setTimeout(() => { open = false; }, 150);
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
      open = false;
    }
  }
</script>

<div class="relative">
  <button
    type="button"
    class="w-full text-left bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm focus:outline-none focus:border-app-border-2 flex items-center gap-2 min-w-0"
    onclick={openPicker}
  >
    {#if selected}
      <span class="font-mono text-xs {getMethodColor(selected.method)} shrink-0">{selected.method}</span>
      <span class="text-app-text truncate">{selected.name}</span>
      {#if selected.collectionPath}
        <span class="text-app-text-4 text-xs shrink-0 truncate max-w-[100px]">{selected.collectionPath}</span>
      {/if}
    {:else if value}
      <span class="font-mono text-xs text-app-text-3 truncate">{value}</span>
    {:else}
      <span class="text-app-text-3">Select a request…</span>
    {/if}
    <span class="ml-auto text-app-text-4 text-xs shrink-0">▾</span>
  </button>

  {#if open}
    <div class="absolute top-full left-0 mt-0.5 z-50 w-full min-w-[280px] bg-app-panel border border-app-border-2 rounded shadow-lg">
      <div class="p-1.5 border-b border-app-border">
        <input
          bind:this={searchEl}
          class="w-full bg-app-card rounded px-2 py-1 text-sm text-app-text placeholder:text-app-text-4 focus:outline-none"
          placeholder="Search requests…"
          bind:value={search}
          oninput={() => (activeIndex = 0)}
          onkeydown={handleKeydown}
          onblur={handleSearchBlur}
        />
      </div>
      <ul class="max-h-52 overflow-y-auto py-1">
        {#if filtered.length === 0}
          <li class="px-3 py-2 text-sm text-app-text-3 italic">No requests found</li>
        {:else}
          {#each filtered as req, idx}
            <li>
              <button
                type="button"
                class="w-full text-left px-3 py-1.5 flex items-center gap-2 {idx === activeIndex ? 'bg-app-card' : 'hover:bg-app-card'}"
                onmousedown={() => select(req.id)}
              >
                <span class="font-mono text-xs {getMethodColor(req.method)} w-14 shrink-0">{req.method}</span>
                <span class="text-sm text-app-text truncate flex-1">{req.name}</span>
                {#if req.collectionPath}
                  <span class="text-xs text-app-text-4 shrink-0 truncate max-w-[120px]">{req.collectionPath}</span>
                {/if}
              </button>
            </li>
          {/each}
        {/if}
      </ul>
    </div>
  {/if}
</div>
