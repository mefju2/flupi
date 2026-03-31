<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'success' | 'error' | 'info' | 'neutral';

  interface Props {
    variant?: Variant;
    mono?: boolean;
    class?: string;
    children: Snippet;
  }

  let { variant = 'neutral', mono = true, class: cls = '', children }: Props = $props();

  const variantClasses: Record<Variant, string> = {
    success: 'text-green-400 bg-green-950/40',
    error: 'text-red-400 bg-red-950/40',
    info: 'text-cyan-400 bg-cyan-950/40 border border-cyan-800/60',
    neutral: 'text-app-text-3 bg-app-card',
  };
</script>

<span class="text-xs px-1.5 py-0.5 rounded {mono ? 'font-mono' : ''} {variantClasses[variant]} {cls}">
  {@render children()}
</span>
