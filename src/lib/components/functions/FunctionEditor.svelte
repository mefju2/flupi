<script lang="ts">
  import { functions, selectedFunctionName } from '$lib/stores/functions';
  import { saveFunction } from '$lib/services/tauri-commands';
  import { createDebouncedSave } from '$lib/services/debounced-save';
  import { project } from '$lib/stores/project';
  import JsEditor from '$lib/components/shared/JsEditor.svelte';
  import SavedIndicator from '$lib/components/shared/SavedIndicator.svelte';

  const JS_IDENTIFIER = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/;

  let fn = $derived($functions.find((f) => f.name === $selectedFunctionName));

  // Local editable state
  let editedName = $state('');
  let editedBody = $state('');
  let nameError = $state('');
  let syncedName = $state<string | undefined>(undefined);
  let savedRecently = $state(false);
  let snapshot = $state<{ name: string; body: string; originalName: string } | null>(null);

  $effect(() => {
    if (!fn || fn.name === syncedName) return;
    syncedName = fn.name;
    editedName = fn.name;
    editedBody = fn.body;
    nameError = '';
  });

  const debouncedSave = createDebouncedSave(async () => {
    if (!snapshot || !$project.path) return;
    const { name, body, originalName } = snapshot;
    const updated = { name, body };
    try {
      await saveFunction($project.path, updated);
      if (name !== originalName) {
        // Name changed: save new file first, then delete old (atomic-safer order)
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
    snapshot = { name: editedName, body: editedBody, originalName: fn?.name ?? editedName };
    debouncedSave.trigger();
  }

  function handleBodyChange(v: string) {
    editedBody = v;
    snapshot = { name: editedName, body: editedBody, originalName: fn?.name ?? editedName };
    debouncedSave.trigger();
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
          {'{{$' + editedName + '(arg1, arg2)}}'}
        </span>
      {/if}
      <SavedIndicator visible={savedRecently} />
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
