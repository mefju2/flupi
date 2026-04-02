<script lang="ts">
  interface Props {
    label: string;
    oldSchema: unknown;
    newSchema: unknown;
  }

  let { label, oldSchema, newSchema }: Props = $props();

  type DiffLine = { type: 'same' | 'add' | 'remove'; text: string };

  function toLines(schema: unknown): string[] {
    if (schema === null || schema === undefined) return [];
    return JSON.stringify(schema, null, 2).split('\n');
  }

  /** Myers-style LCS diff — returns a flat DiffLine array. */
  function diffLines(a: string[], b: string[]): DiffLine[] {
    const m = a.length;
    const n = b.length;

    // Build LCS length table.
    const table: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
    for (let i = 1; i <= m; i++) {
      for (let j = 1; j <= n; j++) {
        table[i][j] = a[i - 1] === b[j - 1]
          ? table[i - 1][j - 1] + 1
          : Math.max(table[i - 1][j], table[i][j - 1]);
      }
    }

    // Backtrack.
    const result: DiffLine[] = [];
    let i = m;
    let j = n;
    while (i > 0 || j > 0) {
      if (i > 0 && j > 0 && a[i - 1] === b[j - 1]) {
        result.push({ type: 'same', text: a[i - 1] });
        i--; j--;
      } else if (j > 0 && (i === 0 || table[i][j - 1] >= table[i - 1][j])) {
        result.push({ type: 'add', text: b[j - 1] });
        j--;
      } else {
        result.push({ type: 'remove', text: a[i - 1] });
        i--;
      }
    }
    return result.reverse();
  }

  const oldLines = $derived(toLines(oldSchema));
  const newLines = $derived(toLines(newSchema));

  const bothEmpty = $derived(oldLines.length === 0 && newLines.length === 0);
  const unchanged = $derived(
    !bothEmpty && JSON.stringify(oldSchema) === JSON.stringify(newSchema)
  );

  const diff = $derived(bothEmpty || unchanged ? [] : diffLines(oldLines, newLines));

  const hasChanges = $derived(diff.some((l) => l.type !== 'same'));
</script>

<div class="flex flex-col gap-1.5">
  <p class="text-xs font-semibold text-app-text-2 uppercase tracking-wider">{label}</p>

  {#if bothEmpty}
    <p class="text-xs text-app-text-4 italic">No schema</p>
  {:else if unchanged}
    <p class="text-xs text-app-text-4 italic">Unchanged</p>
  {:else if !hasChanges}
    <p class="text-xs text-app-text-4 italic">Unchanged</p>
  {:else}
    <div class="rounded-md border border-app-border bg-app-bg overflow-x-auto">
      <pre class="text-xs leading-5 font-mono p-3 whitespace-pre">{#each diff as line}<span
          class={line.type === 'add'
            ? 'block bg-green-950/40 text-green-300'
            : line.type === 'remove'
              ? 'block bg-red-950/40 text-red-400 line-through decoration-red-800/50'
              : 'block text-app-text-3'}
        >{line.type === 'add' ? '+ ' : line.type === 'remove' ? '- ' : '  '}{line.text}</span>{/each}</pre>
    </div>
  {/if}
</div>
