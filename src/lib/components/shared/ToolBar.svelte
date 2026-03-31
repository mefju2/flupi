<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="flex items-center gap-3 px-4 py-2 border-b border-app-border shrink-0">
  {@render children()}
</div>
