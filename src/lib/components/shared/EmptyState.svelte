<script lang="ts">
  interface Props {
    message: string;
    description?: string;
    centered?: boolean;
  }

  let { message, description, centered = false }: Props = $props();
</script>

{#if centered}
  <div class="flex-1 flex items-center justify-center">
    <div class="text-center space-y-1">
      <p class="text-sm text-app-text-4">{message}</p>
      {#if description}
        <p class="text-xs text-app-text-4">{description}</p>
      {/if}
    </div>
  </div>
{:else}
  <p class="px-3 py-4 text-xs text-app-text-4">{message}</p>
{/if}
