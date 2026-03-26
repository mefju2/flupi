<script lang="ts">
  import EnvironmentList from '$lib/components/environments/EnvironmentList.svelte';
  import EnvironmentEditor from '$lib/components/environments/EnvironmentEditor.svelte';
</script>

<div class="flex h-full">
  <div class="w-56 shrink-0 border-r border-zinc-800 overflow-y-auto">
    <EnvironmentList />
  </div>
  <div class="flex-1 overflow-auto">
    <EnvironmentEditor />
  </div>
</div>
