<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    class?: string;
    children: Snippet;
  }

  let { class: cls = '', children }: Props = $props();
</script>

<p class="text-xs text-app-text-3 uppercase tracking-wider {cls}">
  {@render children()}
</p>
