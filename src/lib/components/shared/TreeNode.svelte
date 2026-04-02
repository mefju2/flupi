<script lang="ts">
  import type { RequestTreeNode } from '$lib/services/tauri-commands';
  import TreeNode from './TreeNode.svelte';
  import { getMethodColor } from '$lib/utils/format';
  import { driftedRequestIds } from '$lib/stores/openapi';

  interface InlineEdit {
    value: string;
    onChange: (v: string) => void;
    onConfirm: () => void;
    onCancel: () => void;
  }

  interface Props {
    node: RequestTreeNode;
    activeRequestId: string | null;
    activeCollectionFolder?: string | null;
    onSelect: (id: string) => void;
    onSelectCollection?: (folderName: string) => void;
    onContextMenu: (e: MouseEvent, node: RequestTreeNode) => void;
    showDragHandle?: boolean;
    inlineEdit?: InlineEdit | null;
    expanded?: boolean;
    onToggleExpanded?: () => void;
  }

  let { node, activeRequestId, activeCollectionFolder = null, onSelect, onSelectCollection, onContextMenu, showDragHandle = false, inlineEdit = null, expanded = node.type === 'Collection', onToggleExpanded }: Props = $props();

  function focusAndSelect(el: HTMLElement) {
    el.focus();
    if (el instanceof HTMLInputElement) el.select();
  }
</script>

{#if node.type === 'Collection' || node.type === 'Folder'}
  {@const isActiveCollection = node.type === 'Collection' && activeCollectionFolder === node.folder_name}
  <div>
    <div
      class="flex items-center gap-1.5 px-2 py-1 text-sm select-none rounded
        {isActiveCollection ? 'bg-app-card text-app-text' : 'text-app-text-2 hover:bg-app-card/50 hover:text-app-text'}"
      oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e, node); }}
    >
      <button
        class="text-app-text-3 text-xs shrink-0 focus:outline-none cursor-pointer hover:text-app-text-2 w-3"
        tabindex="0"
        onclick={(e) => { e.stopPropagation(); onToggleExpanded?.(); }}
        onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); onToggleExpanded?.(); } }}
        aria-label={expanded ? 'Collapse' : 'Expand'}
      >{expanded ? '▾' : '▸'}</button>
      <span
        class="flex items-center gap-1.5 flex-1 min-w-0 cursor-pointer"
        role="button"
        tabindex="0"
        onclick={() => !inlineEdit && (node.type === 'Collection' ? onSelectCollection?.(node.folder_name) : onToggleExpanded?.())}
        onkeydown={(e) => { if (e.key === 'Enter' && !inlineEdit) node.type === 'Collection' ? onSelectCollection?.(node.folder_name) : onToggleExpanded?.(); }}
      >
        <span class="text-app-text-3 text-xs shrink-0">📁</span>
        {#if inlineEdit}
          <input
            class="flex-1 min-w-0 bg-app-card border border-cyan-600 rounded px-1 py-0 text-sm text-app-text font-mono focus:outline-none"
            value={inlineEdit.value}
            oninput={(e) => inlineEdit!.onChange((e.target as HTMLInputElement).value)}
            onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); inlineEdit!.onConfirm(); } if (e.key === 'Escape') { e.stopPropagation(); inlineEdit!.onCancel(); } }}
            onblur={() => inlineEdit?.onConfirm()}
            onclick={(e) => e.stopPropagation()}
            use:focusAndSelect
          />
        {:else}
          <span class="truncate">{node.name}</span>
        {/if}
      </span>
    </div>

    {#if expanded}
      <div class="ml-3 border-l border-app-border pl-1">
        {#each node.children as child}
          <TreeNode
            node={child}
            {activeRequestId}
            {activeCollectionFolder}
            {onSelect}
            {onSelectCollection}
            {onContextMenu}
          />
        {/each}
      </div>
    {/if}
  </div>
{:else if node.type === 'Request'}
  {@const isActive = activeRequestId === node.id}
  <div
    class="group flex items-center gap-1.5 px-2 py-1 text-sm cursor-pointer select-none rounded
      {isActive ? 'bg-app-card text-app-text' : 'text-app-text-2 hover:bg-app-card/50 hover:text-app-text'}"
    role="button"
    tabindex="0"
    onclick={() => !inlineEdit && onSelect(node.id)}
    onkeydown={(e) => e.key === 'Enter' && !inlineEdit && onSelect(node.id)}
    oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e, node); }}
  >
    {#if showDragHandle}
      <span
        class="drag-handle text-app-text-4 group-hover:text-app-text-3 cursor-grab active:cursor-grabbing text-xs shrink-0 select-none"
        title="Drag to reorder"
      >⠿</span>
    {/if}
    <span class="font-mono text-xs w-12 shrink-0 {getMethodColor(node.method)}">{node.method}</span>
    {#if inlineEdit}
      <input
        class="flex-1 min-w-0 bg-app-card border border-cyan-600 rounded px-1 py-0 text-sm text-app-text font-mono focus:outline-none"
        value={inlineEdit.value}
        oninput={(e) => inlineEdit!.onChange((e.target as HTMLInputElement).value)}
        onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); inlineEdit!.onConfirm(); } if (e.key === 'Escape') { e.stopPropagation(); inlineEdit!.onCancel(); } }}
        onblur={() => inlineEdit?.onConfirm()}
        onclick={(e) => e.stopPropagation()}
        use:focusAndSelect
      />
    {:else}
      <span class="truncate" title={node.name}>{node.name}</span>
    {/if}
    {#if $driftedRequestIds.has(node.id)}
      <span class="shrink-0 w-2 h-2 rounded-full bg-red-500" title="Schema drift detected"></span>
    {/if}
  </div>
{/if}
