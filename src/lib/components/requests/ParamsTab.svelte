<script lang="ts">
  import KeyValueTable from '$lib/components/shared/KeyValueTable.svelte';

  interface Row {
    key: string;
    value: string;
  }

  interface Props {
    path: string;
    onPathChange: (path: string) => void;
  }

  let { path, onPathChange }: Props = $props();

  function parseParams(p: string): Row[] {
    const idx = p.indexOf('?');
    if (idx === -1) return [];
    return p
      .slice(idx + 1)
      .split('&')
      .filter(Boolean)
      .map((pair) => {
        const [k, ...rest] = pair.split('=');
        return { key: decodeURIComponent(k ?? ''), value: decodeURIComponent(rest.join('=') ?? '') };
      });
  }

  function buildPath(basePath: string, rows: Row[]): string {
    const base = basePath.split('?')[0];
    const qs = rows
      .filter((r) => r.key)
      .map((r) => `${encodeURIComponent(r.key)}=${encodeURIComponent(r.value)}`)
      .join('&');
    return qs ? `${base}?${qs}` : base;
  }

  let rows = $derived(parseParams(path));

  function handleUpdate(updated: Row[]) {
    onPathChange(buildPath(path, updated));
  }
</script>

<div class="p-4">
  <p class="text-xs text-app-text-3 mb-3">Query parameters are appended to the URL path.</p>
  <KeyValueTable rows={rows} onUpdate={handleUpdate} />
</div>
