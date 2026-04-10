<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    Settings2,
    FileCode,
    ArrowUpRight,
    Play,
    SlidersHorizontal,
    Code2,
    GitBranch,
  } from "lucide-svelte";
  import { gitBehindCount } from "$lib/stores/git";

  const sections = [
    { path: "/environments", label: "Environments", icon: Settings2 },
    { path: "/openapi", label: "OpenAPI", icon: FileCode },
    { path: "/requests", label: "Requests", icon: ArrowUpRight },
    { path: "/scenarios", label: "Scenarios", icon: Play },
    { path: "/functions", label: "Functions", icon: Code2 },
    { path: "/git", label: "Git", icon: GitBranch },
    { path: "/settings", label: "Settings", icon: SlidersHorizontal },
  ];

  // Clear the badge when the user navigates to the Git page.
  $: if ($page.url.pathname.startsWith("/git") && $gitBehindCount > 0) {
    gitBehindCount.set(0);
  }
</script>

<nav
  class="flex flex-col w-14 border-r border-app-border bg-app-panel shrink-0"
>
  {#each sections as s}
    <div class="relative group/nav">
      <button
        class="flex items-center justify-center h-14 w-14 transition-colors
          {$page.url.pathname.startsWith(s.path)
          ? 'text-cyan-400 bg-app-card'
          : 'text-app-text-3 hover:text-app-text-2 hover:bg-app-card/50'}"
        onclick={() => goto(s.path)}
        aria-label={s.label}
      >
        <s.icon size={18} />
      </button>
      {#if s.path === "/git" && $gitBehindCount > 0}
        <span
          class="pointer-events-none absolute top-2.5 right-2.5 w-2 h-2 rounded-full bg-amber-400"
        ></span>
      {/if}
      <div
        class="pointer-events-none absolute left-full top-1/2 -translate-y-1/2 ml-2 px-2 py-1
                  bg-app-card border border-app-border-2 rounded text-xs text-app-text whitespace-nowrap
                  opacity-0 group-hover/nav:opacity-100 transition-opacity duration-150 z-50"
      >
        {#if s.path === "/git" && $gitBehindCount > 0}
          Git — {$gitBehindCount} commit{$gitBehindCount === 1 ? "" : "s"} to pull
        {:else}
          {s.label}
        {/if}
      </div>
    </div>
  {/each}
</nav>
