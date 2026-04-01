<script lang="ts">
  interface Props {
    value: string;
    onChange: (value: string) => void;
    envVars: string[];
    placeholder?: string;
    unknownLabel?: string;
  }

  let { value, onChange, envVars, placeholder = '', unknownLabel = 'scenario' }: Props = $props();

  let inputEl = $state<HTMLInputElement | null>(null);
  let focused = $state(false);
  let showDropdown = $state(false);
  let activeIndex = $state(0);
  let navigated = $state(false);
  let blurTimer: ReturnType<typeof setTimeout> | null = null;

  const filtered = $derived(
    envVars.filter((name) => name.toLowerCase().startsWith(value.toLowerCase()))
  );

  const isEnvVar = $derived(envVars.includes(value));

  function handleInput(e: Event) {
    const v = (e.currentTarget as HTMLInputElement).value;
    onChange(v);
    showDropdown = true;
    activeIndex = 0;
    navigated = false;
  }

  function handleFocus() {
    focused = true;
    showDropdown = true;
    activeIndex = 0;
    navigated = false;
  }

  function handleBlur() {
    if (blurTimer) clearTimeout(blurTimer);
    blurTimer = setTimeout(() => {
      showDropdown = false;
      focused = false;
      blurTimer = null;
    }, 150);
  }

  function select(name: string) {
    onChange(name);
    showDropdown = false;
    navigated = false;
    setTimeout(() => inputEl?.focus(), 0);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown || filtered.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = (activeIndex + 1) % filtered.length;
      navigated = true;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = (activeIndex - 1 + filtered.length) % filtered.length;
      navigated = true;
    } else if (e.key === 'Enter') {
      if (navigated && filtered[activeIndex]) {
        e.preventDefault();
        select(filtered[activeIndex]);
      }
    } else if (e.key === 'Escape') {
      showDropdown = false;
    }
  }

  $effect(() => {
    if (activeIndex >= filtered.length) activeIndex = 0;
  });

  $effect(() => {
    return () => {
      if (blurTimer) clearTimeout(blurTimer);
    };
  });
</script>

<div class="relative flex items-center gap-1.5">
  <input
    bind:this={inputEl}
    class="flex-1 bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
    {value}
    {placeholder}
    autocomplete="off"
    spellcheck={false}
    oninput={handleInput}
    onfocus={handleFocus}
    onblur={handleBlur}
    onkeydown={handleKeydown}
  />

  {#if value && !focused}
    {#if isEnvVar}
      <span class="shrink-0 text-[10px] font-mono px-1 py-0.5 rounded border border-cyan-500/40 text-cyan-400">env</span>
    {:else}
      <span class="shrink-0 text-[10px] font-mono px-1 py-0.5 rounded border border-amber-500/30 text-amber-400/70">{unknownLabel}</span>
    {/if}
  {/if}

  {#if showDropdown && filtered.length > 0}
    <ul class="absolute z-50 top-full left-0 mt-0.5 w-full max-h-48 overflow-y-auto bg-app-panel border border-app-border-2 rounded shadow-lg">
      {#each filtered as name, idx}
        <li>
          <button
            class="w-full text-left px-3 py-1.5 font-mono text-sm {idx === activeIndex && navigated ? 'bg-app-card text-cyan-400' : 'text-app-text hover:bg-app-card'}"
            onmousedown={(e) => { e.preventDefault(); select(name); }}
          >{name}</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
