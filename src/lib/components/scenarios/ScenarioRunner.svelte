<script lang="ts">
  import type { ScenarioData } from '$lib/services/tauri-commands';
  import PreRunForm from './PreRunForm.svelte';
  import RunnerStepper from './RunnerStepper.svelte';

  interface Props {
    scenario: ScenarioData;
    onBack: () => void;
    onRun: (inputs: Record<string, string>) => Promise<void>;
  }

  let { scenario, onBack, onRun }: Props = $props();

  type Phase = 'form' | 'running';
  let phase = $state<Phase>('form');

  async function handleRun(inputs: Record<string, string>) {
    phase = 'running';
    try {
      await onRun(inputs);
    } catch (e) {
      console.error('Scenario run failed:', e);
    }
  }
</script>

{#if phase === 'form'}
  <PreRunForm {scenario} onRun={handleRun} {onBack} />
{:else}
  <RunnerStepper {scenario} onBack={() => (phase = 'form')} />
{/if}
