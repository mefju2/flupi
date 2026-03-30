export interface SchemaSuggestion {
  path: string;
  type?: string;
}

function extractFromProperties(
  props: Record<string, unknown>,
  prefix: string,
  depth: number,
  results: SchemaSuggestion[],
): void {
  for (const [key, val] of Object.entries(props)) {
    const fullPath = prefix ? `${prefix}.${key}` : key;
    const propSchema = val as Record<string, unknown>;
    results.push({ path: fullPath, type: propSchema.type as string | undefined });
    if (depth < 3) {
      extractSchemaPaths(val, fullPath, depth + 1, results);
    }
  }
}

export function extractSchemaPaths(
  schema: unknown,
  prefix: string,
  depth: number,
  results: SchemaSuggestion[] = [],
): SchemaSuggestion[] {
  if (depth > 3 || !schema || typeof schema !== 'object') return results;
  const s = schema as Record<string, unknown>;

  // Handle allOf / oneOf / anyOf by merging schemas
  for (const key of ['allOf', 'oneOf', 'anyOf']) {
    if (Array.isArray(s[key])) {
      for (const sub of s[key] as unknown[]) {
        extractSchemaPaths(sub, prefix, depth, results);
      }
    }
  }

  if (s.properties && typeof s.properties === 'object') {
    extractFromProperties(s.properties as Record<string, unknown>, prefix, depth, results);
  }

  // Handle array items
  if (s.items && typeof s.items === 'object') {
    const itemPrefix = `${prefix}[*]`;
    extractSchemaPaths(s.items, itemPrefix, depth + 1, results);
  }

  return results;
}

export function buildOverrideSuggestions(
  requestSchema: unknown,
  requestPath?: string,
): SchemaSuggestion[] {
  const results: SchemaSuggestion[] = [];

  if (requestSchema && typeof requestSchema === 'object') {
    const s = requestSchema as Record<string, unknown>;

    // Body properties
    const body = s.body ?? s.requestBody ?? s.content ?? s;
    extractSchemaPaths(body, 'body', 0, results);

    // Header and path parameters from OpenAPI parameters array
    if (Array.isArray(s.parameters)) {
      for (const param of s.parameters as Record<string, unknown>[]) {
        if (param.in === 'header' && typeof param.name === 'string') {
          results.push({ path: `headers.${param.name}`, type: 'string' });
        }
        if (param.in === 'path' && typeof param.name === 'string') {
          results.push({ path: `path.${param.name}`, type: 'string' });
        }
      }
    }
  }

  // Path params from URL pattern
  if (requestPath) {
    const matches = [...requestPath.matchAll(/(?<!\{)\{([a-zA-Z0-9_-]+)\}(?!\})/g)];
    const alreadySuggested = new Set(
      results.filter((r) => r.path.startsWith('path.')).map((r) => r.path.slice(5)),
    );
    for (const m of matches) {
      if (!alreadySuggested.has(m[1])) {
        results.push({ path: `path.${m[1]}`, type: 'string' });
      }
    }
  }

  return results;
}

export function buildJsonPathSuggestions(responseSchema: unknown): SchemaSuggestion[] {
  if (!responseSchema || typeof responseSchema !== 'object') return [];
  const results: SchemaSuggestion[] = [];
  extractSchemaPaths(responseSchema, '$', 0, results);
  // Replace body[*] style prefixes with proper JSONPath syntax already handled by extractSchemaPaths
  return results;
}
