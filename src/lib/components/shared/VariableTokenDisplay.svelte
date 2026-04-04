<script lang="ts">
  // Matches both {{varName}} and {{$fnName(...)}} tokens
  const TOKEN_REGEX = /\{\{(\$[a-zA-Z_$][a-zA-Z0-9_$]*\([^)]*\)|[\w.-]+)\}\}/g;

  interface TokenPart { type: 'token'; name: string; raw: string; found: boolean; isFunction: boolean; }
  interface TextPart { type: 'text'; text: string; }
  type Part = TokenPart | TextPart;

  interface Props {
    value: string;
    vars: Set<string>;
    secrets: string[];
    fnNames?: Set<string>;
    placeholder?: string;
    multiline?: boolean;
    onTokenHover: (varName: string, anchorEl: HTMLElement) => void;
    onTokenLeave?: () => void;
    onFunctionHover?: (fnName: string, anchorEl: HTMLElement) => void;
    onFunctionLeave?: () => void;
    onclick: () => void;
  }

  let { value, vars, secrets, fnNames = new Set(), placeholder = '', multiline = false, onTokenHover, onTokenLeave, onFunctionHover, onFunctionLeave, onclick }: Props = $props();

  const parsedParts: Part[] = $derived.by(() => {
    const parts: Part[] = [];
    let lastIndex = 0;
    TOKEN_REGEX.lastIndex = 0;
    let match;
    while ((match = TOKEN_REGEX.exec(value)) !== null) {
      if (match.index > lastIndex) {
        parts.push({ type: 'text', text: value.slice(lastIndex, match.index) });
      }
      const inner = match[1];
      const isFunction = inner.startsWith('$');
      let found: boolean;
      if (isFunction) {
        const fnName = inner.slice(1, inner.indexOf('('));
        found = fnNames.has(fnName);
      } else {
        found = vars.has(inner) || secrets.includes(inner);
      }
      parts.push({ type: 'token', name: inner, raw: match[0], found, isFunction });
      lastIndex = match.index + match[0].length;
    }
    if (lastIndex < value.length) {
      parts.push({ type: 'text', text: value.slice(lastIndex) });
    }
    return parts;
  });
</script>

<div
  class="bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text cursor-text w-full
    {multiline ? 'min-h-15 whitespace-pre-wrap break-all flex flex-wrap items-baseline content-start' : 'min-h-7.5 flex items-center overflow-hidden whitespace-nowrap'}"
  role="textbox"
  aria-readonly="true"
  aria-multiline={multiline ? 'true' : 'false'}
  tabindex="-1"
  aria-label="value with variable tokens"
  {onclick}
  onkeydown={(e) => e.key === 'Enter' && onclick()}
>
  {#if value === '' && placeholder !== ''}
    <span class="text-app-text-4">{placeholder}</span>
  {:else}
    {#each parsedParts as part}
      {#if part.type === 'token'}
        <span
          role="img"
          aria-label={part.name}
          title={part.isFunction ? (part.found ? undefined : 'Not found — create it on the Functions page') : undefined}
          class="{part.isFunction ? (part.found ? 'text-cyan-400' : 'text-yellow-400') : (part.found ? 'text-green-400' : 'text-red-400')}"
          onmouseenter={part.isFunction
            ? (part.found ? (e) => {
                const fnName = part.name.slice(1, part.name.indexOf('('));
                onFunctionHover?.(fnName, e.currentTarget as HTMLElement);
              } : undefined)
            : (e) => onTokenHover(part.name, e.currentTarget as HTMLElement)}
          onmouseleave={part.isFunction ? () => onFunctionLeave?.() : () => onTokenLeave?.()}
        >{part.raw}</span>
      {:else}
        <span class="text-app-text">{part.text}</span>
      {/if}
    {/each}
  {/if}
</div>
