<script lang="ts">
  import FunctionList from '$lib/components/functions/FunctionList.svelte';
  import FunctionEditor from '$lib/components/functions/FunctionEditor.svelte';
</script>

<div class="flex h-full">
  <div class="w-56 shrink-0 border-r border-app-border">
    <FunctionList />
  </div>
  <div class="flex-1 min-w-0">
    <FunctionEditor />
  </div>
</div>
