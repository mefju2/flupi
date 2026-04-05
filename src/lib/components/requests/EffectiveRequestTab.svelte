<script lang="ts">
  import { activeRequest } from '$lib/stores/requests';
  import { environments, activeEnvironment } from '$lib/stores/environment';
  import { resolveString, findUnresolved } from '$lib/services/variable-resolver';
  import { getMethodColor } from '$lib/utils/format';
  import type { CollectionData, AuthConfig } from '$lib/services/tauri-commands';
  import SectionHeader from '$lib/components/shared/SectionHeader.svelte';
  import SecretValue from '$lib/components/shared/SecretValue.svelte';

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

  // Keys that are secrets (so we can mask them in auth display)
  let secretKeys = $derived((() => {
    const envName = $activeEnvironment;
    if (!envName) return new Set<string>();
    const entry = $environments.find((e) => e.fileName === envName);
    if (!entry) return new Set<string>();
    return new Set(entry.environment.secrets);
  })());

  let req = $derived($activeRequest);

  // Merge headers respecting disabled lists
  let effectiveHeaders = $derived((() => {
    const disabledReq = new Set(req?.disabledHeaders ?? []);
    const disabledCol = new Set(req?.disabledCollectionHeaders ?? []);
    const base: Record<string, string> = {};
    for (const [k, v] of Object.entries(collection?.headers ?? {})) {
      if (!disabledCol.has(k)) base[k] = v;
    }
    const result = { ...base };
    for (const [k, v] of Object.entries(req?.headers ?? {})) {
      if (!disabledReq.has(k)) result[k] = v;
    }
    return result;
  })());

  // Resolve {param} placeholders in a path using pathParams values (values are also var-resolved)
  function resolvePathParams(path: string, pathParams: Record<string, string>): string {
    return path.replace(/(?<!\{)\{([a-zA-Z0-9_-]+)\}(?!\})/g, (match, key) => {
      if (key in pathParams) return resolveString(pathParams[key], vars);
      return match;
    });
  }

  // Find {param} placeholders with no entry in pathParams
  function findUnresolvedParams(path: string, pathParams: Record<string, string>): string[] {
    const unresolved: string[] = [];
    path.replace(/(?<!\{)\{([a-zA-Z0-9_-]+)\}(?!\})/g, (_, key) => {
      if (!(key in pathParams)) unresolved.push(`{${key}}`);
      return '';
    });
    return unresolved;
  }

  // Full URL: only prepend baseUrl when path doesn't already have a scheme or starts with {{
  let urlTemplate = $derived((() => {
    if (!req) return '';
    const path = req.path;
    const hasScheme = /^https?:\/\//i.test(path) || path.startsWith('{{');
    return hasScheme ? path : (collection?.baseUrl ?? '') + path;
  })());

  // Resolved URL: path params first, then {{vars}}
  let resolvedUrl = $derived((() => {
    if (!req) return '';
    const afterParams = resolvePathParams(urlTemplate, req.pathParams ?? {});
    return resolveString(afterParams, vars);
  })());

  let unresolvedInUrl = $derived((() => {
    if (!req) return [];
    const afterParams = resolvePathParams(urlTemplate, req.pathParams ?? {});
    const unresolvedVars = findUnresolved(afterParams, vars);
    const unresolvedParams = findUnresolvedParams(urlTemplate, req.pathParams ?? {});
    return [...unresolvedParams, ...unresolvedVars];
  })());

  // Effective auth: request auth overrides collection auth (unless 'inherit')
  let effectiveAuth = $derived((() => {
    const own = req?.auth;
    if (!own || own.type === 'inherit') return collection?.auth ?? ({ type: 'none' } as AuthConfig);
    return own;
  })());

  function r(val: string): string {
    return resolveString(val, vars);
  }

  function isSecret(key: string): boolean {
    return secretKeys.has(key);
  }

  function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  // Resolves {{vars}} to their actual values; leaves unresolved ones in amber
  function resolveAndHighlight(text: string): string {
    return escapeHtml(text).replace(
      /\{\{(\w+)\}\}/g,
      (match, key) => {
        if (key in vars) return escapeHtml(vars[key]);
        return `<span class="text-amber-400">${match}</span>`;
      }
    );
  }

  // Resolve path params then {{vars}} in a string, with amber unresolved highlighting
  function resolveAndHighlightFull(text: string, pathParams: Record<string, string> = {}): string {
    const afterParams = text.replace(/(?<!\{)\{([a-zA-Z0-9_-]+)\}(?!\})/g, (match, key) => {
      if (key in pathParams) return resolveString(pathParams[key], vars);
      return match;
    });
    return resolveAndHighlight(afterParams);
  }
</script>

<div class="p-4 space-y-5 text-sm h-full overflow-y-auto">
  {#if !req}
    <p class="text-app-text-4">Select a request to preview.</p>
  {:else}
    <!-- Resolved URL -->
    <section>
      <SectionHeader class="mb-2">Resolved URL</SectionHeader>
      <div class="flex items-center gap-2 bg-app-panel border border-app-border rounded px-3 py-2">
        <span class="font-semibold shrink-0 {getMethodColor(req.method)}">{req.method}</span>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        <span class="font-mono text-app-text break-all">{@html resolveAndHighlightFull(urlTemplate, req.pathParams ?? {})}</span>
      </div>
      {#if unresolvedInUrl.length > 0}
        <p class="text-xs text-amber-400 mt-1">Unresolved: {unresolvedInUrl.join(', ')}</p>
      {/if}
    </section>

    <!-- Effective Headers -->
    <section>
      <SectionHeader class="mb-2">Effective Headers</SectionHeader>
      {#if Object.keys(effectiveHeaders).length === 0}
        <p class="text-app-text-4 text-xs italic">No headers.</p>
      {:else}
        <div class="space-y-1">
          {#each Object.entries(effectiveHeaders) as [key, value]}
            {@const fromCollection = !!(collection?.headers[key]) && !req.headers[key]}
            <div class="flex gap-2 items-baseline bg-app-panel border border-app-border rounded px-2 py-1">
              <span class="font-mono text-app-text-2 shrink-0">{key}</span>
              <span class="text-app-text-4 shrink-0">:</span>
              {#if isSecret(key)}
                <SecretValue value={r(value)} class="text-xs" />
              {:else}
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                <span class="font-mono text-app-text break-all">{@html resolveAndHighlight(value)}</span>
              {/if}
              {#if fromCollection}
                <span class="text-xs text-app-text-4 ml-auto shrink-0">collection</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Auth -->
    <section>
      <SectionHeader class="mb-2">Auth</SectionHeader>
      <div class="bg-app-panel border border-app-border rounded px-3 py-2 font-mono text-app-text text-xs">
        {#if !effectiveAuth || effectiveAuth.type === 'none'}
          <span class="text-app-text-4">None</span>
        {:else if effectiveAuth.type === 'inherit'}
          <span class="text-app-text-4">Inherited from collection</span>
        {:else if effectiveAuth.type === 'bearer'}
          <span class="text-app-text-3">Bearer&nbsp;</span>
          <SecretValue value={r(effectiveAuth.token)} />
        {:else if effectiveAuth.type === 'basic'}
          <span class="text-app-text-3">Basic&nbsp;</span>
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <span>{@html resolveAndHighlight(effectiveAuth.username)}</span>
          <span class="text-app-text-4">&nbsp;/&nbsp;</span>
          <SecretValue value={r(effectiveAuth.password)} />
        {:else if effectiveAuth.type === 'apiKey'}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <span>{@html resolveAndHighlight(effectiveAuth.header)}</span>
          <span class="text-app-text-4">:&nbsp;</span>
          <SecretValue value={r(effectiveAuth.value)} />
        {:else if effectiveAuth.type === 'custom'}
          <div class="space-y-1">
            {#each Object.entries(effectiveAuth.headers) as [k, v]}
              <div class="flex gap-2">
                <span class="text-app-text-2 shrink-0">{k}</span>
                <span class="text-app-text-4">:</span>
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                <span class="text-app-text break-all">{@html resolveAndHighlight(v)}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <!-- Body -->
    {#if req.body && req.body.type !== 'none'}
      {@const bodyLabel = req.body.type === 'raw' ? `raw/${req.body.format}` : req.body.type}
      <section>
        <SectionHeader class="mb-2">Body ({bodyLabel})</SectionHeader>
        <div class="bg-app-panel border border-app-border rounded px-3 py-2 font-mono text-app-text text-xs whitespace-pre-wrap break-all max-h-48 overflow-y-auto">
          {#if req.body.type === 'raw'}
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html resolveAndHighlight(req.body.content)}
          {:else if req.body.type === 'form-urlencoded'}
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html resolveAndHighlight(Object.entries(req.body.content).map(([k, v]) => `${k}=${v}`).join('\n'))}
          {/if}
        </div>
      </section>
    {/if}

    <!-- Variable legend -->
    <p class="text-xs text-app-text-4">
      <span class="text-amber-400 font-mono">{'{{var}}'}</span> = unresolved
    </p>
  {/if}
</div>
