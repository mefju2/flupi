<script lang="ts">
  import { activeRequest, activeRequestId } from '$lib/stores/requests';
  import { project } from '$lib/stores/project';
  import { saveRequest, type AuthConfig, type BodyConfig } from '$lib/services/tauri-commands';
  import { createDebouncedSave } from '$lib/services/debounced-save';
  import ParamsTab from './ParamsTab.svelte';
  import HeadersTab from './HeadersTab.svelte';
  import AuthTab from './AuthTab.svelte';
  import BodyTab from './BodyTab.svelte';
  import ResponsePanel from './ResponsePanel.svelte';

  const METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
  const TABS = ['Params', 'Headers', 'Auth', 'Body'] as const;
  type Tab = (typeof TABS)[number];

  let activeTab = $state<Tab>('Params');

  const debouncedSave = createDebouncedSave(async () => {
    const req = $activeRequest;
    const id = $activeRequestId;
    const path = $project.path;
    if (!req || !id || !path) return;
    await saveRequest(path, id, req);
  });

  function updateRequest(patch: Partial<typeof $activeRequest>) {
    if (!$activeRequest) return;
    activeRequest.set({ ...$activeRequest, ...patch });
    debouncedSave.trigger();
  }

  function methodColor(method: string): string {
    const m: Record<string, string> = {
      GET: 'text-emerald-400',
      POST: 'text-cyan-400',
      PUT: 'text-yellow-400',
      PATCH: 'text-orange-400',
      DELETE: 'text-red-400',
      HEAD: 'text-purple-400',
      OPTIONS: 'text-zinc-400',
    };
    return m[method] ?? 'text-zinc-400';
  }
</script>

<div class="flex flex-col h-full bg-zinc-950">
  {#if !$activeRequest}
    <div class="flex-1 flex items-center justify-center text-zinc-600 text-sm">
      Select a request to edit.
    </div>
  {:else}
    <!-- URL Bar -->
    <div class="flex items-center gap-2 px-4 py-3 border-b border-zinc-800">
      <select
        class="bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm font-semibold {methodColor($activeRequest.method)} focus:outline-none focus:border-zinc-500 shrink-0"
        value={$activeRequest.method}
        onchange={(e) => updateRequest({ method: e.currentTarget.value })}
      >
        {#each METHODS as m}
          <option value={m}>{m}</option>
        {/each}
      </select>

      <input
        class="flex-1 bg-zinc-800 border border-zinc-700 rounded px-3 py-1.5 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
        value={$activeRequest.path}
        placeholder="/api/endpoint"
        oninput={(e) => updateRequest({ path: e.currentTarget.value })}
      />

      <button
        class="px-4 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white text-sm rounded transition-colors shrink-0"
        onclick={() => console.log('Send not implemented yet')}
      >Send</button>
    </div>

    <!-- Tab Bar -->
    <div class="flex border-b border-zinc-800 px-4">
      {#each TABS as tab}
        <button
          class="text-sm px-3 py-2 transition-colors border-b-2 -mb-px {activeTab === tab ? 'text-cyan-400 border-cyan-500' : 'text-zinc-500 border-transparent hover:text-zinc-300'}"
          onclick={() => (activeTab = tab)}
        >{tab}</button>
      {/each}
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-y-auto">
      {#if activeTab === 'Params'}
        <ParamsTab
          path={$activeRequest.path}
          onPathChange={(p) => updateRequest({ path: p })}
        />
      {:else if activeTab === 'Headers'}
        <HeadersTab
          headers={$activeRequest.headers}
          onUpdate={(h) => updateRequest({ headers: h })}
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
      {/if}
    </div>

    <!-- Response Panel -->
    <ResponsePanel />
  {/if}
</div>
