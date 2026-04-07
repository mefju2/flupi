<script lang="ts">
  import type { BodyConfig } from "$lib/services/tauri-commands";
  import { generateBodyFromSchema } from "$lib/services/tauri-commands";
  import KeyValueTable from "$lib/components/shared/KeyValueTable.svelte";
  import JsonEditor from "$lib/components/shared/JsonEditor.svelte";
  import { ChevronDown, Check, Wand2, AlertTriangle } from "lucide-svelte";
  import { environments, activeEnvironment } from "$lib/stores/environment";
  import { functions } from "$lib/stores/functions";

  interface Props {
    body: BodyConfig | undefined;
    onUpdate: (body: BodyConfig) => void;
    requestSchema?: unknown;
  }

  let { body, onUpdate, requestSchema = undefined }: Props = $props();

  // --- Dropdown state ---
  let open = $state(false);

  // --- Per-type draft state (local only, lost on navigation) ---
  let drafts = $state({
    formUrlEncoded: {
      content: {} as Record<string, string>,
      disabledFields: [] as string[],
    },
    rawJson: "",
    rawXml: "",
    rawText: "",
  });

  // --- Grouped menu definition ---
  type BodySelection =
    | { type: "none" }
    | { type: "form-urlencoded" }
    | { type: "raw"; format: "json" | "xml" | "text" };

  const GROUPS: Array<{
    label: string;
    items: Array<{ label: string; sel: BodySelection }>;
  }> = [
    {
      label: "Form",
      items: [{ label: "Form URL Encoded", sel: { type: "form-urlencoded" } }],
    },
    {
      label: "Raw",
      items: [
        { label: "JSON", sel: { type: "raw", format: "json" } },
        { label: "XML", sel: { type: "raw", format: "xml" } },
        { label: "Text", sel: { type: "raw", format: "text" } },
      ],
    },
    {
      label: "Other",
      items: [{ label: "No Body", sel: { type: "none" } }],
    },
  ];

  // --- Selection label for the trigger button ---
  let selectionLabel = $derived.by(() => {
    if (!body || body.type === "none") return "No Body";
    if (body.type === "form-urlencoded") return "Form URL Encoded";
    if (body.type === "raw") {
      return { json: "JSON", xml: "XML", text: "Text" }[body.format];
    }
    return "No Body";
  });

  function isActive(sel: BodySelection): boolean {
    if (!body || body.type === "none") return sel.type === "none";
    if (sel.type === "none") return body.type === "none";
    if (sel.type === "form-urlencoded") return body.type === "form-urlencoded";
    if (sel.type === "raw")
      return body.type === "raw" && body.format === sel.format;
    return false;
  }

  // --- Draft save/restore ---
  function saveDraft(current: BodyConfig | undefined) {
    if (!current) return;
    if (current.type === "form-urlencoded") {
      drafts.formUrlEncoded = {
        content: current.content,
        disabledFields: current.disabledFields ?? [],
      };
    } else if (current.type === "raw") {
      if (current.format === "json") drafts.rawJson = current.content;
      else if (current.format === "xml") drafts.rawXml = current.content;
      else if (current.format === "text") drafts.rawText = current.content;
    }
  }

  function bodyForSelection(sel: BodySelection): BodyConfig {
    if (sel.type === "none") return { type: "none" };
    if (sel.type === "form-urlencoded") {
      return {
        type: "form-urlencoded",
        content: drafts.formUrlEncoded.content,
        disabledFields: drafts.formUrlEncoded.disabledFields,
      };
    }
    if (sel.type === "raw") {
      const content =
        sel.format === "json"
          ? drafts.rawJson
          : sel.format === "xml"
            ? drafts.rawXml
            : drafts.rawText;
      return { type: "raw", format: sel.format, content };
    }
    return { type: "none" };
  }

  function select(sel: BodySelection) {
    saveDraft(body);
    open = false;
    onUpdate(bodyForSelection(sel));
  }

  // --- Form table helpers ---
  function formRows(b: BodyConfig | undefined) {
    if (b?.type !== "form-urlencoded") return [];
    const disabled = b.disabledFields ?? [];
    return Object.entries(b.content).map(([key, value]) => ({
      id: key,
      key,
      value,
      enabled: !disabled.includes(key),
    }));
  }

  // Language for Monaco based on raw format
  const LANG_MAP = { json: "json", xml: "xml", text: "plaintext" } as const;

  // --- Generate from schema ---
  let confirmingGenerate = $state(false);

  // --- Active environment variables for token highlighting ---
  let activeVars = $derived.by(() => {
    const entry = $environments.find((e) => e.fileName === $activeEnvironment);
    if (!entry) return undefined;
    return { ...entry.environment.variables, ...entry.secrets };
  });

  let fnNames = $derived($functions.map((f) => f.name));
  let generating = $state(false);

  let hasGeneratableSchema = $derived(
    body?.type === "raw" && body.format === "json" && requestSchema != null,
  );

  async function handleGenerate() {
    if (generating) return;
    const isEmpty = body?.type === "raw" && body.content.trim() === "";
    if (isEmpty) {
      generating = true;
      try {
        const generated = await generateBodyFromSchema(requestSchema);
        onUpdate({ type: "raw", format: "json", content: generated });
      } catch (err) {
        console.error("[BodyTab] Failed to generate body from schema:", err);
      } finally {
        generating = false;
      }
    } else {
      confirmingGenerate = true;
    }
  }

  async function confirmGenerate() {
    if (generating) return;
    generating = true;
    try {
      const generated = await generateBodyFromSchema(requestSchema);
      onUpdate({ type: "raw", format: "json", content: generated });
    } catch (err) {
      console.error("[BodyTab] Failed to generate body from schema:", err);
    } finally {
      generating = false;
      confirmingGenerate = false;
    }
  }

  function cancelGenerate() {
    confirmingGenerate = false;
  }

  $effect(() => {
    // Reset confirmation state when body type changes away from raw/json
    if (!hasGeneratableSchema) confirmingGenerate = false;
  });
</script>

<div class="flex flex-col h-full">
  <!-- Type selector bar -->
  <div
    class="flex items-center gap-2 px-4 py-3 border-b border-app-border shrink-0"
  >
    <span class="text-xs text-app-text-3">Body</span>

    <div class="relative">
      <!-- Trigger -->
      <button
        class="flex items-center gap-1 text-xs px-2 py-1 bg-app-card border border-app-border-2 rounded hover:bg-app-hover transition-colors text-app-text-2"
        onclick={() => (open = !open)}
      >
        {selectionLabel}
        <ChevronDown size={12} class="text-app-text-4" />
      </button>

      {#if open}
        <!-- Backdrop to close on outside click -->
        <div class="fixed inset-0 z-40" onclick={() => (open = false)}></div>

        <!-- Dropdown panel -->
        <div
          class="absolute top-full left-0 mt-1 z-50 bg-app-panel border border-app-border rounded shadow-lg min-w-[180px]"
        >
          {#each GROUPS as group}
            <div
              class="px-3 pt-2 pb-1 text-[10px] font-semibold text-app-text-4 uppercase tracking-wide"
            >
              {group.label}
            </div>
            {#each group.items as item}
              <button
                class="flex items-center justify-between w-full px-3 py-1.5 text-xs text-left transition-colors
                  {isActive(item.sel)
                  ? 'text-app-text bg-app-hover'
                  : 'text-app-text-2 hover:bg-app-card'}"
                onclick={() => select(item.sel)}
              >
                {item.label}
                {#if isActive(item.sel)}
                  <Check size={11} class="text-cyan-400 shrink-0" />
                {/if}
              </button>
            {/each}
          {/each}
          <div class="pb-1"></div>
        </div>
      {/if}
    </div>

    {#if hasGeneratableSchema}
      <div class="ml-auto flex items-center gap-2">
        {#if confirmingGenerate}
          <span class="flex items-center gap-1 text-xs text-amber-400">
            <AlertTriangle size={11} />
            This will replace the current body.
          </span>
          <button
            class="text-xs px-2 py-1 bg-amber-500/20 border border-amber-500/40 text-amber-300 rounded hover:bg-amber-500/30 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            onclick={confirmGenerate}
            disabled={generating}
          >
            {generating ? "Generating…" : "Replace body"}
          </button>
          <button
            class="text-xs px-2 py-1 bg-app-card border border-app-border-2 text-app-text-2 rounded hover:bg-app-hover transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            onclick={cancelGenerate}
            disabled={generating}
          >
            Cancel
          </button>
        {:else}
          <button
            class="flex items-center gap-1.5 text-xs px-2 py-1 bg-app-card border border-app-border-2 rounded hover:bg-app-hover transition-colors text-app-text-2 disabled:opacity-40 disabled:cursor-not-allowed"
            onclick={handleGenerate}
            disabled={generating}
          >
            <Wand2 size={11} class="text-cyan-400" />
            {generating ? "Generating…" : "Generate from schema"}
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Content area -->
  {#if !body || body.type === "none"}
    <div class="flex-1 overflow-y-auto px-4 py-3">
      <p class="text-sm text-app-text-3">No request body.</p>
    </div>
  {:else if body.type === "form-urlencoded"}
    <div class="flex-1 overflow-y-auto px-4 py-3 space-y-3">
      <p class="text-xs text-app-text-4">
        Encoded as <span class="font-mono"
          >application/x-www-form-urlencoded</span
        >
      </p>
      <KeyValueTable
        rows={formRows(body)}
        showEnabled={true}
        onUpdate={(rows) => {
          const c: Record<string, string> = {};
          const disabled: string[] = [];
          for (const r of rows) {
            if (r.key) {
              c[r.key] = r.value;
              if (r.enabled === false) disabled.push(r.key);
            }
          }
          onUpdate({
            type: "form-urlencoded",
            content: c,
            disabledFields: disabled,
          });
        }}
      />
    </div>
  {:else if body.type === "raw"}
    <div class="flex-1 overflow-hidden">
      <JsonEditor
        value={body.content}
        language={LANG_MAP[body.format]}
        onChange={(v) =>
          onUpdate({ type: "raw", format: body.format, content: v })}
        schema={body.format === "json" ? requestSchema : undefined}
        variables={body.format === "json" ? activeVars : undefined}
        functionNames={body.format === "json" ? fnNames : undefined}
      />
    </div>
  {/if}
</div>
