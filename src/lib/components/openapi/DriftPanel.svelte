<script lang="ts">
  import { project } from '$lib/stores/project';
  import { driftedIdsBySource } from '$lib/stores/openapi';
  import { activeRequest } from '$lib/stores/requests';
  import { getDriftDetails, resolveDrift, getRequest } from '$lib/services/tauri-commands';
  import type { DriftDetails, PathCandidate } from '$lib/services/tauri-commands';
  import SchemaDiff from './SchemaDiff.svelte';

  interface Props {
    requestId: string;
    onDone: () => void;
  }

  let { requestId, onDone }: Props = $props();

  let details = $state<DriftDetails | null>(null);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let accepting = $state(false);
  let acceptError = $state<string | null>(null);
  // For the rename picker: pre-select the top-ranked candidate when details load.
  let selectedOperationId = $state<string | null>(null);

  $effect(() => {
    if (!$project.path || !requestId) return;
    let cancelled = false;
    loading = true;
    loadError = null;
    details = null;
    selectedOperationId = null;
    getDriftDetails($project.path, requestId)
      .then((d) => {
        if (!cancelled) {
          details = d;
          // Pre-select the highest-ranked candidate.
          selectedOperationId = d.candidates[0]?.operationId ?? null;
        }
      })
      .catch((e) => { if (!cancelled) loadError = String(e); })
      .finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });

  function removeFromDriftStore() {
    driftedIdsBySource.update((prev) => {
      const next = new Map(prev);
      for (const [srcId, ids] of next) {
        const filtered = ids.filter((id) => id !== requestId);
        if (filtered.length === 0) next.delete(srcId);
        else next.set(srcId, filtered);
      }
      return next;
    });
  }

  async function handleAccept() {
    if (!$project.path || !details || details.operationRemoved) return;
    // Rename case: a candidate must be chosen.
    const chosenId = details.candidates.length > 0 ? selectedOperationId : null;
    if (details.candidates.length > 0 && !chosenId) return;
    accepting = true;
    acceptError = null;
    try {
      await resolveDrift($project.path, requestId, details.sourceId, chosenId);
      const updated = await getRequest($project.path, requestId);
      activeRequest.set(updated);
      removeFromDriftStore();
      onDone();
    } catch (e) {
      acceptError = e instanceof Error ? e.message : String(e);
    } finally {
      accepting = false;
    }
  }

  function handleDismiss() {
    removeFromDriftStore();
    onDone();
  }

  function isAcceptDisabled(d: DriftDetails | null): boolean {
    if (!d || d.operationRemoved || loading || !!loadError || accepting) return true;
    if (d.candidates.length > 0 && !selectedOperationId) return true;
    return false;
  }
</script>

<div class="flex flex-col h-full bg-app-bg">

  <!-- Header -->
  <div class="flex items-center justify-between gap-4 px-4 py-3 border-b border-amber-800/40 bg-amber-950/20">
    <div class="flex items-center gap-2 min-w-0">
      <span class="text-amber-400 shrink-0">⚠</span>
      <span class="text-sm font-semibold text-amber-300">Schema drift detected</span>
      <span class="font-mono text-xs text-app-text-4 truncate">{requestId}</span>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      {#if !details?.operationRemoved}
        <button
          class="px-3 py-1.5 text-xs bg-amber-600 hover:bg-amber-500 text-white rounded transition-colors disabled:opacity-50"
          disabled={isAcceptDisabled(details)}
          onclick={handleAccept}
        >{accepting ? 'Accepting…' : 'Accept changes'}</button>
      {/if}
      <button
        class="px-3 py-1.5 text-xs bg-app-card hover:bg-app-hover text-app-text-2 rounded transition-colors"
        onclick={handleDismiss}
      >Dismiss</button>
    </div>
  </div>

  {#if acceptError}
    <p class="px-4 py-2 text-xs text-red-400 border-b border-app-border">{acceptError}</p>
  {/if}

  <!-- Body -->
  {#if loading}
    <p class="p-6 text-xs text-app-text-4">Loading diff…</p>
  {:else if loadError}
    <p class="p-6 text-xs text-red-400">{loadError}</p>
  {:else if details}
    <div class="flex-1 overflow-y-auto p-6 flex flex-col gap-5">

      <!-- Meta -->
      <p class="text-xs text-app-text-3">
        Source: <span class="font-mono">{details.sourceId}</span>
        &nbsp;·&nbsp;Operation: <span class="font-mono">{details.operationId}</span>
      </p>

      {#if details.operationRemoved}
        <!-- Operation deleted, no candidates -->
        <div class="flex gap-3 p-4 rounded-lg border border-red-800/50 bg-red-950/20">
          <span class="text-red-400 text-lg shrink-0">⊘</span>
          <div class="flex flex-col gap-1">
            <p class="text-sm font-medium text-red-300">Operation no longer exists</p>
            <p class="text-xs text-app-text-3">
              The operation <span class="font-mono">{details.operationId}</span> was removed from
              the linked spec and no similar path was found.
              Dismiss to keep it as a standalone request, or delete it.
            </p>
          </div>
        </div>
      {:else}
        <!-- Rename picker: operationId gone but candidates available -->
        {#if details.candidates.length > 0}
          <div class="flex flex-col gap-3">
            <div>
              <p class="text-xs font-semibold text-app-text-2 uppercase tracking-wider mb-1">Select the new endpoint</p>
              <p class="text-xs text-app-text-3">
                <span class="font-mono line-through text-red-400/80">{details.storedPath}</span>
                no longer exists. Choose the endpoint this request should map to:
              </p>
            </div>
            <div class="flex flex-col gap-1.5">
              {#each details.candidates as candidate (candidate.operationId)}
                {@const selected = selectedOperationId === candidate.operationId}
                <button
                  class="flex items-start gap-3 p-3 rounded-lg border text-left transition-colors {selected
                    ? 'border-amber-600/60 bg-amber-950/30'
                    : 'border-app-border bg-app-card hover:bg-app-hover'}"
                  onclick={() => (selectedOperationId = candidate.operationId)}
                >
                  <span class="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 {selected ? 'border-amber-500' : 'border-app-text-4'}">
                    {#if selected}
                      <span class="h-2 w-2 rounded-full bg-amber-500"></span>
                    {/if}
                  </span>
                  <div class="flex flex-col gap-0.5 min-w-0">
                    <span class="font-mono text-sm text-app-text-1 truncate">
                      <span class="text-app-text-3 text-xs uppercase mr-1">{candidate.method}</span>{candidate.path}
                    </span>
                    {#if candidate.summary}
                      <span class="text-xs text-app-text-3 truncate">{candidate.summary}</span>
                    {/if}
                  </div>
                </button>
              {/each}
            </div>
          </div>

        {:else if details.pathChanged}
          <!-- Exact-match operationId, only path changed — deterministic, no picker needed -->
          <div class="flex flex-col gap-2 p-4 rounded-lg border border-app-border bg-app-card">
            <p class="text-xs font-semibold text-app-text-2 uppercase tracking-wider">Path changed</p>
            <div class="flex items-center gap-3 font-mono text-sm">
              <span class="line-through text-red-400">{details.storedPath}</span>
              <span class="text-app-text-4">→</span>
              <span class="text-green-400">{details.currentPath}</span>
            </div>
            <p class="text-xs text-app-text-3">
              Accepting will update the request URL to match the spec.
            </p>
          </div>
        {/if}

        <!-- Schema change (shown in addition to path info when applicable) -->
        {#if details.schemaChanged}
          <div class="flex flex-col gap-4 p-4 rounded-lg border border-app-border bg-app-card">
            <p class="text-xs font-semibold text-app-text-2 uppercase tracking-wider">Schema changes</p>
            <SchemaDiff
              label="Request schema"
              oldSchema={details.storedRequestSchema}
              newSchema={details.newRequestSchema}
            />
            <SchemaDiff
              label="Response schema"
              oldSchema={details.storedResponseSchema}
              newSchema={details.newResponseSchema}
            />
            <p class="text-xs text-app-text-3">
              Accepting will update the stored schema reference used for autocomplete and
              extraction hints.
            </p>
          </div>
        {/if}

        {#if !details.pathChanged && !details.schemaChanged && details.candidates.length === 0}
          <p class="text-xs text-app-text-4 italic">No details available for this drift.</p>
        {/if}
      {/if}
    </div>
  {/if}
</div>

