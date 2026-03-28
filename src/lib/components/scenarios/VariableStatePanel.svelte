<script lang="ts">
  interface Props {
    variables: Record<string, string>;
    secretKeys?: string[];
  }

  let { variables, secretKeys = [] }: Props = $props();

  let entries = $derived(Object.entries(variables));
</script>

<div class="space-y-0.5">
  {#if entries.length === 0}
    <p class="text-xs text-app-text-4 italic">No variables in context.</p>
  {:else}
    {#each entries as [key, value]}
      <div class="flex items-center gap-2 py-0.5">
        <span class="font-mono text-xs text-app-text-3 shrink-0 min-w-32">{key}</span>
        <span class="font-mono text-xs text-app-text truncate">
          {secretKeys.includes(key) ? '••••••' : value}
        </span>
      </div>
    {/each}
  {/if}
</div>
