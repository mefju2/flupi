<script lang="ts">
  import type { RequestTreeNode } from '$lib/services/tauri-commands';
  import TreeNode from './TreeNode.svelte';
  import { getMethodColor } from '$lib/utils/format';
  import { driftedRequestIds } from '$lib/stores/openapi';

  interface Props {
    node: RequestTreeNode;
    activeRequestId: string | null;
    onSelect: (id: string) => void;
    onContextMenu: (e: MouseEvent, node: RequestTreeNode) => void;
    showDragHandle?: boolean;
  }

  let { node, activeRequestId, onSelect, onContextMenu, showDragHandle = false }: Props = $props();

  // Collections start expanded, Folders start collapsed
  let expanded = $state(node.type === 'Collection');

</script>

{#if node.type === 'Collection' || node.type === 'Folder'}
  <div>
    <div
      class="flex items-center gap-1.5 px-2 py-1 text-sm cursor-pointer select-none text-zinc-300 hover:bg-zinc-800/50 hover:text-zinc-100 rounded"
      role="button"
      tabindex="0"
      onclick={() => (expanded = !expanded)}
      onkeydown={(e) => e.key === 'Enter' && (expanded = !expanded)}
      oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e, node); }}
    >
      <span class="text-zinc-500 text-xs">{expanded ? '▾' : '▸'}</span>
      <span class="text-zinc-400 text-xs">📁</span>
      <span class="truncate">{node.name}</span>
    </div>

    {#if expanded}
      <div class="ml-3 border-l border-zinc-800 pl-1">
        {#each node.children as child}
          <TreeNode
            node={child}
            {activeRequestId}
            {onSelect}
            {onContextMenu}
          />
        {/each}
        {#if node.children.length === 0}
          <p class="px-2 py-1 text-xs text-zinc-600 italic">Empty</p>
        {/if}
      </div>
    {/if}
  </div>
{:else if node.type === 'Request'}
  {@const isActive = activeRequestId === node.id}
  <div
    class="group flex items-center gap-1.5 px-2 py-1 text-sm cursor-pointer select-none rounded
      {isActive ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-300 hover:bg-zinc-800/50 hover:text-zinc-100'}"
    role="button"
    tabindex="0"
    onclick={() => onSelect(node.id)}
    onkeydown={(e) => e.key === 'Enter' && onSelect(node.id)}
    oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e, node); }}
  >
    {#if showDragHandle}
      <span
        class="drag-handle text-zinc-600 group-hover:text-zinc-400 cursor-grab active:cursor-grabbing text-xs shrink-0 select-none"
        title="Drag to reorder"
      >⠿</span>
    {/if}
    <span class="font-mono text-xs w-12 shrink-0 {getMethodColor(node.method)}">{node.method}</span>
    <span class="truncate">{node.name}</span>
    {#if $driftedRequestIds.has(node.id)}
      <span class="shrink-0 w-2 h-2 rounded-full bg-red-500" title="Schema drift detected"></span>
    {/if}
  </div>
{/if}
