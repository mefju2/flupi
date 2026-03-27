interface Shortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  handler: () => void;
}

export function registerShortcuts(shortcuts: Shortcut[]) {
  function handleKeydown(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;
    for (const s of shortcuts) {
      if (e.key === s.key && !!ctrl === !!s.ctrl && !!e.shiftKey === !!s.shift) {
        e.preventDefault();
        s.handler();
        return;
      }
    }
  }

  window.addEventListener('keydown', handleKeydown);
  return () => window.removeEventListener('keydown', handleKeydown);
}
