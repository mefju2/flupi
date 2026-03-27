<script lang="ts">
  import type { SchemaSuggestion } from '$lib/utils/schema-paths';

  interface Props {
    suggestions: SchemaSuggestion[];
    value: string;
    placeholder?: string;
    onSelect: (path: string) => void;
    onInput: (value: string) => void;
    inputClass?: string;
  }

  let {
    suggestions,
    value,
    placeholder = '',
    onSelect,
    onInput,
    inputClass = '',
  }: Props = $props();

  let open = $state(false);
  let activeIndex = $state(-1);

  let filtered = $derived(
    suggestions
      .filter((s) => !value || s.path.toLowerCase().includes(value.toLowerCase()))
      .slice(0, 10),
  );

  function handleFocus() {
    open = true;
    activeIndex = -1;
  }

  function handleBlur() {
    setTimeout(() => {
      open = false;
      activeIndex = -1;
    }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, filtered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, -1);
    } else if (e.key === 'Enter' && activeIndex >= 0) {
      e.preventDefault();
      onSelect(filtered[activeIndex].path);
      open = false;
    } else if (e.key === 'Escape') {
      open = false;
    }
  }

  function handleInput(e: Event) {
    onInput((e.currentTarget as HTMLInputElement).value);
    open = true;
    activeIndex = -1;
  }
</script>

<div class="relative">
  <input
    class={inputClass}
    {value}
    {placeholder}
    onfocus={handleFocus}
    onblur={handleBlur}
    onkeydown={handleKeydown}
    oninput={handleInput}
    autocomplete="off"
    spellcheck={false}
  />

  {#if open && filtered.length > 0}
    <ul
      class="absolute left-0 top-full mt-0.5 z-50 bg-zinc-900 border border-zinc-700 rounded shadow-lg max-h-52 overflow-y-auto min-w-full"
      role="listbox"
    >
      {#each filtered as suggestion, idx}
        <li
          role="option"
          aria-selected={idx === activeIndex}
          class="flex items-baseline gap-1 px-2 py-1 cursor-pointer text-sm font-mono
            {idx === activeIndex ? 'bg-zinc-800 text-cyan-400' : 'text-zinc-300 hover:bg-zinc-800'}"
          onmousedown={() => { onSelect(suggestion.path); open = false; }}
        >
          <span>{suggestion.path}</span>
          {#if suggestion.type}
            <span class="text-xs text-zinc-500 ml-1">{suggestion.type}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>
