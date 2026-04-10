<script lang="ts">
  import { diffText, type DiffLine } from "$lib/services/tauri-commands";

  interface Props {
    label: string;
    oldSchema: unknown;
    newSchema: unknown;
  }

  let { label, oldSchema, newSchema }: Props = $props();

  function toText(schema: unknown): string {
    if (schema === null || schema === undefined) return "";
    return JSON.stringify(schema, null, 2);
  }

  const oldText = $derived(toText(oldSchema));
  const newText = $derived(toText(newSchema));

  const bothEmpty = $derived(oldText === "" && newText === "");
  const unchanged = $derived(!bothEmpty && oldText === newText);

  let diff = $state<DiffLine[]>([]);

  $effect(() => {
    if (bothEmpty || unchanged) {
      diff = [];
      return;
    }
    diffText(oldText, newText).then((result) => {
      diff = result;
    });
  });

  const hasChanges = $derived(diff.some((l) => l.type !== "same"));
</script>

<div class="flex flex-col gap-1.5">
  <p class="text-xs font-semibold text-app-text-2 uppercase tracking-wider">
    {label}
  </p>

  {#if bothEmpty}
    <p class="text-xs text-app-text-4 italic">No schema</p>
  {:else if unchanged}
    <p class="text-xs text-app-text-4 italic">Unchanged</p>
  {:else if !hasChanges}
    <p class="text-xs text-app-text-4 italic">Unchanged</p>
  {:else}
    <div class="rounded-md border border-app-border bg-app-bg overflow-x-auto">
      <pre
        class="text-xs leading-5 font-mono p-3 whitespace-pre">{#each diff as line}<span
            class={line.type === "add"
              ? "block bg-green-950/40 text-green-300"
              : line.type === "remove"
                ? "block bg-red-950/40 text-red-400 line-through decoration-red-800/50"
                : "block text-app-text-3"}
            >{line.type === "add"
              ? "+ "
              : line.type === "remove"
                ? "- "
                : "  "}{line.text}</span
          >{/each}</pre>
    </div>
  {/if}
</div>
