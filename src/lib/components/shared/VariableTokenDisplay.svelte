<script lang="ts">
  interface TokenPart { type: 'token'; name: string; raw: string; found: boolean; }
  interface TextPart { type: 'text'; text: string; }
  type Part = TokenPart | TextPart;

  interface Props {
    value: string;
    vars: Record<string, string>;
    secrets: string[];
    onTokenHover: (varName: string, anchorEl: HTMLElement) => void;
    onTokenLeave: () => void;
    onclick: () => void;
  }

  let { value, vars, secrets: _secrets, onTokenHover, onTokenLeave, onclick }: Props = $props();

  const parsedParts: Part[] = $derived.by(() => {
    const parts: Part[] = [];
    let lastIndex = 0;
    const regex = /\{\{(\w+)\}\}/g;
    let match;
    while ((match = regex.exec(value)) !== null) {
      if (match.index > lastIndex) {
        parts.push({ type: 'text', text: value.slice(lastIndex, match.index) });
      }
      parts.push({ type: 'token', name: match[1], raw: match[0], found: match[1] in vars });
      lastIndex = match.index + match[0].length;
    }
    if (lastIndex < value.length) {
      parts.push({ type: 'text', text: value.slice(lastIndex) });
    }
    return parts;
  });
</script>

<div
  class="bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text cursor-text w-full min-h-[30px] flex items-center"
  role="textbox"
  tabindex="-1"
  aria-label="value with variable tokens"
  {onclick}
>
  {#if value === ''}
    <span class="text-app-text-4 text-sm"></span>
  {:else}
    {#each parsedParts as part}
      {#if part.type === 'token'}
        <span
          class="{part.found ? 'text-emerald-400' : 'text-red-400'}"
          onmouseenter={(e) => onTokenHover(part.name, e.currentTarget as HTMLElement)}
          onmouseleave={onTokenLeave}
        >{part.raw}</span>
      {:else}
        <span class="text-app-text">{part.text}</span>
      {/if}
    {/each}
  {/if}
</div>
