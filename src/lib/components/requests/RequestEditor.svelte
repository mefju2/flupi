<script lang="ts">
  import { onMount } from 'svelte';
  import { activeRequest, activeRequestId, activeCollection } from '$lib/stores/requests';
  import { project } from '$lib/stores/project';
  import { activeEnvironment } from '$lib/stores/environment';
  import { isExecuting, lastResponse, lastError } from '$lib/stores/execution';
  import { driftedRequestIds } from '$lib/stores/openapi';
  import { saveRequest, sendRequest, type AuthConfig, type BodyConfig } from '$lib/services/tauri-commands';
  import { createDebouncedSave } from '$lib/services/debounced-save';
  import { getMethodColor } from '$lib/utils/format';
  import ParamsTab from './ParamsTab.svelte';
  import PathParamsTab from './PathParamsTab.svelte';
  import HeadersTab from './HeadersTab.svelte';
  import AuthTab from './AuthTab.svelte';
  import BodyTab from './BodyTab.svelte';
  import SchemaTab from './SchemaTab.svelte';
  import EffectiveRequestTab from './EffectiveRequestTab.svelte';
  import DriftPanel from '$lib/components/openapi/DriftPanel.svelte';
  import ExtractionsPanel from '$lib/components/shared/ExtractionsPanel.svelte';
  import ResponsePanel from './ResponsePanel.svelte';
  import VariableAutocomplete from '$lib/components/shared/VariableAutocomplete.svelte';
  import type { Extraction } from '$lib/services/tauri-commands';

  const METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
  type Tab = 'Params' | 'Path' | 'Headers' | 'Auth' | 'Body' | 'Extractions' | 'Schema' | 'Effective Request';

  let activeTab = $state<Tab>('Params');

  // Tabs that are always present.
  const BASE_TABS: Tab[] = ['Params', 'Path', 'Headers', 'Auth', 'Body', 'Extractions'];

  // Schema tab only for template-derived requests; Effective Request always last.
  let visibleTabs = $derived<Tab[]>([
    ...BASE_TABS,
    ...($activeRequest?.templateRef ? ['Schema' as Tab] : []),
    'Effective Request',
  ]);

  // When the active request changes: reset to Params, but go straight to Schema if drifted.
  $effect(() => {
    const id = $activeRequestId;
    if (id && $driftedRequestIds.has(id) && $activeRequest?.templateRef) {
      activeTab = 'Schema';
    } else {
      activeTab = 'Params';
    }
  });

  const debouncedSave = createDebouncedSave(async () => {
    const req = $activeRequest;
    const id = $activeRequestId;
    const path = $project.path;
    if (!req || !id || !path) return;
    await saveRequest(path, id, req);
  });

  $effect(() => {
    const req = $activeRequest;
    if (!req) return;
    const matches = [...req.path.matchAll(/(?<!\{)\{([a-zA-Z0-9_-]+)\}(?!\})/g)];
    const detected = [...new Set(matches.map((m) => m[1]))];
    const current = req.pathParams ?? {};
    const hasNew = detected.some((p) => !(p in current));
    const hasRemoved = Object.keys(current).some((p) => !detected.includes(p));
    if (!hasNew && !hasRemoved) return;
    const next: Record<string, string> = {};
    for (const param of detected) {
      next[param] = current[param] ?? `{{${param}}}`;
    }
    activeRequest.set({ ...req, pathParams: next });
    debouncedSave.trigger();
  });

  onMount(() => {
    const onSend = () => handleSend();
    const onSave = () => { debouncedSave.flush(); };
    window.addEventListener('flupi:send-request', onSend);
    window.addEventListener('flupi:save', onSave);
    return () => {
      window.removeEventListener('flupi:send-request', onSend);
      window.removeEventListener('flupi:save', onSave);
    };
  });

  function updateRequest(patch: Partial<typeof $activeRequest>) {
    if (!$activeRequest) return;
    activeRequest.set({ ...$activeRequest, ...patch });
    debouncedSave.trigger();
  }

  async function handleSend() {
    const id = $activeRequestId;
    const path = $project.path;
    const env = $activeEnvironment ?? '';
    if (!id || !path || $isExecuting) return;

    isExecuting.set(true);
    lastResponse.set(null);
    lastError.set(null);
    try {
      const response = await sendRequest(path, id, env);
      lastResponse.set(response);
    } catch (e) {
      lastError.set(typeof e === 'string' ? e : (e instanceof Error ? e.message : 'Request failed'));
    } finally {
      isExecuting.set(false);
    }
  }
</script>

<div class="flex flex-col h-full bg-app-bg">
  {#if !$activeRequest}
    <div class="flex-1 flex items-center justify-center text-app-text-4 text-sm">
      Select a request to edit.
    </div>
  {:else}
    <!-- URL Bar -->
    <div class="flex items-center gap-2 px-4 py-3 border-b border-app-border">
      <select
        class="bg-app-card border border-app-border-2 rounded px-2 py-1.5 text-sm font-semibold {getMethodColor($activeRequest.method)} focus:outline-none focus:border-app-hover shrink-0"
        value={$activeRequest.method}
        onchange={(e) => updateRequest({ method: e.currentTarget.value })}
      >
        {#each METHODS as m}
          <option value={m}>{m}</option>
        {/each}
      </select>

      <VariableAutocomplete
        class="flex-1"
        value={$activeRequest.path}
        placeholder="/api/endpoint"
        onChange={(v) => updateRequest({ path: v })}
      />

      <button
        class="px-4 py-1.5 bg-cyan-600 hover:bg-cyan-500 disabled:bg-app-hover disabled:text-app-text-3 disabled:cursor-not-allowed text-white text-sm rounded transition-colors shrink-0"
        onclick={handleSend}
        disabled={$isExecuting}
        title="Send (Ctrl+Enter)"
      >{$isExecuting ? 'Sending…' : 'Send'}</button>
    </div>

    <!-- Tab Bar -->
    <div class="flex border-b border-app-border px-4">
      {#each visibleTabs as tab}
        {@const isDriftTab = tab === 'Schema' && !!$activeRequestId && $driftedRequestIds.has($activeRequestId)}
        <button
          class="text-sm px-3 py-2 transition-all duration-150 border-b-2 -mb-px {activeTab === tab ? 'text-cyan-300 border-cyan-500' : 'text-app-text-3 border-transparent hover:text-app-text-2'}"
          onclick={() => (activeTab = tab)}
        >
          {tab}
          {#if isDriftTab}
            <span class="inline-block w-1.5 h-1.5 rounded-full bg-red-500 ml-1 mb-0.5" title="Drift detected"></span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-y-auto">
      {#if activeTab === 'Params'}
        <ParamsTab
          path={$activeRequest.path}
          onPathChange={(p) => updateRequest({ path: p })}
        />
      {:else if activeTab === 'Path'}
        <PathParamsTab
          path={$activeRequest.path}
          pathParams={$activeRequest.pathParams ?? {}}
          onPathParamsChange={(params) => updateRequest({ pathParams: params })}
        />
      {:else if activeTab === 'Headers'}
        <HeadersTab
          headers={$activeRequest.headers}
          disabledHeaders={$activeRequest.disabledHeaders ?? []}
          collectionHeaders={$activeCollection?.headers}
          disabledCollectionHeaders={$activeRequest.disabledCollectionHeaders ?? []}
          onUpdate={(h, disabled) => updateRequest({ headers: h, disabledHeaders: disabled })}
          onDisabledCollectionHeadersChange={(disabled) => updateRequest({ disabledCollectionHeaders: disabled })}
        />
      {:else if activeTab === 'Auth'}
        <AuthTab
          auth={$activeRequest.auth}
          onUpdate={(a: AuthConfig) => updateRequest({ auth: a })}
        />
      {:else if activeTab === 'Body'}
        <BodyTab
          body={$activeRequest.body}
          onUpdate={(b: BodyConfig) => updateRequest({ body: b })}
        />
      {:else if activeTab === 'Extractions'}
        <div class="p-4">
          <ExtractionsPanel
            extractions={$activeRequest.extractions ?? []}
            onUpdate={(extractions: Extraction[]) => updateRequest({ extractions })}
            responseSchema={$activeRequest.templateRef?.responseSchema ?? null}
            unknownVariableLabel="not in env"
          />
        </div>
      {:else if activeTab === 'Schema' && $activeRequest.templateRef}
        {#if $activeRequestId && $driftedRequestIds.has($activeRequestId)}
          <DriftPanel requestId={$activeRequestId} onDone={() => { activeTab = 'Schema'; }} />
        {:else}
          <SchemaTab templateRef={$activeRequest.templateRef} />
        {/if}
      {:else if activeTab === 'Effective Request'}
        <EffectiveRequestTab collection={$activeCollection} />
      {/if}
    </div>

    <!-- Response Panel -->
    <ResponsePanel />
  {/if}
</div>
