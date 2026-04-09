<script lang="ts" module>
  export interface GitFileEntry {
    path: string;
    status: import('$lib/stores/git').GitFileStatus;
  }
</script>

<script lang="ts">
  import { ChevronRight, ChevronDown, Plus, Minus } from 'lucide-svelte';
  import type { GitFileStatus } from '$lib/stores/git';

  interface Props {
    files: GitFileEntry[];
    selectedPath?: string | null;
    onselect: (path: string, status: GitFileStatus) => void;
    onaction: (path: string, status: GitFileStatus) => void;
  }

  let { files, selectedPath = null, onselect, onaction }: Props = $props();

  type TreeNode = {
    name: string;
    path: string;
    status: GitFileStatus | null;
    children: Map<string, TreeNode>;
    isFile: boolean;
  };

  function buildTree(entries: GitFileEntry[]): Map<string, TreeNode> {
    const root = new Map<string, TreeNode>();
    for (const entry of entries) {
      const parts = entry.path.split('/');
      let current = root;
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const isFile = i === parts.length - 1;
        const nodePath = parts.slice(0, i + 1).join('/');
        if (!current.has(part)) {
          current.set(part, {
            name: part,
            path: nodePath,
            status: isFile ? entry.status : null,
            children: new Map(),
            isFile,
          });
        }
        current = current.get(part)!.children;
      }
    }
    return root;
  }

  let openDirs = $state<Set<string>>(new Set());

  $effect(() => {
    for (const entry of files) {
      const parts = entry.path.split('/');
      for (let i = 0; i < parts.length - 1; i++) {
        openDirs.add(parts.slice(0, i + 1).join('/'));
      }
    }
  });

  const tree = $derived(buildTree(files));

  function dotColor(status: GitFileStatus): string {
    switch (status) {
      case 'staged':
        return 'text-green-400';
      case 'modified':
        return 'text-yellow-400';
      case 'deleted':
        return 'text-red-400';
      case 'untracked':
        return 'text-purple-400';
    }
  }

  function toggleDir(path: string) {
    const next = new Set(openDirs);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    openDirs = next;
  }
</script>

{#snippet renderNode(node: TreeNode, depth: number)}
  {#if node.isFile}
    <div
      class="group flex items-center gap-1 rounded px-1 py-0.5 cursor-pointer text-xs
             {node.path === selectedPath
               ? 'bg-app-card text-app-text'
               : 'text-app-text-2 hover:bg-app-card/50'}"
      style="padding-left: {depth * 12 + 4}px"
      role="button"
      tabindex="0"
      onclick={() => onselect(node.path, node.status!)}
      onkeydown={(e) => e.key === 'Enter' && onselect(node.path, node.status!)}
    >
      <span class="w-1.5 h-1.5 rounded-full shrink-0 {dotColor(node.status!)} bg-current"></span>
      <span class="font-mono truncate flex-1">{node.name}</span>
      <button
        class="shrink-0 opacity-0 group-hover:opacity-100 p-0.5 rounded
               hover:bg-app-border transition-all"
        onclick={(e) => {
          e.stopPropagation();
          onaction(node.path, node.status!);
        }}
        title={node.status === 'staged' ? 'Unstage' : 'Stage'}
        aria-label={node.status === 'staged' ? 'Unstage file' : 'Stage file'}
      >
        {#if node.status === 'staged'}
          <Minus size={10} />
        {:else}
          <Plus size={10} />
        {/if}
      </button>
    </div>
  {:else}
    {@const isOpen = openDirs.has(node.path)}
    <button
      class="flex items-center gap-1 w-full text-left px-1 py-0.5 text-xs
             text-app-text-3 hover:text-app-text-2"
      style="padding-left: {depth * 12 + 4}px"
      onclick={() => toggleDir(node.path)}
    >
      {#if isOpen}<ChevronDown size={10} />{:else}<ChevronRight size={10} />{/if}
      <span class="font-mono">{node.name}</span>
    </button>
    {#if isOpen}
      {#each [...node.children.values()] as child}
        {@render renderNode(child, depth + 1)}
      {/each}
    {/if}
  {/if}
{/snippet}

<div class="flex flex-col">
  {#each [...tree.values()] as node}
    {@render renderNode(node, 0)}
  {/each}
</div>
