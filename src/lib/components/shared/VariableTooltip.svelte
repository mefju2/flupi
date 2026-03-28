<script lang="ts">
  import { environments } from '$lib/stores/environment';
  import type { EnvironmentEntry } from '$lib/stores/environment';
  import { saveEnvironment, saveSecrets } from '$lib/services/tauri-commands';
  import { onMount, tick } from 'svelte';

  interface Props {
    varName: string;
    anchorEl: HTMLElement;
    envEntry: EnvironmentEntry | null;
    projectPath: string;
    onclose: () => void;
  }

  let { varName, anchorEl, envEntry, projectPath, onclose }: Props = $props();

  let tooltipEl = $state<HTMLElement | null>(null);
  let style = $state('position: fixed; opacity: 0; top: 0; left: 0;');

  const isSecret = $derived(
    envEntry ? envEntry.environment.secrets.includes(varName) : false
  );

  const isNewVar = $derived(
    envEntry
      ? !(varName in envEntry.environment.variables) &&
        !envEntry.environment.secrets.includes(varName)
      : false
  );

  const initialValue = $derived(
    envEntry
      ? isSecret
        ? (envEntry.secrets[varName] ?? '')
        : (envEntry.environment.variables[varName] ?? '')
      : ''
  );

  let currentValue = $state('');
  let debounceId: ReturnType<typeof setTimeout> | null = null;
  let inputEl = $state<HTMLInputElement | null>(null);

  onMount(async () => {
    currentValue = initialValue;
    await tick();

    const rect = anchorEl.getBoundingClientRect();
    const tipHeight = tooltipEl?.offsetHeight ?? 100;
    const tipWidth = tooltipEl?.offsetWidth ?? 256;

    let top: number;
    if (rect.top - tipHeight - 6 > 0) {
      top = rect.top - tipHeight - 6;
    } else {
      top = rect.bottom + 6;
    }

    let left = Math.min(rect.left, window.innerWidth - tipWidth - 8);
    left = Math.max(8, left);

    style = `position: fixed; top: ${top}px; left: ${left}px;`;
    inputEl?.focus();

    // Click outside closes the tooltip
    const handleClickOutside = (e: MouseEvent) => {
      if (tooltipEl && !tooltipEl.contains(e.target as Node)) {
        onclose();
      }
    };
    // Use setTimeout to avoid immediately closing on the same click that opened it
    const timerId = setTimeout(() => {
      document.addEventListener('click', handleClickOutside);
    }, 0);

    return () => {
      clearTimeout(timerId);
      document.removeEventListener('click', handleClickOutside);
      if (debounceId) clearTimeout(debounceId);
    };
  });

  function handleInput(e: Event) {
    const newValue = (e.target as HTMLInputElement).value;
    currentValue = newValue;
    if (debounceId) clearTimeout(debounceId);
    debounceId = setTimeout(() => saveValue(newValue), 400);
  }

  async function saveValue(newValue: string) {
    if (!envEntry) return;

    if (isSecret) {
      const updatedSecrets = { ...envEntry.secrets, [varName]: newValue };
      await saveSecrets(projectPath, envEntry.fileName, updatedSecrets);
      environments.update(list =>
        list.map(e =>
          e.fileName === envEntry!.fileName ? { ...e, secrets: updatedSecrets } : e
        )
      );
    } else {
      const updatedVars = { ...envEntry.environment.variables, [varName]: newValue };
      const updatedEnv = { ...envEntry.environment, variables: updatedVars };
      await saveEnvironment(projectPath, envEntry.fileName, updatedEnv);
      environments.update(list =>
        list.map(e =>
          e.fileName === envEntry!.fileName ? { ...e, environment: updatedEnv } : e
        )
      );
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onclose();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={tooltipEl}
  {style}
  class="w-64 bg-app-panel border border-app-border-2 rounded shadow-lg p-3 text-sm z-50"
  onkeydown={handleKeydown}
>
  <div class="flex items-center justify-between mb-2">
    <span class="font-mono text-app-text-3 text-xs">{`{{${varName}}}`}</span>
    <div class="flex items-center gap-2">
      <span class="text-xs px-1.5 py-0.5 rounded bg-app-card text-app-text-3">
        {isSecret ? 'Secret' : 'Variable'}
      </span>
      <button
        class="text-app-text-4 hover:text-app-text leading-none"
        onclick={onclose}
        aria-label="Close"
      >×</button>
    </div>
  </div>

  {#if envEntry}
    <input
      bind:this={inputEl}
      type="text"
      value={currentValue}
      oninput={handleInput}
      class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text focus:outline-none focus:border-cyan-500 mt-1"
      placeholder="(empty)"
    />
    {#if isNewVar}
      <p class="text-app-text-3 text-xs mt-1 italic">Will be added as a variable to the environment.</p>
    {/if}
  {:else}
    <p class="text-app-text-4 text-xs italic">No active environment</p>
  {/if}
</div>
