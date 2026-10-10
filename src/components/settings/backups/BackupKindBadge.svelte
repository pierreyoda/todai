<script lang="ts">
  import type { WorkspaceBackupKind } from "../../../client/app";
  import IconClock from "../../common/icons/IconClock.svelte";

  const LABELS: Record<WorkspaceBackupKind, string> = {
    manual: "Manual",
    automatic: "Automatic",
    pre_migration: "Before update",
    pre_restore: "Before restore",
  };

  const DESCRIPTIONS: Record<WorkspaceBackupKind, string> = {
    manual: "Made on request",
    automatic: "Made when the workspace was opened, at most once a day",
    pre_migration:
      "Made before an update of todai changed the workspace's database",
    pre_restore: "The workspace's state before restoring another backup",
  };

  type BackupKindBadgeProps = {
    kind: WorkspaceBackupKind;
  };
  const { kind }: BackupKindBadgeProps = $props();
</script>

<span class={["badge", kind]} title={DESCRIPTIONS[kind]}>
  {#if kind === "automatic"}
    <IconClock class="size-3 text-slate-400" />
  {/if}
  {LABELS[kind]}
</span>

<style lang="postcss">
  @reference "tailwindcss";

  .badge {
    /* Layout */
    @apply inline-flex items-center gap-1 rounded-md px-2 py-0.5;
    /* Typography */
    @apply text-xs/4 font-medium whitespace-nowrap;
    /* Surface, matching the translucent fields and menus: neutral for routine backups, tinted for the others */
    @apply bg-white/5 text-slate-300 ring-1 ring-white/10 ring-inset;

    &.manual {
      @apply bg-pink-500/10 text-pink-300 ring-pink-500/20;
    }
    &.pre_migration {
      @apply bg-blue-500/10 text-blue-300 ring-blue-500/20;
    }
    &.pre_restore {
      @apply bg-amber-400/10 text-amber-300 ring-amber-400/20;
    }
  }
</style>
