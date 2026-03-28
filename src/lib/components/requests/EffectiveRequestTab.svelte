<script lang="ts">
  import { activeRequest } from '$lib/stores/requests';
  import { environments, activeEnvironment } from '$lib/stores/environment';
  import { resolveString, findUnresolved } from '$lib/services/variable-resolver';
  import { getMethodColor } from '$lib/utils/format';
  import type { CollectionData, AuthConfig } from '$lib/services/tauri-commands';

  interface Props {
    collection?: CollectionData | null;
  }

  let { collection = null }: Props = $props();

  // Get resolved variables for the active environment
  let vars = $derived((() => {
    const envName = $activeEnvironment;
    if (!envName) return {} as Record<string, string>;
    const entry = $environments.find((e) => e.fileName === envName);
    if (!entry) return {} as Record<string, string>;
    return { ...entry.environment.variables, ...entry.secrets };
  })());

  let req = $derived($activeRequest);

  // Merge headers: collection headers + request headers (request wins)
  let effectiveHeaders = $derived((() => {
    const base: Record<string, string> = { ...(collection?.headers ?? {}) };
    const own = req?.headers ?? {};
    return { ...base, ...own };
  })());

  // Full URL template (baseUrl + path) before resolution
  let urlTemplate = $derived(req ? (collection?.baseUrl ?? '') + req.path : '');

  let unresolvedInUrl = $derived(req ? findUnresolved(urlTemplate, vars) : []);

  // Effective auth: request auth overrides collection auth (unless 'inherit')
  let effectiveAuth = $derived((() => {
    const own = req?.auth;
    if (!own || own.type === 'inherit') return collection?.auth ?? ({ type: 'none' } as AuthConfig);
    return own;
  })());

  function authLabel(auth: AuthConfig | undefined): string {
    if (!auth) return 'None';
    if (auth.type === 'bearer') return `Bearer ${resolveString(auth.token, vars)}`;
    if (auth.type === 'basic') return `Basic (${resolveString(auth.username, vars)})`;
    if (auth.type === 'apiKey') return `${resolveString(auth.header, vars)}: ••••`;
    if (auth.type === 'custom') return `Custom (${Object.keys(auth.headers).length} headers)`;
    if (auth.type === 'inherit') return 'Inherited from collection';
    return 'None';
  }

  function highlightVars(text: string): string {
    return text.replace(
      /\{\{(\w+)\}\}/g,
      (match, key) => key in vars
        ? `<span class="text-cyan-400">${match}</span>`
        : `<span class="text-amber-400">${match}</span>`
    );
  }
</script>

<div class="p-4 space-y-5 text-sm">
  {#if !req}
    <p class="text-app-text-4">Select a request to preview.</p>
  {:else}
    <!-- Resolved URL -->
    <section>
      <p class="text-xs text-app-text-3 uppercase tracking-wider mb-2">Resolved URL</p>
      <div class="flex items-center gap-2 bg-app-panel border border-app-border rounded px-3 py-2">
        <span class="font-semibold shrink-0 {getMethodColor(req.method)}">{req.method}</span>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        <span class="font-mono text-app-text break-all">{@html highlightVars(urlTemplate)}</span>
      </div>
      {#if unresolvedInUrl.length > 0}
        <p class="text-xs text-amber-400 mt-1">Unresolved: {unresolvedInUrl.join(', ')}</p>
      {/if}
    </section>

    <!-- Effective Headers -->
    <section>
      <p class="text-xs text-app-text-3 uppercase tracking-wider mb-2">Effective Headers</p>
      {#if Object.keys(effectiveHeaders).length === 0}
        <p class="text-app-text-4 text-xs italic">No headers.</p>
      {:else}
        <div class="space-y-1">
          {#each Object.entries(effectiveHeaders) as [key, value]}
            {@const isCollection = !!(collection?.headers[key]) && !req.headers[key]}
            <div class="flex gap-2 items-baseline bg-app-panel border border-app-border rounded px-2 py-1">
              <span class="font-mono text-app-text-2 shrink-0">{key}</span>
              <span class="text-app-text-4 shrink-0">:</span>
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              <span class="font-mono text-app-text break-all">{@html highlightVars(value)}</span>
              {#if isCollection}
                <span class="text-xs text-app-text-4 ml-auto shrink-0">collection</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Auth -->
    <section>
      <p class="text-xs text-app-text-3 uppercase tracking-wider mb-2">Auth</p>
      <div class="bg-app-panel border border-app-border rounded px-3 py-2 font-mono text-app-text">
        {authLabel(effectiveAuth)}
      </div>
    </section>

    <!-- Body -->
    {#if req.body && req.body.type !== 'none'}
      <section>
        <p class="text-xs text-app-text-3 uppercase tracking-wider mb-2">Body ({req.body.type})</p>
        <div class="bg-app-panel border border-app-border rounded px-3 py-2 font-mono text-app-text text-xs whitespace-pre-wrap break-all max-h-48 overflow-y-auto">
          {#if req.body.type === 'json'}
            {typeof req.body.content === 'string' ? req.body.content : JSON.stringify(req.body.content, null, 2)}
          {:else if req.body.type === 'raw'}
            {req.body.content}
          {:else if req.body.type === 'form'}
            {Object.entries(req.body.content).map(([k, v]) => `${k}=${v}`).join('\n')}
          {/if}
        </div>
      </section>
    {/if}

    <!-- Variable legend -->
    <p class="text-xs text-app-text-4">
      <span class="text-cyan-400 font-mono">{'{{var}}'}</span> = resolved &nbsp;
      <span class="text-amber-400 font-mono">{'{{var}}'}</span> = unresolved
    </p>
  {/if}
</div>
