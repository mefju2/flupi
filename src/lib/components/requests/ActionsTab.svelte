<script lang="ts">
  import type {
    Extraction,
    PreRequestAction,
  } from "$lib/services/tauri-commands";
  import ExtractionsPanel from "$lib/components/shared/ExtractionsPanel.svelte";
  import PreRequestActionsPanel from "./PreRequestActionsPanel.svelte";

  interface Props {
    extractions: Extraction[];
    preRequestActions: PreRequestAction[];
    onUpdateExtractions: (extractions: Extraction[]) => void;
    onUpdatePreRequestActions: (actions: PreRequestAction[]) => void;
    responseSchema?: unknown;
  }

  let {
    extractions,
    preRequestActions,
    onUpdateExtractions,
    onUpdatePreRequestActions,
    responseSchema = null,
  }: Props = $props();

  type SubTab = "Pre Request" | "Post Response";
  let activeSubTab = $state<SubTab>("Pre Request");
</script>

<div class="flex flex-col h-full overflow-hidden">
  <!-- Sub-tab bar -->
  <div class="flex border-b border-app-border px-4 shrink-0">
    {#each ["Pre Request", "Post Response"] as SubTab[] as tab}
      <button
        class="text-xs px-3 py-2 transition-all duration-150 border-b-2 -mb-px {activeSubTab ===
        tab
          ? 'text-cyan-300 border-cyan-500'
          : 'text-app-text-3 border-transparent hover:text-app-text-2'}"
        onclick={() => (activeSubTab = tab)}>{tab}</button
      >
    {/each}
  </div>

  <!-- Sub-tab content -->
  <div class="flex-1 overflow-y-auto p-4">
    {#if activeSubTab === "Pre Request"}
      <PreRequestActionsPanel
        actions={preRequestActions}
        onUpdate={onUpdatePreRequestActions}
      />
    {:else}
      <ExtractionsPanel
        {extractions}
        onUpdate={onUpdateExtractions}
        {responseSchema}
      />
    {/if}
  </div>
</div>
