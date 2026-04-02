<script lang="ts">
  interface Props {
    value: string;
    onChange: (value: string) => void;
    envVars: string[];
    placeholder?: string;
  }

  let { value, onChange, envVars, placeholder = 'Select variable…' }: Props = $props();
</script>

<select
  class="w-full bg-app-card border border-app-border-2 rounded px-2 py-1 text-sm font-mono focus:outline-none focus:border-app-border-2 {value ? 'text-app-text' : 'text-app-text-4'}"
  {value}
  onchange={(e) => onChange(e.currentTarget.value)}
>
  <option value="" disabled selected={!value}>{placeholder}</option>
  {#each envVars as name}
    <option value={name}>{name}</option>
  {/each}
  {#if value && !envVars.includes(value)}
    <option value={value}>{value} (not in env)</option>
  {/if}
</select>
