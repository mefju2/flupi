<script lang="ts">
  import { fade } from 'svelte/transition';

  interface Props {
    visible: boolean;
    message?: string;
    class?: string;
  }

  let { visible, message = 'Saved', class: cls = '' }: Props = $props();
</script>

{#if visible}
  <span class="text-xs text-green-400 {cls}" transition:fade={{ duration: 150 }}>{message}</span>
{/if}
