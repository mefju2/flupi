<script lang="ts">
  interface Props {
    value: string;
    onChange: (value: string) => void;
    envVars: string[];
    placeholder?: string;
  }

  let { value, onChange, envVars, placeholder = 'Variable name…' }: Props = $props();

  let open = $state(false);
  let activeIndex = $state(-1);

  let filtered = $derived(
    envVars
      .filter((v) => !value || v.toLowerCase().includes(value.toLowerCase()))
      .slice(0, 10),
  );

  let isNew = $derived(value.length > 0 && !envVars.includes(value));

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
      onChange(filtered[activeIndex]);
      open = false;
    } else if (e.key === 'Escape') {
      open = false;
    }
  }

  function handleInput(e: Event) {
    onChange((e.currentTarget as HTMLInputElement).value);
    open = true;
    activeIndex = -1;
  }

  function select(name: string) {
    onChange(name);
    open = false;
  }
</script>

<div class="relative">
  <div class="relative">
    <input
      class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2 {isNew ? 'pr-12' : ''}"
      {value}
      {placeholder}
      onfocus={handleFocus}
      onblur={handleBlur}
      onkeydown={handleKeydown}
      oninput={handleInput}
      autocomplete="off"
      spellcheck={false}
    />
    {#if isNew}
      <span class="group/badge absolute right-2 top-1/2 -translate-y-1/2">
        <span class="text-[10px] font-sans font-medium text-cyan-500 bg-cyan-500/10 px-1 rounded cursor-default">+new</span>
        <span class="absolute bottom-full right-0 mb-1.5 hidden group-hover/badge:block pointer-events-none z-50">
          <span class="block whitespace-nowrap bg-app-card border border-app-border-2 text-app-text-2 text-xs font-sans px-2 py-1 rounded shadow-lg">
            Will be created in your env file after the first successful extraction
          </span>
        </span>
      </span>
    {/if}
  </div>

  {#if open && filtered.length > 0}
    <ul
      class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-app-card border border-app-border-2 rounded shadow-lg max-h-40 overflow-y-auto"
      onpointerdown={(e) => e.stopPropagation()}
    >
      {#each filtered as name, i}
        <li>
          <button
            type="button"
            class="w-full text-left px-2 py-1 text-sm font-mono text-app-text hover:bg-app-hover transition-colors {i === activeIndex ? 'bg-app-hover' : ''}"
            onpointerdown={() => select(name)}
          >{name}</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
