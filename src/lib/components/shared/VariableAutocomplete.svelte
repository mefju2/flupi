<script lang="ts">
  import { environments, activeEnvironment } from '$lib/stores/environment';
  import type { EnvironmentEntry } from '$lib/stores/environment';
  import { project } from '$lib/stores/project';
  import { functions } from '$lib/stores/functions';
  import type { FunctionParam } from '$lib/services/tauri-commands';
  import VariableTokenDisplay from './VariableTokenDisplay.svelte';
  import VariableTooltip from './VariableTooltip.svelte';
  import LocalVarTooltip from './LocalVarTooltip.svelte';
  import ScenarioInputTooltip from './ScenarioInputTooltip.svelte';
  import FunctionTooltip from './FunctionTooltip.svelte';

  interface VarItem {
    name: string;
    value: string;
    masked?: boolean;
    kind?: 'env' | 'local' | 'input';
    description?: string;
    defaultValue?: string;
  }

  interface FnItem {
    name: string;
    params?: FunctionParam[];
  }

  interface Props {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    multiline?: boolean;
    class?: string;
    extraVars?: VarItem[];
    onExtraVarEdit?: (name: string, value: string) => void;
  }

  let { value, onChange, placeholder = '', multiline = false, class: className = '', extraVars = [], onExtraVarEdit }: Props = $props();

  let inputEl = $state<HTMLInputElement | HTMLTextAreaElement | null>(null);
  let showDropdown = $state(false);
  let activeIndex = $state(0);
  let triggerStart = $state(-1);
  let triggerIsFunction = $state(false);
  let focused = $state(false);
  let hoveredVar = $state<string | null>(null);
  let hoveredVarKind = $state<'env' | 'local' | 'input' | undefined>(undefined);
  let hoveredVarDescription = $state<string | undefined>(undefined);
  let hoveredVarDefaultValue = $state<string>('');
  let tooltipAnchor = $state<HTMLElement | null>(null);
  let hoveredFn = $state<string | null>(null);
  let fnTooltipAnchor = $state<HTMLElement | null>(null);

  const activeEnvEntry: EnvironmentEntry | null = $derived(
    $environments.find(e => e.fileName === $activeEnvironment) ?? null
  );

  const secretsList: string[] = $derived(
    activeEnvEntry?.environment.secrets ?? []
  );

  const allVars: VarItem[] = $derived.by(() => {
    const entry = activeEnvEntry;
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

  const varNames: Set<string> = $derived(new Set(allVars.map(v => v.name)));
  const fnNames: Set<string> = $derived(new Set($functions.map(f => f.name)));

  // Fragment typed after {{ or {{$
  const fragment: string = $derived(triggerStart < 0 ? '' : (() => {
    const raw = value.slice(triggerStart + 2);
    return triggerIsFunction ? raw.replace(/^\$/, '') : raw;
  })());

  const filteredVars: VarItem[] = $derived(
    allVars.filter((v) => v.name.toLowerCase().startsWith(fragment.toLowerCase()))
  );

  const filteredFns: FnItem[] = $derived(
    $functions
      .filter((f) => f.name.toLowerCase().startsWith(fragment.toLowerCase()))
      .map((f) => ({ name: f.name, params: f.params }))
  );

  // Flat list for keyboard navigation: vars first (when not in {{$ mode), then functions
  const flatItems: Array<{ kind: 'var'; item: VarItem } | { kind: 'fn'; item: FnItem }> = $derived([
    ...(triggerIsFunction ? [] : filteredVars.map(item => ({ kind: 'var' as const, item }))),
    ...filteredFns.map(item => ({ kind: 'fn' as const, item })),
  ]);

  function paramDefault(p: FunctionParam): string {
    if (p.param_type === 'number') return '0';
    if (p.param_type === 'boolean') return 'true';
    return '""';
  }

  function fnSignature(item: FnItem): string {
    if (!item.params || item.params.length === 0) return `$${item.name}()`;
    return `$${item.name}(${item.params.map((p) => `${p.name}: ${p.param_type}`).join(', ')})`;
  }

  function findTriggerStart(text: string, cursor: number): { start: number; isFunction: boolean } | null {
    const before = text.slice(0, cursor);
    const idx = before.lastIndexOf('{{');
    if (idx === -1) return null;
    const after = text.slice(idx + 2, cursor);
    if (after.includes('}')) return null;
    const isFunction = after.startsWith('$');
    return { start: idx, isFunction };
  }

  function handleInput(e: Event) {
    const el = e.currentTarget as HTMLInputElement | HTMLTextAreaElement;
    const cursor = el.selectionStart ?? el.value.length;
    const trigger = findTriggerStart(el.value, cursor);
    if (trigger) {
      triggerStart = trigger.start;
      triggerIsFunction = trigger.isFunction;
      const hasItems = trigger.isFunction ? filteredFns.length > 0 : (filteredVars.length > 0 || filteredFns.length > 0);
      showDropdown = hasItems;
    } else {
      triggerStart = -1;
      triggerIsFunction = false;
      showDropdown = false;
    }
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
    setTimeout(() => {
      if (inputEl) {
        inputEl.focus();
        const pos = before.length + varName.length + 4;
        inputEl.setSelectionRange(pos, pos);
      }
    }, 0);
  }

  function selectFn(fnItem: FnItem) {
    if (triggerStart < 0) return;
    const before = value.slice(0, triggerStart);
    const after = value.slice(triggerStart).replace(/\{\{[^}]*/, '');
    const argsStr = (fnItem.params ?? []).map(paramDefault).join(', ');
    const token = `{{$${fnItem.name}(${argsStr})}}`;
    const newVal = before + token + after;
    onChange(newVal);
    showDropdown = false;
    triggerStart = -1;
    // Place cursor after the closing token
    setTimeout(() => {
      if (inputEl) {
        inputEl.focus();
        const pos = before.length + token.length;
        inputEl.setSelectionRange(pos, pos);
      }
    }, 0);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = (activeIndex + 1) % flatItems.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = (activeIndex - 1 + flatItems.length) % flatItems.length;
    } else if (e.key === 'Enter' || e.key === 'Tab') {
      const active = flatItems[activeIndex];
      if (active) {
        e.preventDefault();
        if (active.kind === 'var') selectVar(active.item.name);
        else selectFn(active.item);
      }
    } else if (e.key === 'Escape') {
      showDropdown = false;
    }
  }

  function handleFocus() {
    focused = true;
    closeTooltip();
  }

  let blurTimer: ReturnType<typeof setTimeout> | null = null;
  let tooltipCloseTimer: ReturnType<typeof setTimeout> | null = null;

  function handleBlur() {
    if (blurTimer) clearTimeout(blurTimer);
    blurTimer = setTimeout(() => {
      showDropdown = false;
      focused = false;
      blurTimer = null;
    }, 150);
  }

  $effect(() => {
    return () => {
      if (blurTimer) clearTimeout(blurTimer);
      if (tooltipCloseTimer) clearTimeout(tooltipCloseTimer);
      if (fnTooltipCloseTimer) clearTimeout(fnTooltipCloseTimer);
    };
  });

  function onTokenHover(varName: string, anchorEl: HTMLElement) {
    if (tooltipCloseTimer) { clearTimeout(tooltipCloseTimer); tooltipCloseTimer = null; }
    const meta = allVars.find((v) => v.name === varName);
    hoveredVar = varName;
    hoveredVarKind = meta?.kind;
    hoveredVarDescription = meta?.description;
    hoveredVarDefaultValue = meta?.defaultValue ?? '';
    tooltipAnchor = anchorEl;
  }

  function scheduleTooltipClose() {
    if (tooltipCloseTimer) clearTimeout(tooltipCloseTimer);
    tooltipCloseTimer = setTimeout(() => {
      hoveredVar = null;
      tooltipAnchor = null;
      tooltipCloseTimer = null;
    }, 300);
  }

  function cancelTooltipClose() {
    if (tooltipCloseTimer) { clearTimeout(tooltipCloseTimer); tooltipCloseTimer = null; }
  }

  function closeTooltip() {
    if (tooltipCloseTimer) { clearTimeout(tooltipCloseTimer); tooltipCloseTimer = null; }
    hoveredVar = null;
    hoveredVarKind = undefined;
    hoveredVarDescription = undefined;
    hoveredVarDefaultValue = '';
    tooltipAnchor = null;
  }

  let fnTooltipCloseTimer: ReturnType<typeof setTimeout> | null = null;

  function onFunctionHover(fnName: string, anchorEl: HTMLElement) {
    if (fnTooltipCloseTimer) { clearTimeout(fnTooltipCloseTimer); fnTooltipCloseTimer = null; }
    hoveredFn = fnName;
    fnTooltipAnchor = anchorEl;
  }

  function scheduleFnTooltipClose() {
    if (fnTooltipCloseTimer) clearTimeout(fnTooltipCloseTimer);
    fnTooltipCloseTimer = setTimeout(() => {
      hoveredFn = null;
      fnTooltipAnchor = null;
      fnTooltipCloseTimer = null;
    }, 300);
  }

  function cancelFnTooltipClose() {
    if (fnTooltipCloseTimer) { clearTimeout(fnTooltipCloseTimer); fnTooltipCloseTimer = null; }
  }

  const baseClass = 'bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2';

  let wrapperEl = $state<HTMLDivElement | null>(null);
  let dropdownPos = $state({ top: 0, left: 0, width: 0 });

  $effect(() => {
    if (showDropdown && wrapperEl) {
      const rect = wrapperEl.getBoundingClientRect();
      dropdownPos = { top: rect.bottom + 2, left: rect.left, width: rect.width };
    }
  });
</script>

<div bind:this={wrapperEl} class="relative {className}">
  {#if !multiline}
    {#if !focused}
      <VariableTokenDisplay
        {value}
        vars={varNames}
        secrets={secretsList}
        fnNames={fnNames}
        {placeholder}
        onTokenHover={onTokenHover}
        onTokenLeave={scheduleTooltipClose}
        onFunctionHover={onFunctionHover}
        onFunctionLeave={scheduleFnTooltipClose}
        onclick={() => (inputEl as HTMLInputElement | null)?.focus()}
      />
    {/if}
    <input
      bind:this={inputEl as HTMLInputElement}
      class="w-full {baseClass} {!focused ? 'sr-only' : ''}"
      {value}
      {placeholder}
      oninput={handleInput}
      onkeydown={handleKeydown}
      onblur={handleBlur}
      onfocus={handleFocus}
    />
  {:else}
    {#if !focused}
      <VariableTokenDisplay
        {value}
        vars={varNames}
        secrets={secretsList}
        fnNames={fnNames}
        {placeholder}
        multiline={true}
        onTokenHover={onTokenHover}
        onTokenLeave={scheduleTooltipClose}
        onFunctionHover={onFunctionHover}
        onFunctionLeave={scheduleFnTooltipClose}
        onclick={() => (inputEl as HTMLTextAreaElement | null)?.focus()}
      />
    {/if}
    <textarea
      bind:this={inputEl as HTMLTextAreaElement}
      class="w-full {baseClass} min-h-15 resize-y {!focused ? 'sr-only' : ''}"
      {value}
      {placeholder}
      oninput={handleInput}
      onkeydown={handleKeydown}
      onblur={handleBlur}
      onfocus={handleFocus}
    ></textarea>
  {/if}

  {#if hoveredVar && tooltipAnchor}
    {#if hoveredVarKind === 'local'}
      <LocalVarTooltip
        varName={hoveredVar}
        anchorEl={tooltipAnchor}
        onclose={closeTooltip}
        onmouseenter={cancelTooltipClose}
        onmouseleave={scheduleTooltipClose}
      />
    {:else if hoveredVarKind === 'input'}
      <ScenarioInputTooltip
        varName={hoveredVar}
        anchorEl={tooltipAnchor}
        description={hoveredVarDescription}
        defaultValue={hoveredVarDefaultValue}
        onSave={(v) => onExtraVarEdit?.(hoveredVar!, v)}
        onclose={closeTooltip}
        onmouseenter={cancelTooltipClose}
        onmouseleave={scheduleTooltipClose}
      />
    {:else if $project.path}
      <VariableTooltip
        varName={hoveredVar}
        anchorEl={tooltipAnchor}
        envEntry={activeEnvEntry}
        projectPath={$project.path}
        onclose={closeTooltip}
        onmouseenter={cancelTooltipClose}
        onmouseleave={scheduleTooltipClose}
      />
    {/if}
  {/if}

  {#if hoveredFn && fnTooltipAnchor}
    {@const fnDef = $functions.find((f) => f.name === hoveredFn)}
    {#if fnDef}
      <FunctionTooltip
        fn={fnDef}
        anchorEl={fnTooltipAnchor}
        onclose={() => { hoveredFn = null; fnTooltipAnchor = null; }}
        onmouseenter={cancelFnTooltipClose}
        onmouseleave={scheduleFnTooltipClose}
      />
    {/if}
  {/if}

  {#if showDropdown && flatItems.length > 0}
    {@const showVarSection = !triggerIsFunction && filteredVars.length > 0}
    {@const showFnSection = filteredFns.length > 0}
    {@const varOffset = 0}
    {@const fnOffset = triggerIsFunction ? 0 : filteredVars.length}
    <ul
      class="fixed z-50 max-h-56 overflow-y-auto bg-app-panel border border-app-border-2 rounded shadow-lg"
      style="top: {dropdownPos.top}px; left: {dropdownPos.left}px; width: {dropdownPos.width}px"
    >
      {#if showVarSection}
        <li class="px-3 py-1 text-xs text-app-text-4 uppercase tracking-wider border-b border-app-border select-none">
          Variables
        </li>
        {#each filteredVars as item, idx}
          {@const flatIdx = varOffset + idx}
          <li>
            <button
              class="w-full text-left px-3 py-1.5 flex items-center gap-2 {flatIdx === activeIndex ? 'bg-app-card' : 'hover:bg-app-card'}"
              onmousedown={(e) => { e.preventDefault(); selectVar(item.name); }}
            >
              <span class="font-mono text-sm {flatIdx === activeIndex ? 'text-cyan-400' : 'text-app-text'}">{item.name}</span>
              <span class="text-app-text-3 text-xs truncate">{item.masked ? '••••••' : item.value}</span>
            </button>
          </li>
        {/each}
      {/if}

      {#if showFnSection}
        {#if showVarSection}
          <li class="px-3 py-1 text-xs text-app-text-4 uppercase tracking-wider border-t border-b border-app-border select-none">
            Functions
          </li>
        {:else if !triggerIsFunction}
          <li class="px-3 py-1 text-xs text-app-text-4 uppercase tracking-wider border-b border-app-border select-none">
            Functions
          </li>
        {/if}
        {#each filteredFns as item, idx}
          {@const flatIdx = fnOffset + idx}
          <li>
            <button
              class="w-full text-left px-3 py-1.5 flex items-center gap-2 {flatIdx === activeIndex ? 'bg-app-card' : 'hover:bg-app-card'}"
              onmousedown={(e) => { e.preventDefault(); selectFn(item); }}
            >
              <span class="font-mono text-sm {flatIdx === activeIndex ? 'text-cyan-400' : 'text-app-text'}">$<span class="text-cyan-300">{item.name}</span>{#if item.params && item.params.length > 0}({item.params.map((p) => `${p.name}: ${p.param_type}`).join(', ')}){:else}(){/if}</span>
            </button>
          </li>
        {/each}
      {/if}
    </ul>
  {/if}
</div>
