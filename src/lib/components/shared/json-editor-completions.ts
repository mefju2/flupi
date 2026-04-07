import * as monaco from "monaco-editor";

export interface CompletionCtx {
  variables?: Record<string, string>;
  functionNames?: string[];
}

// Keyed by model URI string — each JsonEditor instance registers its context here
// so the globally-registered provider can look up per-editor data.
export const completionContexts = new Map<string, CompletionCtx>();

let _registered = false;

export function registerCompletionProviders() {
  if (_registered) return;
  _registered = true;

  function provideCompletionItems(
    m: monaco.editor.ITextModel,
    position: monaco.Position,
  ): monaco.languages.CompletionList | null {
    const ctx = completionContexts.get(m.uri.toString());
    if (!ctx || (!ctx.variables && !ctx.functionNames)) return null;

    const textBefore = m
      .getLineContent(position.lineNumber)
      .substring(0, position.column - 1);
    const lastOpen = textBefore.lastIndexOf("{{");
    if (lastOpen === -1) return null;
    const afterBraces = textBefore.slice(lastOpen + 2);
    // Already closed — don't suggest
    if (afterBraces.includes("}")) return null;

    const isFnMode = afterBraces.startsWith("$");
    const fragment = isFnMode ? afterBraces.slice(1) : afterBraces;

    // Replace from the opening '{' of '{{' through the cursor
    const replaceRange = new monaco.Range(
      position.lineNumber,
      lastOpen + 1,
      position.lineNumber,
      position.column,
    );

    const suggestions: monaco.languages.CompletionItem[] = [];

    if (!isFnMode && ctx.variables) {
      for (const [name, val] of Object.entries(ctx.variables)) {
        if (!name.toLowerCase().startsWith(fragment.toLowerCase())) continue;
        suggestions.push({
          label: `{{${name}}}`,
          kind: monaco.languages.CompletionItemKind.Variable,
          insertText: `{{${name}}}`,
          range: replaceRange,
          detail: val.length > 60 ? val.slice(0, 60) + "…" : val,
        });
      }
    }

    if (ctx.functionNames) {
      for (const name of ctx.functionNames) {
        if (!name.toLowerCase().startsWith(fragment.toLowerCase())) continue;
        suggestions.push({
          label: `{{$${name}()}}`,
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: `{{$${name}()}}`,
          range: replaceRange,
          detail: "function",
        });
      }
    }

    return { suggestions };
  }

  for (const lang of ["json", "xml", "plaintext"] as const) {
    monaco.languages.registerCompletionItemProvider(lang, {
      triggerCharacters: ["{", "$"],
      provideCompletionItems,
    });
  }
}
