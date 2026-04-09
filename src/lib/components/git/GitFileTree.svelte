<script lang="ts">
  import { ChevronDown, ChevronRight } from "lucide-svelte";

  interface Props {
    files: string[];
    kind: "modified" | "deleted" | "untracked";
    selectedPath?: string | null;
    onselect: (
      path: string,
      kind: "modified" | "deleted" | "untracked",
    ) => void;
  }

  let { files, kind, selectedPath = null, onselect }: Props = $props();

  interface FileNode {
    name: string;
    fullPath: string;
    children: FileNode[];
    isDir: boolean;
  }

  function buildTree(paths: string[]): FileNode[] {
    const root: FileNode[] = [];
    const dirMap = new Map<string, FileNode>();
    for (const p of paths) {
      const parts = p.split("/");
      let current = root;
      let cumPath = "";
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        cumPath = cumPath ? `${cumPath}/${part}` : part;
        if (i === parts.length - 1) {
          current.push({ name: part, fullPath: p, children: [], isDir: false });
        } else {
          let dir = dirMap.get(cumPath);
          if (!dir) {
            dir = { name: part, fullPath: cumPath, children: [], isDir: true };
            dirMap.set(cumPath, dir);
            current.push(dir);
          }
          current = dir.children;
        }
      }
    }
    return root;
  }

  function collectAllDirPaths(nodes: FileNode[]): string[] {
    const dirs: string[] = [];
    function walk(ns: FileNode[]) {
      for (const n of ns) {
        if (n.isDir) {
          dirs.push(n.fullPath);
          walk(n.children);
        }
      }
    }
    walk(nodes);
    return dirs;
  }

  const tree = $derived(buildTree(files));
  let openDirs = $state(new Set<string>());
  let userClosedDirs = $state(new Set<string>());

  $effect(() => {
    // When tree changes (auto-refresh), expand new dirs unless user explicitly closed them
    openDirs = new Set(
      collectAllDirPaths(tree).filter((d) => !userClosedDirs.has(d)),
    );
  });

  function toggle(path: string) {
    const next = new Set(openDirs);
    if (next.has(path)) {
      next.delete(path);
      userClosedDirs = new Set([...userClosedDirs, path]);
    } else {
      next.add(path);
      const c = new Set(userClosedDirs);
      c.delete(path);
      userClosedDirs = c;
    }
    openDirs = next;
  }

  const dotClass = $derived(
    kind === "deleted"
      ? "bg-red-400"
      : kind === "modified"
        ? "bg-yellow-400"
        : "bg-green-400",
  );
</script>

{#snippet renderNode(node: FileNode, depth: number)}
  {#if node.isDir}
    <button
      class="flex items-center gap-1 w-full text-left text-xs text-app-text-3 hover:text-app-text py-0.5 transition-colors"
      style:padding-left="{depth * 12 + 4}px"
      onclick={() => toggle(node.fullPath)}
    >
      {#if openDirs.has(node.fullPath)}
        <ChevronDown size={12} class="shrink-0" />
      {:else}
        <ChevronRight size={12} class="shrink-0" />
      {/if}
      <span class="font-mono">{node.name}/</span>
    </button>
    {#if openDirs.has(node.fullPath)}
      {#each node.children as child}
        {@render renderNode(child, depth + 1)}
      {/each}
    {/if}
  {:else}
    <button
      class="flex items-center gap-2 w-full text-left text-xs py-0.5 rounded transition-colors
             {selectedPath === node.fullPath
        ? 'bg-app-card text-app-text'
        : 'text-app-text-2 hover:bg-app-card/50 hover:text-app-text'}"
      style:padding-left="{depth * 12 + 4}px"
      onclick={() => onselect(node.fullPath, kind)}
    >
      <span class="w-1.5 h-1.5 rounded-full shrink-0 {dotClass}"></span>
      <span class="font-mono truncate">{node.name}</span>
    </button>
  {/if}
{/snippet}

<div class="flex flex-col">
  {#each tree as node}
    {@render renderNode(node, 0)}
  {/each}
</div>
