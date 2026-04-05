<script lang="ts">
  import { untrack } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { theme } from '$lib/stores/ui';
  import { ensureMonacoEnv } from '$lib/services/monaco-env';

  // Register themes once per module lifetime
  let _themesRegistered = false;
  function registerThemes() {
    if (_themesRegistered) return;
    _themesRegistered = true;

    monaco.editor.defineTheme('flupi-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'string.value.json', foreground: '86efac' },
        { token: 'string.key.json', foreground: 'f4f4f5' },
        { token: 'number.json', foreground: '7dd3fc' },
        { token: 'keyword.json', foreground: 'c084fc' },
        { token: 'delimiter.bracket.json', foreground: '71717a' },
        { token: 'delimiter.array.json', foreground: '71717a' },
        { token: 'delimiter.colon.json', foreground: '71717a' },
        { token: 'delimiter.comma.json', foreground: '71717a' },
      ],
      colors: {
        'editor.background': '#18181b',
        'editor.foreground': '#f4f4f5',
        'editor.lineHighlightBackground': '#27272a',
        'editorLineNumber.foreground': '#52525b',
        'editorLineNumber.activeForeground': '#a1a1aa',
        'editor.selectionBackground': '#3f3f46',
        'editor.inactiveSelectionBackground': '#3f3f4680',
        'editorCursor.foreground': '#06b6d4',
        'editorIndentGuide.background1': '#27272a',
        'editorIndentGuide.activeBackground1': '#3f3f46',
        'editorWidget.background': '#27272a',
        'editorWidget.border': '#3f3f46',
        'editorSuggestWidget.background': '#27272a',
        'editorSuggestWidget.border': '#3f3f46',
        'editorSuggestWidget.selectedBackground': '#3f3f46',
        'scrollbarSlider.background': '#3f3f4660',
        'scrollbarSlider.hoverBackground': '#3f3f46a0',
        'scrollbarSlider.activeBackground': '#52525b',
      },
    });

    monaco.editor.defineTheme('flupi-light', {
      base: 'vs',
      inherit: true,
      rules: [
        { token: 'string.value.json', foreground: '166534' },
        { token: 'string.key.json', foreground: '030712' },
        { token: 'number.json', foreground: '1d4ed8' },
        { token: 'keyword.json', foreground: '7e22ce' },
        { token: 'delimiter.bracket.json', foreground: '6b7280' },
        { token: 'delimiter.array.json', foreground: '6b7280' },
        { token: 'delimiter.colon.json', foreground: '6b7280' },
        { token: 'delimiter.comma.json', foreground: '6b7280' },
      ],
      colors: {
        'editor.background': '#f9fafb',
        'editor.foreground': '#030712',
        'editor.lineHighlightBackground': '#f3f4f6',
        'editorLineNumber.foreground': '#9ca3af',
        'editorLineNumber.activeForeground': '#6b7280',
        'editor.selectionBackground': '#e5e7eb',
        'editorCursor.foreground': '#0891b2',
        'editorIndentGuide.background1': '#e5e7eb',
        'editorIndentGuide.activeBackground1': '#d1d5db',
        'editorWidget.background': '#f9fafb',
        'editorWidget.border': '#e5e7eb',
        'scrollbarSlider.background': '#d1d5db60',
        'scrollbarSlider.hoverBackground': '#d1d5dba0',
        'scrollbarSlider.activeBackground': '#9ca3af',
      },
    });
  }

  interface Props {
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    readonly?: boolean;
    schema?: unknown;
    language?: 'json' | 'xml' | 'plaintext';
  }

  let { value, onChange, placeholder = '{}', readonly = false, schema = undefined, language = 'json' }: Props = $props();

  // Tracks the URI of the currently active model so the schema effect can reference it
  let _currentUriStr = $state('');

  let container = $state<HTMLDivElement | undefined>(undefined);
  let editor: monaco.editor.IStandaloneCodeEditor | undefined;
  let ignoreChange = false;

  // Create / destroy editor
  $effect(() => {
    if (!container) return;

    const lang = language; // tracked — editor recreates when language changes
    const initialValue = untrack(() => value);
    const initialReadonly = untrack(() => readonly);

    ensureMonacoEnv();
    registerThemes();

    const isDark = document.documentElement.classList.contains('dark');
    const modelUri = monaco.Uri.parse(`inmemory://flupi/body.${lang}`);
    _currentUriStr = modelUri.toString();

    const model = monaco.editor.createModel(initialValue, lang, modelUri);

    editor = monaco.editor.create(container, {
      model,
      theme: isDark ? 'flupi-dark' : 'flupi-light',
      automaticLayout: true,
      minimap: { enabled: false },
      wordWrap: 'on',
      tabSize: 2,
      fontSize: 13,
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
      scrollBeyondLastLine: false,
      readOnly: initialReadonly,
      overviewRulerLanes: 0,
      lineNumbers: 'on',
      formatOnPaste: true,
    });

    const changeDispose = model.onDidChangeContent(() => {
      if (!ignoreChange) onChange(editor!.getValue());
    });

    // Suppress markers that overlap any {{variable}} template span (JSON only)
    const markerDispose = monaco.editor.onDidChangeMarkers((uris) => {
      const uriStr = modelUri.toString();
      if (!uris.some((u) => u.toString() === uriStr)) return;
      const markers = monaco.editor.getModelMarkers({ resource: modelUri });
      const text = model.getValue();
      const tmplRe = /\{\{[^}]*\}\}/g;
      const ranges: Array<[number, number]> = [];
      let m: RegExpExecArray | null;
      while ((m = tmplRe.exec(text)) !== null) ranges.push([m.index, m.index + m[0].length]);
      if (!ranges.length) return;
      const filtered = markers.filter((mk) => {
        const s = model.getOffsetAt({ lineNumber: mk.startLineNumber, column: mk.startColumn });
        const e = model.getOffsetAt({ lineNumber: mk.endLineNumber, column: mk.endColumn });
        return !ranges.some(([rs, re]) => rs < e && re > s);
      });
      if (filtered.length !== markers.length) {
        monaco.editor.setModelMarkers(model, lang, filtered);
      }
    });

    return () => {
      changeDispose.dispose();
      markerDispose.dispose();
      editor?.dispose();
      editor = undefined;
      if (lang === 'json') applySchema(undefined);
      model.dispose();
    };
  });

  // Sync value from parent (e.g. switching requests) without cursor jump
  $effect(() => {
    const v = value;
    if (editor && editor.getValue() !== v) {
      ignoreChange = true;
      editor.setValue(v);
      ignoreChange = false;
    }
  });

  // Sync readonly prop
  $effect(() => {
    editor?.updateOptions({ readOnly: readonly });
  });

  // H-1: Schema is applied solely through this reactive effect — no duplicate call in mount.
  // Cleanup calls applySchema(undefined) before this effect fires for a new mount,
  // so there is no race between unmount clearing and remount setting the schema.
  $effect(() => {
    if (language === 'json') applySchema(schema);
  });

  function applySchema(s: unknown) {
    const uriStr = _currentUriStr;
    if (!uriStr) return;
    const schemaObj = s != null && typeof s === 'object' ? s : undefined;
    monaco.json.jsonDefaults.setDiagnosticsOptions({
      validate: true,
      enableSchemaRequest: false,
      schemas: schemaObj
        ? [{ uri: uriStr, fileMatch: [uriStr], schema: schemaObj }]
        : [],
    });
  }

  // Sync theme (Monaco themes are global)
  $effect(() => {
    const t = $theme;
    const isDark = t === 'dark' || (t === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    registerThemes();
    monaco.editor.setTheme(isDark ? 'flupi-dark' : 'flupi-light');
  });

  function handleFormat() {
    editor?.getAction('editor.action.formatDocument')?.run();
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-end px-3 py-1 border-b border-app-border bg-app-panel shrink-0">
    <button
      class="text-xs text-app-text-3 hover:text-app-text transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      onclick={handleFormat}
      disabled={readonly}
    >Format</button>
  </div>
  <div bind:this={container} class="flex-1 min-h-0"></div>
</div>
