export function resolveString(template: string, variables: Record<string, string>): string {
  return template.replace(/\{\{(\w+)\}\}/g, (match, key) => {
    return variables[key] ?? match;
  });
}

export function findUnresolved(template: string, variables: Record<string, string>): string[] {
  const unresolved: string[] = [];
  template.replace(/\{\{(\w+)\}\}/g, (match, key) => {
    if (!(key in variables)) unresolved.push(key);
    return match;
  });
  return unresolved;
}


