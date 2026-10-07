<script lang="ts">
  import IconPlus from "../../common/icons/IconPlus.svelte";
  import SettingsContainer from "../SettingsContainer.svelte";
</script>

<SettingsContainer>
  {#snippet title()}
    Workspaces
  {/snippet}
  <div class="flex flex-col grow">
    <div class="add-area">
      <IconPlus class="text-pink-500 size-20 self-center" />
      <h3>Create a new workspace</h3>
    </div>
  </div>
</SettingsContainer>

<style lang="postcss">
  @reference "tailwindcss";

  .add-area {
    @apply self-center flex flex-col gap-4;
  }
</style>
