<script lang="ts">
  import { untrack } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { theme } from '$lib/stores/ui';
  import { ensureMonacoEnv } from '$lib/services/monaco-env';

  let _themesRegistered = false;
  function registerThemes() {
    if (_themesRegistered) return;
    _themesRegistered = true;
    monaco.editor.defineTheme('flupi-dark', {
      base: 'vs-dark', inherit: true, rules: [],
      colors: {
        'editor.background': '#18181b', 'editor.foreground': '#f4f4f5',
        'editor.lineHighlightBackground': '#27272a', 'editorLineNumber.foreground': '#52525b',
        'editorLineNumber.activeForeground': '#a1a1aa', 'editor.selectionBackground': '#3f3f46',
        'editorCursor.foreground': '#06b6d4', 'editorIndentGuide.background1': '#27272a',
        'editorIndentGuide.activeBackground1': '#3f3f46', 'editorWidget.background': '#27272a',
        'editorWidget.border': '#3f3f46', 'scrollbarSlider.background': '#3f3f4660',
        'scrollbarSlider.hoverBackground': '#3f3f46a0',
      },
    });
    monaco.editor.defineTheme('flupi-light', {
      base: 'vs', inherit: true, rules: [],
      colors: {
        'editor.background': '#f9fafb', 'editor.foreground': '#030712',
        'editor.lineHighlightBackground': '#f3f4f6', 'editorLineNumber.foreground': '#9ca3af',
        'editorLineNumber.activeForeground': '#6b7280', 'editor.selectionBackground': '#e5e7eb',
        'editorCursor.foreground': '#0891b2', 'editorIndentGuide.background1': '#e5e7eb',
        'editorIndentGuide.activeBackground1': '#d1d5db', 'editorWidget.background': '#f9fafb',
        'editorWidget.border': '#e5e7eb',
      },
    });
  }

  const MODEL_URI = monaco.Uri.parse('inmemory://flupi/function.js');

  interface Props {
    value: string;
    onChange: (v: string) => void;
    readonly?: boolean;
  }

  let { value, onChange, readonly = false }: Props = $props();

  let container = $state<HTMLDivElement | undefined>(undefined);
  let editor: monaco.editor.IStandaloneCodeEditor | undefined;
  let ignoreChange = false;

  $effect(() => {
    if (!container) return;
    const initialValue = untrack(() => value);
    ensureMonacoEnv();
    registerThemes();
    const isDark = document.documentElement.classList.contains('dark');

    let model = monaco.editor.getModel(MODEL_URI);
    if (!model) model = monaco.editor.createModel(initialValue, 'javascript', MODEL_URI);
    else model.setValue(initialValue);

    editor = monaco.editor.create(container, {
      model,
      theme: isDark ? 'flupi-dark' : 'flupi-light',
      fontSize: 13,
      tabSize: 2,
      lineNumbers: 'on',
      wordWrap: 'on',
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      readOnly: readonly,
      automaticLayout: true,
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
    });

    const disposable = editor.onDidChangeModelContent(() => {
      if (ignoreChange) return;
      onChange(editor!.getValue());
    });

    return () => {
      disposable.dispose();
      editor?.dispose();
      editor = undefined;
      const m = monaco.editor.getModel(MODEL_URI);
      m?.dispose();
    };
  });

  $effect(() => {
    const currentTheme = $theme;
    const isDark = currentTheme === 'dark' || (currentTheme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    monaco.editor.setTheme(isDark ? 'flupi-dark' : 'flupi-light');
  });

  $effect(() => {
    const incoming = value;
    if (!editor) return;
    const current = editor.getValue();
    if (current === incoming) return;
    ignoreChange = true;
    editor.setValue(incoming);
    ignoreChange = false;
  });
</script>

<div bind:this={container} class="w-full h-full"></div>
