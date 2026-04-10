<script lang="ts">
  interface Props {
    hasStagedFiles: boolean;
    isCommitting: boolean;
    oncommit: (message: string) => void;
  }

  let { hasStagedFiles, isCommitting, oncommit }: Props = $props();

  let message = $state('');

  const canCommit = $derived(hasStagedFiles && message.trim().length > 0 && !isCommitting);

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter' && canCommit) submit();
  }

  function submit() {
    if (!canCommit) return;
    oncommit(message.trim());
    message = '';
  }
</script>

<div class="flex flex-col gap-2 shrink-0">
  <div class="border-t border-app-border"></div>
  <textarea
    class="w-full rounded bg-app-card border border-app-border text-xs text-app-text
           font-sans px-2 py-1.5 resize-none placeholder:text-app-text-3 focus:outline-none
           focus:border-cyan-500/50 transition-colors disabled:opacity-50"
    rows={3}
    placeholder={hasStagedFiles ? 'Commit message (⌘↵ to commit)' : 'Stage files to commit'}
    bind:value={message}
    disabled={!hasStagedFiles || isCommitting}
    onkeydown={handleKeydown}
  ></textarea>
  <button
    class="w-full px-3 py-1.5 rounded text-xs font-medium transition-colors
           {canCommit
             ? 'bg-cyan-600 hover:bg-cyan-500 text-white cursor-pointer'
             : 'bg-app-card text-app-text-3 cursor-not-allowed'}"
    onclick={submit}
    disabled={!canCommit}
  >
    {isCommitting ? 'Committing…' : 'Commit'}
  </button>
</div>
