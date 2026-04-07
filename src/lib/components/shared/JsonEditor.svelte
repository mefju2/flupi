<script lang="ts">
  import { onMount } from "svelte";
  import * as monaco from "monaco-editor";
  import { theme } from "$lib/stores/ui";
  import { ensureMonacoEnv } from "$lib/services/monaco-env";
  import {
    completionContexts,
    registerCompletionProviders,
  } from "./json-editor-completions";

  // Register themes once per module lifetime
  let _themesRegistered = false;
  function registerThemes() {
    if (_themesRegistered) return;
    _themesRegistered = true;

    monaco.editor.defineTheme("flupi-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [
        { token: "string.value.json", foreground: "86efac" },
        { token: "string.key.json", foreground: "f4f4f5" },
        { token: "number.json", foreground: "7dd3fc" },
        { token: "keyword.json", foreground: "c084fc" },
        { token: "delimiter.bracket.json", foreground: "71717a" },
        { token: "delimiter.array.json", foreground: "71717a" },
        { token: "delimiter.colon.json", foreground: "71717a" },
        { token: "delimiter.comma.json", foreground: "71717a" },
      ],
      colors: {
        "editor.background": "#18181b",
        "editor.foreground": "#f4f4f5",
        "editor.lineHighlightBackground": "#27272a",
        "editorLineNumber.foreground": "#52525b",
        "editorLineNumber.activeForeground": "#a1a1aa",
        "editor.selectionBackground": "#3f3f46",
        "editor.inactiveSelectionBackground": "#3f3f4680",
        "editorCursor.foreground": "#06b6d4",
        "editorIndentGuide.background1": "#27272a",
        "editorIndentGuide.activeBackground1": "#3f3f46",
        "editorWidget.background": "#27272a",
        "editorWidget.border": "#3f3f46",
        "editorSuggestWidget.background": "#27272a",
        "editorSuggestWidget.border": "#3f3f46",
        "editorSuggestWidget.selectedBackground": "#3f3f46",
        "scrollbarSlider.background": "#3f3f4660",
        "scrollbarSlider.hoverBackground": "#3f3f46a0",
        "scrollbarSlider.activeBackground": "#52525b",
      },
    });

    monaco.editor.defineTheme("flupi-light", {
      base: "vs",
      inherit: true,
      rules: [
        { token: "string.value.json", foreground: "166534" },
        { token: "string.key.json", foreground: "030712" },
        { token: "number.json", foreground: "1d4ed8" },
        { token: "keyword.json", foreground: "7e22ce" },
        { token: "delimiter.bracket.json", foreground: "6b7280" },
        { token: "delimiter.array.json", foreground: "6b7280" },
        { token: "delimiter.colon.json", foreground: "6b7280" },
        { token: "delimiter.comma.json", foreground: "6b7280" },
      ],
      colors: {
        "editor.background": "#f9fafb",
        "editor.foreground": "#030712",
        "editor.lineHighlightBackground": "#f3f4f6",
        "editorLineNumber.foreground": "#9ca3af",
        "editorLineNumber.activeForeground": "#6b7280",
        "editor.selectionBackground": "#e5e7eb",
        "editorCursor.foreground": "#0891b2",
        "editorIndentGuide.background1": "#e5e7eb",
        "editorIndentGuide.activeBackground1": "#d1d5db",
        "editorWidget.background": "#f9fafb",
        "editorWidget.border": "#e5e7eb",
        "scrollbarSlider.background": "#d1d5db60",
        "scrollbarSlider.hoverBackground": "#d1d5dba0",
        "scrollbarSlider.activeBackground": "#9ca3af",
      },
    });
  }

  interface Props {
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    readonly?: boolean;
    schema?: unknown;
    language?: "json" | "xml" | "plaintext";
    variables?: Record<string, string>;
    functionNames?: string[];
  }

  let {
    value,
    onChange,
    placeholder = "{}",
    readonly = false,
    schema = undefined,
    language = "json",
    variables = undefined,
    functionNames = undefined,
  }: Props = $props();

  let container: HTMLDivElement | undefined;
  let editor: monaco.editor.IStandaloneCodeEditor | undefined;
  let model: monaco.editor.ITextModel | undefined;
  let ignoreChange = false;
  let decorCollection: monaco.editor.IEditorDecorationsCollection | undefined;

  // URI of the active model — plain string, set on mount, cleared on destroy.
  let currentUriStr = "";

  // Create the editor once on mount; tear it down on destroy.
  // onMount is NOT a reactive context — no prop reads here register as signal
  // dependencies, so prop changes never tear down and recreate the editor.
  onMount(() => {
    ensureMonacoEnv();
    registerThemes();
    registerCompletionProviders();

    const lang = language;
    const isDark = document.documentElement.classList.contains("dark");
    const modelUri = monaco.Uri.parse(`inmemory://flupi/body.${lang}`);
    currentUriStr = modelUri.toString();
    completionContexts.set(currentUriStr, { variables, functionNames });

    model = monaco.editor.createModel(value, lang, modelUri);

    editor = monaco.editor.create(container!, {
      model,
      theme: isDark ? "flupi-dark" : "flupi-light",
      automaticLayout: true,
      minimap: { enabled: false },
      wordWrap: "on",
      tabSize: 2,
      fontSize: 13,
      fontFamily:
        "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      scrollBeyondLastLine: false,
      readOnly: readonly,
      overviewRulerLanes: 0,
      lineNumbers: "on",
      formatOnPaste: true,
    });

    decorCollection = editor.createDecorationsCollection([]);

    const changeDispose = model.onDidChangeContent(() => {
      // onChange is a $props() getter — reading it here always returns the current
      // prop value at the time the callback fires, not the value captured at creation.
      if (!ignoreChange) onChange(editor!.getValue());
      updateDecorations();
    });

    // Suppress markers that overlap any {{variable}} template span (JSON only)
    const markerDispose = monaco.editor.onDidChangeMarkers((uris) => {
      if (!model) return;
      const uriStr = currentUriStr;
      if (!uris.some((u) => u.toString() === uriStr)) return;
      const markers = monaco.editor.getModelMarkers({ resource: modelUri });
      const text = model.getValue();
      const tmplRe = /\{\{[^}]*\}\}/g;
      const ranges: Array<[number, number]> = [];
      let m: RegExpExecArray | null;
      while ((m = tmplRe.exec(text)) !== null)
        ranges.push([m.index, m.index + m[0].length]);
      if (!ranges.length) return;
      const filtered = markers.filter((mk) => {
        const s = model!.getOffsetAt({
          lineNumber: mk.startLineNumber,
          column: mk.startColumn,
        });
        const e = model!.getOffsetAt({
          lineNumber: mk.endLineNumber,
          column: mk.endColumn,
        });
        return !ranges.some(([rs, re]) => rs < e && re > s);
      });
      if (filtered.length !== markers.length) {
        monaco.editor.setModelMarkers(model!, lang, filtered);
      }
    });

    return () => {
      changeDispose.dispose();
      markerDispose.dispose();
      editor?.dispose();
      editor = undefined;
      decorCollection = undefined;
      applySchema(undefined);
      completionContexts.delete(currentUriStr);
      model?.dispose();
      model = undefined;
      currentUriStr = "";
    };
  });

  // Sync value from parent (e.g. switching requests) without destroying undo history.
  // Uses pushEditOperations so Ctrl+Z continues to work within the same session.
  $effect(() => {
    const v = value;
    if (model && editor && editor.getValue() !== v) {
      ignoreChange = true;
      model.pushEditOperations(
        [],
        [{ range: model.getFullModelRange(), text: v }],
        () => null,
      );
      ignoreChange = false;
    }
  });

  // Sync readonly prop
  $effect(() => {
    editor?.updateOptions({ readOnly: readonly });
  });

  // Sync language without recreating the editor
  $effect(() => {
    const lang = language;
    if (model) monaco.editor.setModelLanguage(model, lang);
  });

  // Apply/clear JSON schema when language or schema prop changes
  $effect(() => {
    applySchema(language === "json" ? schema : undefined);
  });

  function applySchema(s: unknown) {
    if (!currentUriStr) return;
    const schemaObj = s != null && typeof s === "object" ? s : undefined;
    monaco.json.jsonDefaults.setDiagnosticsOptions({
      validate: true,
      enableSchemaRequest: false,
      schemas: schemaObj
        ? [
            {
              uri: currentUriStr,
              fileMatch: [currentUriStr],
              schema: schemaObj,
            },
          ]
        : [],
    });
  }

  // Sync theme (Monaco themes are global)
  $effect(() => {
    const t = $theme;
    const isDark =
      t === "dark" ||
      (t === "system" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);
    registerThemes();
    monaco.editor.setTheme(isDark ? "flupi-dark" : "flupi-light");
  });

  // Update Monaco inline decorations for {{var}} and {{$fn()}} tokens
  const TOKEN_REGEX = /\{\{(\$[a-zA-Z_$][a-zA-Z0-9_$]*\([^)]*\)|[\w.-]+)\}\}/g;

  function computeDecorations(): monaco.editor.IModelDeltaDecoration[] {
    if (!model) return [];
    const text = model.getValue();
    const result: monaco.editor.IModelDeltaDecoration[] = [];
    TOKEN_REGEX.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = TOKEN_REGEX.exec(text)) !== null) {
      const start = model.getPositionAt(m.index);
      const end = model.getPositionAt(m.index + m[0].length);
      const capture = m[1];
      let inlineClassName: string;
      if (capture.startsWith("$")) {
        const fnName =
          capture.match(/^\$([a-zA-Z_$][a-zA-Z0-9_$]*)/)?.[1] ?? "";
        inlineClassName = functionNames?.includes(fnName)
          ? "flupi-fn-found"
          : "flupi-fn-missing";
      } else {
        inlineClassName =
          variables != null && capture in variables
            ? "flupi-var-found"
            : "flupi-var-missing";
      }
      result.push({
        range: new monaco.Range(
          start.lineNumber,
          start.column,
          end.lineNumber,
          end.column,
        ),
        options: { inlineClassName },
      });
    }
    return result;
  }

  function updateDecorations() {
    decorCollection?.set(computeDecorations());
  }

  // Recompute decorations and sync completion context when variables or functionNames props change
  $effect(() => {
    // Access both props so Svelte tracks them as dependencies
    const _v = variables;
    const _f = functionNames;
    if (currentUriStr)
      completionContexts.set(currentUriStr, {
        variables: _v,
        functionNames: _f,
      });
    updateDecorations();
  });

  function handleFormat() {
    editor?.getAction("editor.action.formatDocument")?.run();
  }
</script>

<div class="flex flex-col h-full">
  <div
    class="flex items-center justify-end px-3 py-1 border-b border-app-border bg-app-panel shrink-0"
  >
    <button
      class="text-xs text-app-text-3 hover:text-app-text transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      onclick={handleFormat}
      disabled={readonly}>Format</button
    >
  </div>
  <div bind:this={container} class="flex-1 min-h-0"></div>
</div>
