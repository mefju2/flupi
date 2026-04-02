<script lang="ts">
  interface Props {
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    readonly?: boolean;
  }

  let { value, onChange, placeholder = '{}', readonly = false }: Props = $props();
</script>

<textarea
  class="w-full min-h-[180px] bg-app-panel border border-app-border-2 px-3 py-2 text-sm text-app-text font-mono placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2 resize-y rounded-none"
  {value}
  {placeholder}
  {readonly}
  oninput={(e) => onChange(e.currentTarget.value)}
></textarea>
