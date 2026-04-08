export function formatRelativeTime(iso: string | null): string {
  if (!iso) return "Never";
  const diff = Date.now() - new Date(iso).getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  return new Date(iso).toLocaleDateString();
}

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

export function tryParseJson(raw: string): unknown | null {
  try { return JSON.parse(raw); } catch { return null; }
}
