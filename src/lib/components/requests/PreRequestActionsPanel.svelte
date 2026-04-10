<script lang="ts">
  import type { PreRequestAction } from "$lib/services/tauri-commands";
  import { functions } from "$lib/stores/functions";
  import EnvVarCombobox from "$lib/components/shared/EnvVarCombobox.svelte";
  import { environments, activeEnvironment } from "$lib/stores/environment";

  interface Props {
    actions: PreRequestAction[];
    onUpdate: (actions: PreRequestAction[]) => void;
  }

  let { actions, onUpdate }: Props = $props();

  const envVarNames = $derived.by(() => {
    const entry = $environments.find((e) => e.fileName === $activeEnvironment);
    if (!entry) return [];
    return [
      ...Object.keys(entry.environment.variables),
      ...entry.environment.secrets,
    ];
  });

  function addAction() {
    onUpdate([
      ...actions,
      { type: "set_variable", variable: "", function_name: "", args: [] },
    ]);
  }

  function removeAction(i: number) {
    onUpdate(actions.filter((_, idx) => idx !== i));
  }

  function updateField(
    i: number,
    field: "variable" | "function_name",
    value: string,
  ) {
    const updated = [...actions];
    if (updated[i].type === "set_variable") {
      const action = { ...updated[i] } as Extract<
        PreRequestAction,
        { type: "set_variable" }
      >;
      if (field === "function_name") {
        // Reset args when function changes, using type-appropriate defaults
        const fn = $functions.find((f) => f.name === value);
        action.function_name = value;
        action.args = fn
          ? (fn.params ?? []).map((p) =>
              p.param_type === "boolean" ? "false" : "",
            )
          : [];
      } else {
        action[field] = value;
      }
      updated[i] = action;
    }
    onUpdate(updated);
  }

  function updateArg(actionIdx: number, argIdx: number, value: string) {
    const updated = [...actions];
    if (updated[actionIdx].type === "set_variable") {
      const action = { ...updated[actionIdx] } as Extract<
        PreRequestAction,
        { type: "set_variable" }
      >;
      const args = [...action.args];
      args[argIdx] = value;
      action.args = args;
      updated[actionIdx] = action;
    }
    onUpdate(updated);
  }
</script>

<div class="space-y-1">
  <p class="text-xs text-app-text-3 mb-2">
    Run actions before the request is sent. Each action calls a function and
    stores the result in an env variable.
  </p>

  {#if actions.length > 0}
    <div class="grid grid-cols-[1fr_1fr_auto] gap-2 mb-1">
      <span class="text-xs text-app-text-3 px-1">Variable</span>
      <span class="text-xs text-app-text-3 px-1">Function</span>
      <span></span>
    </div>
  {/if}

  {#each actions as action, i}
    {#if action.type === "set_variable"}
      {@const selectedFn =
        $functions.find((f) => f.name === action.function_name) ?? null}
      <div class="space-y-1.5 border border-app-border rounded p-2 bg-app-card">
        <div class="grid grid-cols-[1fr_1fr_auto] gap-2 items-center">
          <EnvVarCombobox
            value={action.variable}
            onChange={(v) => updateField(i, "variable", v)}
            envVars={envVarNames}
          />
          <select
            class="bg-app-bg border border-app-border-2 rounded px-2 py-1 text-sm text-app-text-2 focus:outline-none focus:border-app-border-2"
            value={action.function_name}
            onchange={(e) =>
              updateField(i, "function_name", e.currentTarget.value)}
          >
            <option value="">— Select function —</option>
            {#each $functions as fn}
              <option value={fn.name}>{fn.name}</option>
            {/each}
          </select>
          <button
            class="text-app-text-4 hover:text-red-400 transition-colors text-lg leading-none"
            onclick={() => removeAction(i)}
            aria-label="Remove action">×</button
          >
        </div>

        {#if selectedFn && (selectedFn.params ?? []).length > 0}
          <div
            class="grid gap-1.5 pl-1"
            style="grid-template-columns: repeat({Math.min(
              (selectedFn.params ?? []).length,
              3,
            )}, 1fr)"
          >
            {#each selectedFn.params ?? [] as param, pi}
              <div class="flex flex-col gap-0.5">
                <div class="flex items-center gap-1">
                  <span class="text-[10px] text-app-text-4 font-mono"
                    >{param.name}</span
                  >
                  <span
                    class="text-[9px] px-1 rounded font-sans {param.param_type ===
                    'string'
                      ? 'text-cyan-400 bg-cyan-500/10'
                      : param.param_type === 'number'
                        ? 'text-violet-400 bg-violet-500/10'
                        : 'text-amber-400 bg-amber-500/10'}"
                    >{param.param_type}</span
                  >
                </div>
                {#if param.param_type === "boolean"}
                  <label
                    class="flex items-center gap-2 h-7.5 bg-app-bg border border-app-border-2 rounded cursor-pointer px-2"
                  >
                    <input
                      type="checkbox"
                      class="accent-cyan-500 w-3.5 h-3.5"
                      checked={(action.args[pi] ?? "false") === "true"}
                      onchange={(e) =>
                        updateArg(
                          i,
                          pi,
                          e.currentTarget.checked ? "true" : "false",
                        )}
                    />
                    <span class="text-sm font-mono text-app-text-2"
                      >{(action.args[pi] ?? "false") === "true"
                        ? "true"
                        : "false"}</span
                    >
                  </label>
                {:else if param.param_type === "number"}
                  <input
                    type="number"
                    class="w-full bg-app-bg border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
                    value={action.args[pi] ?? ""}
                    placeholder="0"
                    oninput={(e) => updateArg(i, pi, e.currentTarget.value)}
                  />
                {:else}
                  <input
                    type="text"
                    class="w-full bg-app-bg border border-app-border-2 rounded px-2 py-1 text-sm font-mono text-app-text placeholder:text-app-text-4 focus:outline-none focus:border-app-border-2"
                    value={action.args[pi] ?? ""}
                    placeholder={param.name}
                    oninput={(e) => updateArg(i, pi, e.currentTarget.value)}
                  />
                {/if}
              </div>
            {/each}
          </div>
        {:else if selectedFn && (selectedFn.params ?? []).length === 0}
          <p class="text-[10px] text-app-text-4 pl-1">
            This function takes no parameters.
          </p>
        {/if}
      </div>
    {/if}
  {/each}

  <button
    class="text-xs text-cyan-500 hover:text-cyan-400 transition-colors mt-1"
    onclick={addAction}>+ Add Action</button
  >
</div>
