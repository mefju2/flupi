<script lang="ts">
  import { onMount } from 'svelte';
  import type { ScenarioData } from '$lib/services/tauri-commands';
  import PreRunForm from './PreRunForm.svelte';
  import RunnerStepper from './RunnerStepper.svelte';

  interface Props {
    scenario: ScenarioData;
    onBack: () => void;
    onRun: (inputs: Record<string, string>) => Promise<Record<string, string> | void>;
  }

  let { scenario, onBack, onRun }: Props = $props();

  type Phase = 'form' | 'running';
  let phase = $state<Phase>(scenario.inputs.length === 0 ? 'running' : 'form');
  let lastInputs = $state<Record<string, string>>({});
  let runKey = $state(0);

  onMount(() => {
    if (scenario.inputs.length === 0) {
      onRun({}).catch((e) => console.error('Scenario run failed:', e));
    }
  });

  async function handleRun(inputs: Record<string, string>) {
    lastInputs = inputs;
    phase = 'running';
    try {
      const resolved = await onRun(inputs);
      if (resolved) lastInputs = resolved;
    } catch (e) {
      console.error('Scenario run failed:', e);
    }
  }

  async function retryRun() {
    runKey += 1;
    try {
      await onRun(lastInputs);
    } catch (e) {
      console.error('Scenario retry failed:', e);
    }
  }
</script>

{#if phase === 'form'}
  <PreRunForm {scenario} onRun={handleRun} {onBack} />
{:else}
  {#key runKey}
    <RunnerStepper {scenario} inputs={lastInputs} onBack={() => (phase = 'form')} onRetry={retryRun} />
  {/key}
{/if}
