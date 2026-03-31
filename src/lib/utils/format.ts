// Method color for Tailwind classes
export function getMethodColor(method: string): string {
  const colors: Record<string, string> = {
    GET: 'text-green-400',
    POST: 'text-cyan-400',
    PUT: 'text-yellow-400',
    PATCH: 'text-orange-400',
    DELETE: 'text-red-400',
    HEAD: 'text-purple-400',
    OPTIONS: 'text-zinc-400',
  };
  return colors[method?.toUpperCase()] ?? 'text-zinc-400';
}
