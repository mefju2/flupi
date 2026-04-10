<script lang="ts">
  import { Eye, EyeOff } from "lucide-svelte";

  interface Props {
    value: string;
    class?: string;
    onchange?: (value: string) => void;
  }

  let { value, class: className = "", onchange }: Props = $props();

  let revealed = $state(!value);
</script>

{#if onchange}
  <div class="flex items-center gap-1 {className}">
    {#if revealed}
      <input
        class="flex-1 min-w-0 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-hover"
        type="text"
        {value}
        oninput={(e) => onchange(e.currentTarget.value)}
        placeholder="Value"
      />
    {:else}
      <div
        class="flex-1 min-w-0 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono overflow-hidden text-ellipsis tracking-widest"
      >
        ••••••••
      </div>
    {/if}
    <button
      type="button"
      class="text-app-text-4 hover:text-app-text-2 transition-colors shrink-0"
      onclick={() => (revealed = !revealed)}
      aria-label={revealed ? "Hide secret" : "Reveal secret"}
    >
      {#if revealed}
        <EyeOff size={14} />
      {:else}
        <Eye size={14} />
      {/if}
    </button>
  </div>
{:else}
  <span class="inline-flex items-center gap-1 {className}">
    <span class="font-mono tracking-widest break-all overflow-hidden"
      >{revealed || !value ? value : "••••••••"}</span
    >
    <button
      type="button"
      class="text-app-text-4 hover:text-app-text-2 transition-colors shrink-0"
      onclick={() => (revealed = !revealed)}
      aria-label={revealed ? "Hide secret" : "Reveal secret"}
    >
      {#if revealed}
        <EyeOff size={12} />
      {:else}
        <Eye size={12} />
      {/if}
    </button>
  </span>
{/if}
