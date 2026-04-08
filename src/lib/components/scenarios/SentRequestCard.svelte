<script lang="ts">
  import type {
    SentRequest,
    SentRequestBodyTyped,
  } from "$lib/services/tauri-commands";

  interface Props {
    sentRequest: SentRequest;
  }

  let { sentRequest }: Props = $props();

  let expanded = $state(false);

  function bodyText(
    body: SentRequestBodyTyped | null | undefined,
  ): string | null {
    if (!body) return null;
    if (body.type === "json") return JSON.stringify(body.content, null, 2);
    if (body.type === "form")
      return Object.entries(body.content)
        .map(([k, v]) => `${k}=${v}`)
        .join("\n");
    return body.content;
  }
</script>

<div>
  <div class="flex items-center justify-between mb-1">
    <p class="text-xs text-app-text-3">Sent Request</p>
    <button
      class="text-xs text-app-text-4 hover:text-app-text-2 transition-colors"
      onclick={() => (expanded = !expanded)}
      >{expanded ? "Collapse" : "Expand"}</button
    >
  </div>

  {#if expanded}
    {@const rawBody = bodyText(sentRequest.body)}
    <div class="bg-app-card rounded p-2 space-y-2">
      <div class="flex items-center gap-2 min-w-0">
        <span class="font-mono text-xs text-cyan-400 shrink-0"
          >{sentRequest.method}</span
        >
        <span class="font-mono text-xs text-app-text-2 truncate"
          >{sentRequest.url}</span
        >
      </div>

      {#if Object.keys(sentRequest.headers).length > 0}
        <div>
          <p class="text-xs text-app-text-4 mb-1">Headers</p>
          <div class="space-y-0.5">
            {#each Object.entries(sentRequest.headers) as [k, v]}
              <div class="flex gap-2 min-w-0">
                <span class="font-mono text-xs text-app-text-3 shrink-0"
                  >{k}:</span
                >
                <span class="font-mono text-xs text-app-text-2 truncate"
                  >{v}</span
                >
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if rawBody !== null}
        <div>
          <p class="text-xs text-app-text-4 mb-1">Request Body</p>
          <pre
            class="font-mono text-xs text-app-text-2 overflow-auto max-h-48 whitespace-pre-wrap break-all">{rawBody}</pre>
        </div>
      {/if}
    </div>
  {:else}
    <button
      class="w-full text-left font-mono text-xs text-app-text-3 bg-app-card rounded p-2 hover:text-app-text-2 transition-colors truncate"
      onclick={() => (expanded = true)}
      >{sentRequest.method} {sentRequest.url}</button
    >
  {/if}
</div>
