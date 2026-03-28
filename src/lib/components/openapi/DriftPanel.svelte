<script lang="ts">
  import { project } from '$lib/stores/project';
  import { driftedRequestIds } from '$lib/stores/openapi';
  import { getRequest } from '$lib/services/tauri-commands';

  interface Props {
    requestId: string;
    onResolved: () => void;
  }

  let { requestId, onResolved }: Props = $props();

  let request = $state<Awaited<ReturnType<typeof getRequest>> | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  $effect(() => {
    if (!$project.path || !requestId) return;
    loading = true;
    error = null;
    getRequest($project.path, requestId)
      .then((r) => { request = r; })
      .catch((e) => { error = String(e); })
      .finally(() => { loading = false; });
  });

  function formatSchema(schema: unknown): string {
    if (schema === null || schema === undefined) return '';
    try { return JSON.stringify(schema, null, 2); } catch { return String(schema); }
  }

  function handleResolve() {
    driftedRequestIds.update((prev) => {
      const next = new Set(prev);
      next.delete(requestId);
      return next;
    });
    onResolved();
  }
</script>

<div class="flex flex-col h-full bg-app-bg">
  <div class="flex items-center justify-between px-4 py-3 border-b border-app-border">
    <div class="flex flex-col gap-0.5">
      <span class="text-sm font-semibold text-app-text">Drift Detected</span>
      <span class="font-mono text-xs text-app-text-3">{requestId}</span>
    </div>
    <button
      class="px-3 py-1.5 text-xs bg-app-card hover:bg-app-hover text-app-text-2 rounded transition-colors"
      onclick={handleResolve}
    >Mark as Resolved</button>
  </div>

  {#if loading}
    <p class="p-4 text-xs text-app-text-4">Loading…</p>
  {:else if error}
    <p class="p-4 text-xs text-red-400">{error}</p>
  {:else if request?.templateRef}
    {@const ref = request.templateRef}
    <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-4">
      <div class="flex gap-2 text-xs text-app-text-3">
        <span>source: <span class="font-mono text-app-text-3">{ref.sourceId}</span></span>
        <span>·</span>
        <span>operation: <span class="font-mono text-app-text-3">{ref.operationId}</span></span>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div class="flex flex-col gap-1">
          <h3 class="text-xs font-semibold text-app-text-3 uppercase tracking-wider">Request Schema</h3>
          {#if formatSchema(ref.requestSchema)}
            <pre class="bg-app-panel border border-app-border rounded p-3 font-mono text-xs text-app-text-2 overflow-x-auto whitespace-pre">{formatSchema(ref.requestSchema)}</pre>
          {:else}
            <p class="text-xs text-app-text-4 italic">No schema available.</p>
          {/if}
        </div>
        <div class="flex flex-col gap-1">
          <h3 class="text-xs font-semibold text-app-text-3 uppercase tracking-wider">Response Schema</h3>
          {#if formatSchema(ref.responseSchema)}
            <pre class="bg-app-panel border border-app-border rounded p-3 font-mono text-xs text-app-text-2 overflow-x-auto whitespace-pre">{formatSchema(ref.responseSchema)}</pre>
          {:else}
            <p class="text-xs text-app-text-4 italic">No schema available.</p>
          {/if}
        </div>
      </div>
    </div>
  {:else}
    <p class="p-4 text-xs text-app-text-4">No template reference found for this request.</p>
  {/if}
</div>
