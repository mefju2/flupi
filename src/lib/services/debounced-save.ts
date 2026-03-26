export function createDebouncedSave(saveFn: () => Promise<void>, delay = 500): {
  trigger: () => void;
  flush: () => Promise<void>;
} {
  let timer: ReturnType<typeof setTimeout> | null = null;

  function trigger() {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      saveFn().catch((e) => console.error('[auto-save] failed:', e));
      timer = null;
    }, delay);
  }

  async function flush() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    await saveFn();
  }

  return { trigger, flush };
}
