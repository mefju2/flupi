<script lang="ts">
  import { functions, selectedFunctionName } from '$lib/stores/functions';
  import { saveFunction, deleteFunction } from '$lib/services/tauri-commands';
  import { project } from '$lib/stores/project';

  let creatingNew = $state(false);
  let newName = $state('');
  let inputEl = $state<HTMLInputElement | undefined>(undefined);
  let nameError = $state('');
  let pendingDelete = $state<string | null>(null);

  const JS_IDENTIFIER = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/;

  function startCreating() {
    creatingNew = true;
    newName = '';
    nameError = '';
    setTimeout(() => inputEl?.focus(), 0);
  }

  async function confirmCreate() {
    const trimmed = newName.trim();
    if (!trimmed) { cancelCreate(); return; }
    if (!JS_IDENTIFIER.test(trimmed)) {
      nameError = 'Must be a valid JS identifier (no spaces or special characters)';
      return;
    }
    if ($functions.some((f) => f.name === trimmed)) {
      nameError = 'A function with that name already exists';
      return;
    }
    if (!$project.path) return;

    const fn = { name: trimmed, body: `// Arguments are available as args[0], args[1], ...\nreturn args[0];` };
    creatingNew = false;
    newName = '';
    nameError = '';

    try {
      await saveFunction($project.path, fn);
      functions.update((list) => [...list, fn].sort((a, b) => a.name.localeCompare(b.name)));
      selectedFunctionName.set(fn.name);
    } catch (e) {
      console.error('Failed to create function:', e);
    }
  }

  function cancelCreate() {
    creatingNew = false;
    newName = '';
    nameError = '';
  }

  async function remove(name: string) {
    pendingDelete = name;
  }

  async function confirmDelete(name: string) {
    if (!$project.path) return;
    pendingDelete = null;
    try {
      await deleteFunction($project.path, name);
      functions.update((list) => list.filter((f) => f.name !== name));
      if ($selectedFunctionName === name) selectedFunctionName.set($functions[0]?.name ?? null);
    } catch (e) {
      console.error('Failed to delete function:', e);
    }
  }
</script>

<div class="flex flex-col h-full bg-app-panel">
  <div class="px-3 py-2 text-xs text-app-text-3 uppercase tracking-wider border-b border-app-border">
    Functions
  </div>

  <div class="flex-1 overflow-y-auto">
    {#each $functions as fn (fn.name)}
      <div
        class="group flex items-center justify-between px-3 py-2 text-sm cursor-pointer select-none
          {$selectedFunctionName === fn.name
            ? 'border-l-2 border-cyan-500 bg-app-card text-app-text'
            : 'border-l-2 border-transparent text-app-text-2 hover:bg-app-card/50 hover:text-app-text'}"
        role="button"
        tabindex="0"
        onclick={() => selectedFunctionName.set(fn.name)}
        onkeydown={(e) => e.key === 'Enter' && selectedFunctionName.set(fn.name)}
      >
        <span class="truncate font-mono text-xs">{fn.name}</span>
        {#if pendingDelete === fn.name}
          <span class="flex items-center gap-1 shrink-0 ml-2">
            <span class="text-xs text-app-text-3">Delete?</span>
            <button
              class="text-red-400 hover:text-red-300 text-base leading-none"
              onclick={(e) => { e.stopPropagation(); confirmDelete(fn.name); }}
              aria-label="Confirm delete"
            >×</button>
            <button
              class="text-xs text-app-text-4 hover:text-app-text-2"
              onclick={(e) => { e.stopPropagation(); pendingDelete = null; }}
              aria-label="Cancel delete"
            >cancel</button>
          </span>
        {:else}
          <button
            class="opacity-0 group-hover:opacity-30 hover:opacity-100! text-app-text-4 hover:text-red-400 transition-opacity text-base leading-none shrink-0 ml-2"
            onclick={(e) => { e.stopPropagation(); remove(fn.name); }}
            aria-label="Delete function"
          >×</button>
        {/if}
      </div>
    {/each}

    {#if $functions.length === 0}
      <p class="px-3 py-4 text-xs text-app-text-4">No functions yet.</p>
    {/if}
  </div>

  <div class="border-t border-app-border px-3 py-2">
    {#if creatingNew}
      <input
        bind:this={inputEl}
        bind:value={newName}
        class="w-full bg-app-card text-app-text text-xs px-2 py-1 rounded outline-none border
          {nameError ? 'border-red-500' : 'border-app-border-2 focus:border-cyan-500'} font-mono"
        placeholder="functionName"
        onkeydown={(e) => { if (e.key === 'Enter') confirmCreate(); else if (e.key === 'Escape') cancelCreate(); }}
        onblur={() => { if (!nameError) confirmCreate(); }}
      />
      {#if nameError}
        <p class="mt-1 text-xs text-red-400">{nameError}</p>
      {/if}
    {:else}
      <button
        class="text-xs text-app-text-3 hover:text-app-text-2 transition-colors"
        onclick={startCreating}
      >+ New Function</button>
    {/if}
  </div>
</div>
