<script lang="ts">
  import type { SentRequest, SentRequestBodyTyped } from '$lib/services/tauri-commands';

  interface Props {
    sentRequest: SentRequest;
  }

  let { sentRequest }: Props = $props();

  let headersOpen = $state(false);

  function formatSentBody(body: SentRequestBodyTyped): string {
    if (body.type === 'json') return JSON.stringify(body.content, null, 2);
    if (body.type === 'form')
      return Object.entries(body.content).map(([k, v]) => `${k}=${v}`).join('\n');
    return body.content;
  }
</script>

<div class="p-4 space-y-3">
  <div class="flex items-center gap-2 min-w-0">
    <span class="font-mono text-xs text-cyan-400 shrink-0">{sentRequest.method}</span>
    <span class="font-mono text-xs text-app-text-2 break-all">{sentRequest.url}</span>
  </div>

  {#if Object.keys(sentRequest.headers).length > 0}
    <div>
      <button
        class="text-xs text-app-text-3 hover:text-app-text hover:bg-app-card rounded px-2 transition-colors flex items-center gap-1 mb-1 -mx-2"
        onclick={() => (headersOpen = !headersOpen)}
      >
        <span>{headersOpen ? '▾' : '▸'}</span>
        Headers ({Object.keys(sentRequest.headers).length})
      </button>
      {#if headersOpen}
        <div class="space-y-0.5">
          {#each Object.entries(sentRequest.headers) as [k, v]}
            <div class="flex gap-2 text-xs font-mono">
              <span class="text-app-text-3 shrink-0">{k}:</span>
              <span class="text-app-text-2 break-all">{v}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if sentRequest.body}
    {@const sentBodyText = formatSentBody(sentRequest.body)}
    <div>
      <p class="text-xs text-app-text-3 mb-1">Body</p>
      <pre class="text-xs font-mono text-app-text bg-app-panel border border-app-border p-3 overflow-auto whitespace-pre-wrap break-all">{sentBodyText}</pre>
    </div>
  {/if}
</div>
