<script lang="ts">
  import { RefreshCw } from 'lucide-svelte';

  interface Props {
    filePath: string;
    oldContent: string;
    newContent: string;
    isLoading: boolean;
  }

  let { filePath, oldContent, newContent, isLoading }: Props = $props();

  type DiffLine = { type: 'same' | 'add' | 'remove'; text: string };

  function diffLines(a: string[], b: string[]): DiffLine[] {
    const m = a.length;
    const n = b.length;
    const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
    for (let i = 1; i <= m; i++)
      for (let j = 1; j <= n; j++)
        dp[i][j] = a[i - 1] === b[j - 1] ? dp[i - 1][j - 1] + 1 : Math.max(dp[i - 1][j], dp[i][j - 1]);

    const result: DiffLine[] = [];
    let i = m;
    let j = n;
    while (i > 0 || j > 0) {
      if (i > 0 && j > 0 && a[i - 1] === b[j - 1]) {
        result.push({ type: 'same', text: a[i - 1] });
        i--;
        j--;
      } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
        result.push({ type: 'add', text: b[j - 1] });
        j--;
      } else {
        result.push({ type: 'remove', text: a[i - 1] });
        i--;
      }
    }
    return result.reverse();
  }

  const diffOrNull = $derived.by(() => {
    const a = oldContent ? oldContent.split('\n') : [];
    const b = newContent ? newContent.split('\n') : [];
    if (Math.max(a.length, b.length) > 2000) return null;
    return diffLines(a, b);
  });

  const hasChanges = $derived(diffOrNull !== null && diffOrNull.some((l) => l.type !== 'same'));
  const fileName = $derived(filePath.split('/').at(-1) ?? filePath);
  const isNewFile = $derived(oldContent === '' && newContent !== '');
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center gap-3 px-4 py-2.5 border-b border-app-border bg-app-panel shrink-0">
    <span class="font-mono text-sm text-app-text font-medium">{fileName}</span>
    {#if filePath !== fileName}
      <span class="font-mono text-xs text-app-text-3 truncate">{filePath}</span>
    {/if}
    {#if isNewFile}
      <span class="ml-auto text-xs px-1.5 py-0.5 rounded bg-green-500/15 text-green-400 border border-green-500/25">
        new file
      </span>
    {/if}
  </div>

  {#if isLoading}
    <div class="flex items-center gap-2 p-4 text-sm text-app-text-3">
      <RefreshCw size={14} class="animate-spin" />
      Loading diff…
    </div>
  {:else if diffOrNull === null}
    <p class="p-4 text-sm text-app-text-3 italic">File too large to diff (>2000 lines).</p>
  {:else if diffOrNull.length === 0}
    <p class="p-4 text-sm text-app-text-3 italic">Empty file.</p>
  {:else if !hasChanges}
    <p class="p-4 text-sm text-app-text-3 italic">No changes detected.</p>
  {:else}
    <div class="flex-1 overflow-auto">
      <pre class="text-xs leading-5 font-mono p-4 whitespace-pre">{#each diffOrNull as line}<span
            class={line.type === 'add'
              ? 'block bg-green-950/40 text-green-300'
              : line.type === 'remove'
                ? 'block bg-red-950/40 text-red-400 line-through decoration-red-800/50'
                : 'block text-app-text-3'}
          >{line.type === 'add' ? '+ ' : line.type === 'remove' ? '- ' : '  '}{line.text}</span>{/each}</pre>
    </div>
  {/if}
</div>
