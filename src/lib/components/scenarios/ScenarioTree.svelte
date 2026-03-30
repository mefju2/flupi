<script lang="ts">
  import { onMount } from 'svelte';
  import { project } from '$lib/stores/project';
  import { scenarioTree, activeScenarioId, activeScenario } from '$lib/stores/scenarios';
  import {
    loadScenarioTree, getScenario, createScenario, deleteScenario,
    renameScenario, duplicateScenario, type ScenarioTreeNode,
  } from '$lib/services/tauri-commands';
  import ContextMenu from '$lib/components/shared/ContextMenu.svelte';

  let toast = $state<string | null>(null);
  let contextMenu: { x: number; y: number; items: { label: string; action: () => void; danger?: boolean }[] } | null = $state(null);
  let pendingInput = $state<{ type: 'rename' | 'new'; id: string; value: string } | null>(null);
  let expandedGroups = $state<Set<string>>(new Set());

  function showToast(msg: string) { toast = msg; setTimeout(() => (toast = null), 3000); }
  function focusOnMount(el: HTMLElement) { el.focus(); if (el instanceof HTMLInputElement) el.select(); }

  async function reload() {
    if (!$project.path) return;
    try { scenarioTree.set(await loadScenarioTree($project.path)); }
    catch (e) { console.error('Failed to load scenario tree:', e); }
  }

  onMount(reload);

  async function selectScenario(id: string) {
    if (!$project.path) return;
    activeScenarioId.set(id);
    try { activeScenario.set(await getScenario($project.path, id)); }
    catch (e) { console.error('Failed to load scenario:', e); }
  }

  function openContextMenu(e: MouseEvent, node: ScenarioTreeNode) {
    const items: { label: string; action: () => void; danger?: boolean }[] = [];
    if (node.type === 'Scenario') {
      items.push({ label: 'Rename', action: () => { pendingInput = { type: 'rename', id: node.id, value: node.name }; } });
      items.push({ label: 'Duplicate', action: async () => {
        if (!$project.path) return;
        try { await duplicateScenario($project.path, node.id); await reload(); } catch (err) { console.error(err); }
      }});
      items.push({ label: 'Delete', danger: true, action: async () => {
        if (!$project.path || !confirm('Delete this scenario?')) return;
        try {
          await deleteScenario($project.path, node.id);
          if ($activeScenarioId === node.id) { activeScenarioId.set(null); activeScenario.set(null); }
          await reload();
        } catch (err) { console.error(err); }
      }});
    } else if (node.type === 'Group') {
      items.push({ label: 'New Scenario in Group', action: () => { pendingInput = { type: 'new', id: node.name, value: 'New Scenario' }; } });
    }
    contextMenu = { x: e.clientX, y: e.clientY, items };
  }

  async function confirmPendingInput() {
    if (!pendingInput || !$project.path) return;
    const { type, id, value } = pendingInput;
    const trimmed = value.trim();
    if (!trimmed) { pendingInput = null; return; }
    pendingInput = null;
    try {
      if (type === 'new') await createScenario($project.path, id || null, trimmed);
      else if (type === 'rename') { await renameScenario($project.path, id, trimmed); showToast('Renamed.'); }
      await reload();
    } catch (e) { console.error(e); }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter') confirmPendingInput();
    if (e.key === 'Escape') pendingInput = null;
  }

  function toggleGroup(name: string) {
    const next = new Set(expandedGroups);
    if (next.has(name)) next.delete(name); else next.add(name);
    expandedGroups = next;
  }

  const { groups, rootScenarios } = $derived.by(() => {
    const groups: (ScenarioTreeNode & { type: 'Group' })[] = [];
    const rootScenarios: (ScenarioTreeNode & { type: 'Scenario' })[] = [];
    for (const n of $scenarioTree) {
      if (n.type === 'Group') groups.push(n as ScenarioTreeNode & { type: 'Group' });
      else if (n.type === 'Scenario') rootScenarios.push(n as ScenarioTreeNode & { type: 'Scenario' });
    }
    return { groups, rootScenarios };
  });
</script>

<div class="flex flex-col h-full bg-app-panel">
  <div class="px-3 py-2 text-xs text-app-text-3 uppercase tracking-wider border-b border-app-border">
    Scenarios
  </div>

  <div class="flex-1 overflow-y-auto px-1 py-1">
    {#each groups as group (group.name)}
      <div>
        <div
          class="flex items-center gap-1.5 px-2 py-1 text-sm cursor-pointer select-none text-app-text-2 hover:bg-app-card/50 hover:text-app-text rounded"
          role="button" tabindex="0"
          onclick={() => toggleGroup(group.name)}
          onkeydown={(e) => e.key === 'Enter' && toggleGroup(group.name)}
          oncontextmenu={(e) => { e.preventDefault(); openContextMenu(e, group); }}
        >
          <span class="text-app-text-3 text-xs">{expandedGroups.has(group.name) ? '▾' : '▸'}</span>
          <span class="text-app-text-3 text-xs">📁</span>
          <span class="truncate">{group.name}</span>
        </div>
        {#if expandedGroups.has(group.name)}
          <div class="ml-3 border-l border-app-border pl-1">
            {#each group.children as child}
              {#if child.type === 'Scenario'}
                <div
                  class="flex items-center gap-1.5 px-2 py-1 text-sm cursor-pointer select-none rounded
                    {$activeScenarioId === child.id ? 'bg-app-card text-app-text' : 'text-app-text-2 hover:bg-app-card/50 hover:text-app-text'}"
                  role="button" tabindex="0"
                  onclick={() => selectScenario(child.id)}
                  onkeydown={(e) => e.key === 'Enter' && selectScenario(child.id)}
                  oncontextmenu={(e) => { e.preventDefault(); openContextMenu(e, child); }}
                >
                  <span class="text-app-text-3 text-xs">⚡</span>
                  <span class="truncate">{child.name}</span>
                </div>
              {/if}
            {/each}
            {#if group.children.length === 0}
              <p class="px-2 py-1 text-xs text-app-text-4 italic">Empty</p>
            {/if}
          </div>
        {/if}
      </div>
    {/each}

    {#each rootScenarios as scenario (scenario.id)}
      <div
        class="flex items-center gap-1.5 px-2 py-1 text-sm cursor-pointer select-none rounded
          {$activeScenarioId === scenario.id ? 'bg-app-card text-app-text' : 'text-app-text-2 hover:bg-app-card/50 hover:text-app-text'}"
        role="button" tabindex="0"
        onclick={() => selectScenario(scenario.id)}
        onkeydown={(e) => e.key === 'Enter' && selectScenario(scenario.id)}
        oncontextmenu={(e) => { e.preventDefault(); openContextMenu(e, scenario); }}
      >
        <span class="text-app-text-3 text-xs">⚡</span>
        <span class="truncate">{scenario.name}</span>
      </div>
    {/each}

    {#if $scenarioTree.length === 0}
      <p class="px-3 py-4 text-xs text-app-text-4">No scenarios yet. Create one to chain requests together.</p>
    {/if}
  </div>

  {#if toast}
    <div class="px-3 py-2 text-xs text-cyan-400 bg-app-card border-t border-app-border-2 animate-pulse">{toast}</div>
  {/if}

  {#if pendingInput}
    <div class="px-3 py-2 border-t border-app-border">
      <p class="text-xs text-app-text-3 mb-1">{pendingInput.type === 'new' ? 'Create:' : 'Rename to:'}</p>
      <input
        class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm text-app-text font-mono focus:outline-none focus:border-cyan-600"
        bind:value={pendingInput.value} onkeydown={handleKey} use:focusOnMount
      />
      <div class="flex gap-2 mt-1">
        <button class="text-xs text-cyan-500" onclick={confirmPendingInput}>OK</button>
        <button class="text-xs text-app-text-3" onclick={() => pendingInput = null}>Cancel</button>
      </div>
    </div>
  {/if}

  <div class="border-t border-app-border px-3 py-2">
    <button
      class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors"
      onclick={() => { pendingInput = { type: 'new', id: '', value: 'New Scenario' }; }}
    >+ New Scenario</button>
  </div>
</div>

{#if contextMenu}
  <ContextMenu x={contextMenu.x} y={contextMenu.y} items={contextMenu.items} onClose={() => contextMenu = null} />
{/if}
