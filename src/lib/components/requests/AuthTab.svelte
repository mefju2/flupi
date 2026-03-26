<script lang="ts">
  import type { AuthConfig } from '$lib/services/tauri-commands';
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';

  interface Props {
    auth: AuthConfig | undefined;
    onUpdate: (auth: AuthConfig) => void;
  }

  let { auth, onUpdate }: Props = $props();

  let authType = $derived(auth?.type ?? 'inherit');

  function setType(type: AuthConfig['type']) {
    if (type === 'none') onUpdate({ type: 'none' });
    else if (type === 'inherit') onUpdate({ type: 'inherit' });
    else if (type === 'bearer') onUpdate({ type: 'bearer', token: '' });
    else if (type === 'basic') onUpdate({ type: 'basic', username: '', password: '' });
    else if (type === 'apiKey') onUpdate({ type: 'apiKey', header: 'X-API-Key', value: '' });
    else if (type === 'custom') onUpdate({ type: 'custom', headers: {} });
  }

  function customRows(a: AuthConfig | undefined) {
    if (a?.type !== 'custom') return [];
    return Object.entries(a.headers).map(([key, value]) => ({ key, value }));
  }
</script>

<div class="p-4 space-y-4">
  <div>
    <label for="auth-type" class="text-xs text-zinc-400 block mb-1">Auth Type</label>
    <select
      id="auth-type"
      class="bg-zinc-800 border border-zinc-700 text-zinc-100 text-sm px-2 py-1.5 rounded focus:outline-none focus:border-zinc-500"
      value={authType}
      onchange={(e) => setType(e.currentTarget.value as AuthConfig['type'])}
    >
      <option value="inherit">Inherit from collection</option>
      <option value="none">None</option>
      <option value="bearer">Bearer Token</option>
      <option value="basic">Basic Auth</option>
      <option value="apiKey">API Key</option>
      <option value="custom">Custom Headers</option>
    </select>
  </div>

  {#if auth?.type === 'none'}
    <p class="text-sm text-zinc-500">No authentication.</p>
  {:else if auth?.type === 'inherit'}
    <p class="text-sm text-zinc-500">Authentication is inherited from the parent collection.</p>
  {:else if auth?.type === 'bearer'}
    <div>
      <label for="auth-bearer-token" class="text-xs text-zinc-400 block mb-1">Token</label>
      <textarea
        id="auth-bearer-token"
        class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500 min-h-[60px] resize-y"
        value={auth.token}
        placeholder="Bearer token..."
        oninput={(e) => onUpdate({ type: 'bearer', token: e.currentTarget.value })}
      ></textarea>
    </div>
  {:else if auth?.type === 'basic'}
    <div class="space-y-2">
      <div>
        <label for="auth-basic-username" class="text-xs text-zinc-400 block mb-1">Username</label>
        <input
          id="auth-basic-username"
          class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
          value={auth.username}
          placeholder="username"
          oninput={(e) => onUpdate({ type: 'basic', username: e.currentTarget.value, password: (auth as Extract<AuthConfig, {type:'basic'}>).password })}
        />
      </div>
      <div>
        <label for="auth-basic-password" class="text-xs text-zinc-400 block mb-1">Password</label>
        <input
          id="auth-basic-password"
          type="password"
          class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
          value={auth.password}
          placeholder="password"
          oninput={(e) => onUpdate({ type: 'basic', username: (auth as Extract<AuthConfig, {type:'basic'}>).username, password: e.currentTarget.value })}
        />
      </div>
    </div>
  {:else if auth?.type === 'apiKey'}
    <div class="space-y-2">
      <div>
        <label for="auth-apikey-header" class="text-xs text-zinc-400 block mb-1">Header Name</label>
        <input
          id="auth-apikey-header"
          class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
          value={auth.header}
          placeholder="X-API-Key"
          oninput={(e) => onUpdate({ type: 'apiKey', header: e.currentTarget.value, value: (auth as Extract<AuthConfig, {type:'apiKey'}>).value })}
        />
      </div>
      <div>
        <label for="auth-apikey-value" class="text-xs text-zinc-400 block mb-1">Value</label>
        <input
          id="auth-apikey-value"
          class="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500"
          value={auth.value}
          placeholder="api key value"
          oninput={(e) => onUpdate({ type: 'apiKey', header: (auth as Extract<AuthConfig, {type:'apiKey'}>).header, value: e.currentTarget.value })}
        />
      </div>
    </div>
  {:else if auth?.type === 'custom'}
    <div>
      <span class="text-xs text-zinc-400 block mb-2">Custom Headers</span>
      <KeyValueTable
        rows={customRows(auth)}
        onUpdate={(rows) => {
          const h: Record<string, string> = {};
          for (const r of rows) { if (r.key) h[r.key] = r.value; }
          onUpdate({ type: 'custom', headers: h });
        }}
      />
    </div>
  {/if}
</div>
