import type { ScriptFunction } from '$lib/services/tauri-commands';
import { evalInSandbox } from '$lib/services/function-sandbox';

/** Parse a comma-separated, optionally-quoted args string into an array of strings. */
function parseArgs(argsStr: string): string[] {
  const trimmed = argsStr.trim();
  if (!trimmed) return [];

  const result: string[] = [];
  let current = '';
  let inQuote: '"' | "'" | null = null;

  for (let i = 0; i < trimmed.length; i++) {
    const ch = trimmed[i];
    if (inQuote) {
      if (ch === inQuote) {
        inQuote = null;
      } else {
        current += ch;
      }
    } else if (ch === '"' || ch === "'") {
      inQuote = ch;
    } else if (ch === ',') {
      result.push(current.trim());
      current = '';
    } else {
      current += ch;
    }
  }
  result.push(current.trim());
  return result;
}

/**
 * Scan an array of template strings and return every unique function token
 * found, e.g. ["$randomGuid()", "$randomInt(1, 100)"].
 */
export function extractFunctionTokens(templates: string[]): string[] {
  const FUNC_TOKEN_RE = /\{\{\$([a-zA-Z_$][a-zA-Z0-9_$]*)\(([^)]*)\)\}\}/g;
  const seen = new Set<string>();
  for (const t of templates) {
    FUNC_TOKEN_RE.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = FUNC_TOKEN_RE.exec(t)) !== null) {
      seen.add(`$${match[1]}(${match[2]})`);
    }
  }
  return [...seen];
}

/**
 * Evaluate all function call tokens found in the given templates.
 *
 * Returns a map of token → resolved string value, keyed exactly as they
 * appear inside {{ }}, e.g. `{ "$randomGuid()": "a3f2..." }`.
 *
 * Functions are evaluated in a sandboxed iframe with `sandbox="allow-scripts"`,
 * which has no access to the Tauri bridge, DOM storage, or parent window.
 *
 * Throws a descriptive Error if any function is not found or throws during
 * evaluation. The caller is responsible for surfacing this error to the user
 * and aborting the request/scenario.
 */
export async function evaluateFunctionCalls(
  templates: string[],
  scriptFunctions: ScriptFunction[],
): Promise<Record<string, string>> {
  const tokens = extractFunctionTokens(templates);
  if (tokens.length === 0) return {};

  const fnMap = new Map(scriptFunctions.map((f) => [f.name, f]));
  const result: Record<string, string> = {};

  for (const token of tokens) {
    const parenIdx = token.indexOf('(');
    const name = token.slice(1, parenIdx); // strip leading $
    const argsStr = token.slice(parenIdx + 1, -1);
    const args = parseArgs(argsStr);

    const fn = fnMap.get(name);
    if (fn === undefined) {
      throw new Error(`Function "${name}" is not defined. Create it on the Functions page.`);
    }

    // Inject named param bindings as a preamble so bodies can use either
    // the param name (e.g. `min`) or the positional index (e.g. `args[0]`).
    const preamble = (fn.params ?? [])
      .map((p, i) => `const ${p.name} = args[${i}];\n`)
      .join('');

    try {
      const { result: value } = await evalInSandbox(preamble + fn.body, args);
      result[token] = value;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      throw new Error(`Function "${name}" threw an error: ${msg}`);
    }
  }

  return result;
}
