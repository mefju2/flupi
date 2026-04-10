<script lang="ts">
  import type { GitFileStatus } from "$lib/stores/git";
  import ContextMenu from "$lib/components/shared/ContextMenu.svelte";

  interface Props {
    x: number;
    y: number;
    path: string;
    status: GitFileStatus;
    onclose: () => void;
    ondiscard?: (path: string, status: GitFileStatus) => void;
    ondelete?: (path: string, status: GitFileStatus) => void;
    onshowexplorer?: (path: string, status: GitFileStatus) => void;
  }

  let {
    x,
    y,
    path,
    status,
    onclose,
    ondiscard,
    ondelete,
    onshowexplorer,
  }: Props = $props();

  const items = $derived([
    ...(status !== "untracked"
      ? [{ label: "Discard changes", action: () => ondiscard?.(path, status) }]
      : []),
    {
      label: "Remove file",
      action: () => ondelete?.(path, status),
      danger: true as const,
    },
    {
      label: "Show in file explorer",
      action: () => onshowexplorer?.(path, status),
    },
  ]);
</script>

<ContextMenu {x} {y} {items} onClose={onclose} />
