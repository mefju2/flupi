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
  class="w-full min-h-[120px] bg-zinc-900 border border-zinc-700 px-3 py-2 text-sm text-zinc-100 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500 resize-y rounded-none"
  {value}
  {placeholder}
  {readonly}
  oninput={(e) => onChange(e.currentTarget.value)}
></textarea>
