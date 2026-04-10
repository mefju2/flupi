<script lang="ts">
  import { RefreshCw } from "lucide-svelte";
  import type { DiffLine } from "$lib/services/tauri-commands";

  interface Props {
    filePath: string;
    lines: DiffLine[];
    isNewFile: boolean;
    isLoading: boolean;
  }

  let { filePath, lines, isNewFile, isLoading }: Props = $props();

  type DiffHunk = { lines: DiffLine[] } | { collapsed: number };

  // Build context-diff hunks: only emit changed lines + CTX surrounding lines.
  // Collapses long unchanged stretches into a single placeholder to keep DOM small.
  const CTX = 3;
  function buildHunks(lines: DiffLine[]): DiffHunk[] {
    const show = new Uint8Array(lines.length);
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].type !== "same") {
        const lo = Math.max(0, i - CTX);
        const hi = Math.min(lines.length - 1, i + CTX);
        for (let j = lo; j <= hi; j++) show[j] = 1;
      }
    }
    const hunks: DiffHunk[] = [];
    let i = 0;
    while (i < lines.length) {
      if (show[i]) {
        const start = i;
        while (i < lines.length && show[i]) i++;
        hunks.push({ lines: lines.slice(start, i) });
      } else {
        const start = i;
        while (i < lines.length && !show[i]) i++;
        hunks.push({ collapsed: i - start });
      }
    }
    return hunks;
  }

  const hasChanges = $derived(lines.some((l) => l.type !== "same"));

  type RenderLine = DiffLine | { type: "collapsed"; count: number };

  // Flatten hunks into a render-ready array: changed/context lines stay as DiffLine,
  // collapsed unchanged runs become a single placeholder — minimizing DOM nodes.
  const renderLines = $derived.by((): RenderLine[] | null => {
    if (!hasChanges) return null;
    const result: RenderLine[] = [];
    for (const hunk of buildHunks(lines)) {
      if ("collapsed" in hunk) {
        result.push({ type: "collapsed", count: hunk.collapsed });
      } else {
        for (const line of hunk.lines) result.push(line);
      }
    }
    return result;
  });

  const fileName = $derived(filePath.split("/").at(-1) ?? filePath);
</script>

<div class="flex flex-col h-full">
  <div
    class="flex items-center gap-3 px-4 py-2.5 border-b border-app-border bg-app-panel shrink-0"
  >
    <span class="font-mono text-sm text-app-text font-medium">{fileName}</span>
    {#if filePath !== fileName}
      <span class="font-mono text-xs text-app-text-3 truncate">{filePath}</span>
    {/if}
    {#if isNewFile}
      <span
        class="ml-auto text-xs px-1.5 py-0.5 rounded bg-green-500/15 text-green-400 border border-green-500/25"
      >
        new file
      </span>
    {/if}
  </div>

  {#if isLoading}
    <div class="flex items-center gap-2 p-4 text-sm text-app-text-3">
      <RefreshCw size={14} class="animate-spin" />
      Loading diff…
    </div>
  {:else if lines.length === 0}
    <p class="p-4 text-sm text-app-text-3 italic">Empty file.</p>
  {:else if !hasChanges}
    <p class="p-4 text-sm text-app-text-3 italic">No changes detected.</p>
  {:else}
    <div class="flex-1 overflow-auto">
      <pre
        class="text-xs leading-5 font-mono p-4 whitespace-pre">{#each renderLines! as line}{#if line.type === "collapsed"}<span
              class="block text-app-text-3/50 italic select-none"
              >···  {line.count} unchanged line{line.count === 1
                ? ""
                : "s"}  ···</span
            >{:else}<span
              class={line.type === "add"
                ? "block bg-green-500/10 text-green-700 dark:bg-green-950/40 dark:text-green-300"
                : line.type === "remove"
                  ? "block bg-red-500/10 text-red-700 line-through decoration-red-400/50 dark:bg-red-950/40 dark:text-red-400 dark:decoration-red-800/50"
                  : "block text-app-text-3"}
              >{line.type === "add"
                ? "+ "
                : line.type === "remove"
                  ? "- "
                  : "  "}{line.text}</span
            >{/if}{/each}</pre>
    </div>
  {/if}
</div>
