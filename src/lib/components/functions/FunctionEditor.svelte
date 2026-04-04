<script lang="ts">
  import { functions, selectedFunctionName } from '$lib/stores/functions';
  import { saveFunction } from '$lib/services/tauri-commands';
  import type { FunctionParam } from '$lib/services/tauri-commands';
  import { createDebouncedSave } from '$lib/services/debounced-save';
  import { project } from '$lib/stores/project';
  import JsEditor from '$lib/components/shared/JsEditor.svelte';
  import SavedIndicator from '$lib/components/shared/SavedIndicator.svelte';

  const JS_IDENTIFIER = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/;
  const PARAM_TYPES = ['string', 'number', 'boolean'] as const;

  let fn = $derived($functions.find((f) => f.name === $selectedFunctionName));

  // Local editable state
  let editedName = $state('');
  let editedBody = $state('');
  let editedParams = $state<FunctionParam[]>([]);
  let nameError = $state('');
  let syncedName = $state<string | undefined>(undefined);
  let savedRecently = $state(false);
  let snapshot = $state<{ name: string; body: string; params: FunctionParam[]; originalName: string } | null>(null);

  $effect(() => {
    if (!fn || fn.name === syncedName) return;
    syncedName = fn.name;
    editedName = fn.name;
    editedBody = fn.body;
    editedParams = fn.params ? fn.params.map((p) => ({ ...p })) : [];
    nameError = '';
  });

  const usageSignature = $derived(
    editedParams.length > 0
      ? `{{$${editedName}(${editedParams.map((p) => p.name).join(', ')})}}`
      : `{{$${editedName}()}}`
  );

  const paramNameErrors = $derived(
    editedParams.map((p) => JS_IDENTIFIER.test(p.name) ? '' : 'Must be a valid JS identifier')
  );
  const hasParamErrors = $derived(paramNameErrors.some((e) => e !== ''));

  function triggerSave() {
    if (hasParamErrors) return;
    snapshot = { name: editedName, body: editedBody, params: editedParams.map((p) => ({ ...p })), originalName: fn?.name ?? editedName };
    debouncedSave.trigger();
  }

  const debouncedSave = createDebouncedSave(async () => {
    if (!snapshot || !$project.path) return;
    const { name, body, params, originalName } = snapshot;
    const updated = { name, body, params };
    try {
      await saveFunction($project.path, updated);
      if (name !== originalName) {
        const { deleteFunction } = await import('$lib/services/tauri-commands');
        await deleteFunction($project.path, originalName);
      }
      functions.update((list) =>
        list.map((f) => (f.name === originalName ? updated : f)).sort((a, b) => a.name.localeCompare(b.name))
      );
      selectedFunctionName.set(updated.name);
      syncedName = updated.name;
      savedRecently = true;
      setTimeout(() => (savedRecently = false), 2000);
    } catch (e) {
      console.error('Failed to save function:', e);
    }
  });

  function handleNameChange(e: Event) {
    const v = (e.target as HTMLInputElement).value;
    editedName = v;
    if (!JS_IDENTIFIER.test(v)) {
      nameError = 'Must be a valid JS identifier';
      return;
    }
    if (v !== fn?.name && $functions.some((f) => f.name === v)) {
      nameError = 'Name already in use';
      return;
    }
    nameError = '';
    triggerSave();
  }

  function handleBodyChange(v: string) {
    editedBody = v;
    triggerSave();
  }

  function addParam() {
    editedParams = [...editedParams, { name: `param${editedParams.length + 1}`, param_type: 'string' }];
    triggerSave();
  }

  function removeParam(idx: number) {
    editedParams = editedParams.filter((_, i) => i !== idx);
    triggerSave();
  }

  function updateParamName(idx: number, name: string) {
    editedParams = editedParams.map((p, i) => i === idx ? { ...p, name } : p);
    triggerSave();
  }

  function updateParamType(idx: number, param_type: string) {
    editedParams = editedParams.map((p, i) => i === idx ? { ...p, param_type: param_type as FunctionParam['param_type'] } : p);
    triggerSave();
  }
</script>

<div class="flex flex-col h-full bg-app-bg">
  {#if fn}
    <div class="flex items-center gap-3 px-6 py-3 border-b border-app-border shrink-0">
      <input
        class="bg-app-card border font-mono text-sm px-2 py-1 rounded outline-none
          {nameError ? 'border-red-500 text-red-400' : 'border-app-border-2 focus:border-cyan-500 text-app-text'}"
        value={editedName}
        oninput={handleNameChange}
        spellcheck={false}
        aria-label="Function name"
      />
      {#if nameError}
        <span class="text-xs text-red-400">{nameError}</span>
      {:else}
        <span class="font-mono text-xs text-app-text-4 select-all bg-app-card border border-app-border-2 px-2 py-1 rounded">
          {usageSignature}
        </span>
      {/if}
      <SavedIndicator visible={savedRecently} />
    </div>

    <div class="px-6 py-3 border-b border-app-border shrink-0">
      <div class="flex items-center justify-between mb-2">
        <span class="text-xs text-app-text-4 uppercase tracking-wider">Parameters</span>
        <button
          class="text-xs text-cyan-400 hover:text-cyan-300 transition-colors"
          onclick={addParam}
        >+ Add parameter</button>
      </div>
      {#if editedParams.length === 0}
        <p class="text-xs text-app-text-4 italic">No parameters — body can use <span class="font-mono">args[0]</span>, <span class="font-mono">args[1]</span>, …</p>
      {:else}
        <ul class="space-y-1.5">
          {#each editedParams as param, idx}
            <li class="flex items-center gap-2">
              <span class="text-xs text-app-text-4 font-mono w-5 text-right shrink-0">{idx + 1}</span>
              <input
                class="bg-app-card border rounded px-2 py-0.5 text-sm font-mono text-app-text focus:outline-none w-32
                  {paramNameErrors[idx] ? 'border-red-500 text-red-400' : 'border-app-border-2 focus:border-cyan-500'}"
                value={param.name}
                oninput={(e) => updateParamName(idx, (e.target as HTMLInputElement).value)}
                spellcheck={false}
                aria-label="Parameter name"
                title={paramNameErrors[idx] || undefined}
              />
              <select
                class="bg-app-card border border-app-border-2 rounded px-2 py-0.5 text-xs text-app-text-3 focus:outline-none focus:border-cyan-500"
                value={param.param_type}
                onchange={(e) => updateParamType(idx, (e.target as HTMLSelectElement).value)}
                aria-label="Parameter type"
              >
                {#each PARAM_TYPES as t}
                  <option value={t}>{t}</option>
                {/each}
              </select>
              <button
                class="text-app-text-4 hover:text-red-400 transition-colors text-sm leading-none ml-auto"
                onclick={() => removeParam(idx)}
                aria-label="Remove parameter"
              >×</button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div class="flex-1 min-h-0">
      <JsEditor value={editedBody} onChange={handleBodyChange} />
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center text-app-text-4 text-sm">
      Select a function to edit.
    </div>
  {/if}
</div>
