<script lang="ts">
  import type { TemplateRef } from '$lib/services/tauri-commands';

  interface Props {
    templateRef: TemplateRef;
  }

  let { templateRef }: Props = $props();

  function formatSchema(schema: unknown): string {
    if (schema === null || schema === undefined) return '';
    try {
      return JSON.stringify(schema, null, 2);
    } catch {
      return String(schema);
    }
  }

  let requestSchemaText = $derived(formatSchema(templateRef.requestSchema));
  let responseSchemaText = $derived(formatSchema(templateRef.responseSchema));
</script>

<div class="flex flex-col gap-4 p-3">
  <!-- Source info -->
  <div class="flex flex-col gap-0.5">
    <div class="flex items-center gap-2 text-xs text-app-text-3">
      <span class="font-mono">source:</span>
      <span class="font-mono text-app-text-3">{templateRef.sourceId}</span>
    </div>
    <div class="flex items-center gap-2 text-xs text-app-text-3">
      <span class="font-mono">operation:</span>
      <span class="font-mono text-app-text-3">{templateRef.operationId}</span>
    </div>
    <div class="flex items-center gap-2 text-xs text-app-text-3">
      <span class="font-mono">hash:</span>
      <span class="font-mono text-app-text-4">{templateRef.schemaHash}</span>
    </div>
  </div>

  <!-- Request Schema -->
  <div class="flex flex-col gap-1">
    <h3 class="text-xs font-semibold text-app-text-3 uppercase tracking-wider">Request Schema</h3>
    {#if requestSchemaText}
      <pre class="bg-app-bg border border-app-border rounded p-3 font-mono text-xs text-app-text-2 overflow-x-auto whitespace-pre">{requestSchemaText}</pre>
    {:else}
      <p class="text-xs text-app-text-4 italic">No schema available.</p>
    {/if}
  </div>

  <!-- Response Schema -->
  <div class="flex flex-col gap-1">
    <h3 class="text-xs font-semibold text-app-text-3 uppercase tracking-wider">Response Schema</h3>
    {#if responseSchemaText}
      <pre class="bg-app-bg border border-app-border rounded p-3 font-mono text-xs text-app-text-2 overflow-x-auto whitespace-pre">{responseSchemaText}</pre>
    {:else}
      <p class="text-xs text-app-text-4 italic">No schema available.</p>
    {/if}
  </div>
</div>
