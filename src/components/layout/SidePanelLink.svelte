<script lang="ts">
  import { page } from "$app/state";
  import type { Snippet } from "svelte";

  import type { RouteId } from "$app/types";
  import Button from "../common/Button.svelte";
  import type { AppPage } from "../../constants";

  type SidePanelLinkProps = {
    label: string;
    icon: Snippet;
    routeId: RouteId;
    /** Reverse icon and label. False by default. */
    reverse?: boolean;
  };
  const {
    label,
    icon,
    routeId,
    reverse = false,
  }: SidePanelLinkProps = $props();

  const currentRouteId = page.route.id ?? "/";
</script>

<a href={routeId}>
  <Button
    style="outline"
    class={["page-link", currentRouteId === routeId && "current"]}
  >
    <div
      class={[
        "w-full flex items-center justify-between",
        reverse && "flex-row-reverse",
      ]}
    >
      <h2>{label}</h2>
      {@render icon()}
    </div>
  </Button>
</a>

<style lang="postcss">
  @reference "tailwindcss";

  :global(.page-link) {
    &.current {
      @apply bg-gray-700 hover:bg-gray-500;
    }
  }
</style>
