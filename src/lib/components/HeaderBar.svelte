<script lang="ts">
  import Icon from "./Icon.svelte";
  import flashIcon from "$lib/assets/icons/flash.svg?url";
  import sunIcon from "$lib/assets/icons/sun.svg?url";
  import moonIcon from "$lib/assets/icons/moon.svg?url";
  import { theme } from "$lib/state/theme.svelte";
</script>

<header class="header">
  <div class="brand">
    <span class="mark"><Icon src={flashIcon} size={16} /></span>
    <strong>FlashDesk</strong>
  </div>

  <div class="actions">
    <button
      type="button"
      class="ui-btn ui-btn--ghost ui-btn--icon"
      title="切换主题"
      aria-label="切换主题"
      onclick={() => theme.toggle()}
    >
      <Icon src={theme.mode === "dark" ? sunIcon : moonIcon} size={16} />
    </button>
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    height: var(--header-height);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    padding: 0 var(--space-6);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .mark {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border-radius: var(--radius-sm);
    background: var(--color-accent);
    color: var(--color-text-inverse);
  }

  .brand strong {
    color: var(--color-text);
    font-size: var(--text-md);
    font-weight: 800;
    letter-spacing: -0.01em;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: auto;
    flex-shrink: 0;
  }
</style>
