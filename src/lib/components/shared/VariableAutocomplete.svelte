<script lang="ts">
  import { environments, activeEnvironment } from '$lib/stores/environment';

  interface VarItem {
    name: string;
    value: string;
    masked?: boolean;
  }

  interface Props {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    multiline?: boolean;
    class?: string;
    extraVars?: VarItem[];
  }

  let { value, onChange, placeholder = '', multiline = false, class: className = '', extraVars = [] }: Props = $props();

  let inputEl = $state<HTMLInputElement | HTMLTextAreaElement | null>(null);
  let showDropdown = $state(false);
  let activeIndex = $state(0);
  let triggerStart = $state(-1);

  const allVars: VarItem[] = $derived.by(() => {
    const activeEnvName = $activeEnvironment;
    const entry = $environments.find((e) => e.fileName === activeEnvName);
    const result: VarItem[] = [];
    if (entry) {
      for (const [name, val] of Object.entries(entry.environment.variables)) {
        result.push({ name, value: val });
      }
      for (const secretName of entry.environment.secrets) {
        result.push({ name: secretName, value: '••••••', masked: true });
      }
    }
    for (const v of extraVars) {
      result.push(v);
    }
    return result;
  });

  const fragment: string = $derived(triggerStart < 0 ? '' : value.slice(triggerStart + 2));

  const filtered: VarItem[] = $derived(allVars.filter((v) => v.name.toLowerCase().startsWith(fragment.toLowerCase())));

  function findTriggerStart(text: string, cursor: number): number {
    const before = text.slice(0, cursor);
    const idx = before.lastIndexOf('{{');
    if (idx === -1) return -1;
    const after = text.slice(idx + 2, cursor);
    if (after.includes('}')) return -1;
    return idx;
  }

  function handleInput(e: Event) {
    const el = e.currentTarget as HTMLInputElement | HTMLTextAreaElement;
    const cursor = el.selectionStart ?? el.value.length;
    const ts = findTriggerStart(el.value, cursor);
    triggerStart = ts;
    showDropdown = ts >= 0 && filtered.length > 0;
    activeIndex = 0;
    onChange(el.value);
  }

  function selectVar(varName: string) {
    if (triggerStart < 0) return;
    const before = value.slice(0, triggerStart);
    const after = value.slice(triggerStart).replace(/\{\{[^}]*/, '');
    const newVal = before + '{{' + varName + '}}' + after;
    onChange(newVal);
    showDropdown = false;
    triggerStart = -1;
    // Restore focus and place cursor after the inserted token
    setTimeout(() => {
      if (inputEl) {
        inputEl.focus();
        const pos = before.length + varName.length + 4;
        inputEl.setSelectionRange(pos, pos);
      }
    }, 0);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = (activeIndex + 1) % filtered.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = (activeIndex - 1 + filtered.length) % filtered.length;
    } else if (e.key === 'Enter' || e.key === 'Tab') {
      if (filtered[activeIndex]) {
        e.preventDefault();
        selectVar(filtered[activeIndex].name);
      }
    } else if (e.key === 'Escape') {
      showDropdown = false;
    }
  }

  function handleBlur() {
    // Delay to allow click on dropdown item to register
    setTimeout(() => { showDropdown = false; }, 150);
  }

  const baseClass = 'bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500';
</script>

<div class="relative {className}">
  {#if multiline}
    <textarea
      bind:this={inputEl as HTMLTextAreaElement}
      class="w-full {baseClass} min-h-[60px] resize-y"
      {value}
      {placeholder}
      oninput={handleInput}
      onkeydown={handleKeydown}
      onblur={handleBlur}
    ></textarea>
  {:else}
    <input
      bind:this={inputEl as HTMLInputElement}
      class="w-full {baseClass}"
      {value}
      {placeholder}
      oninput={handleInput}
      onkeydown={handleKeydown}
      onblur={handleBlur}
    />
  {/if}

  {#if showDropdown && filtered.length > 0}
    <ul class="absolute z-50 top-full left-0 mt-0.5 w-full max-h-48 overflow-y-auto bg-zinc-900 border border-zinc-700 rounded shadow-lg">
      {#each filtered as item, idx}
        <li>
          <button
            class="w-full text-left px-3 py-1.5 flex items-center gap-2 {idx === activeIndex ? 'bg-zinc-800' : 'hover:bg-zinc-800'}"
            onmousedown={(e) => { e.preventDefault(); selectVar(item.name); }}
          >
            <span class="font-mono text-sm {idx === activeIndex ? 'text-cyan-400' : 'text-zinc-200'}">{item.name}</span>
            <span class="text-zinc-500 text-xs truncate">{item.masked ? '••••••' : item.value}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
