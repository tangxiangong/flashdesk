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
    <div class="brand-copy">
      <strong>FlashDesk</strong>
      <span>固件烧录工具</span>
    </div>
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
    padding: 0 var(--space-4);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .mark {
    display: grid;
    width: 28px;
    height: 28px;
    place-items: center;
    border: 1px solid var(--color-accent-border);
    border-radius: var(--radius-sm);
    background: var(--color-accent);
    color: var(--color-text-inverse);
  }

  .brand-copy {
    display: grid;
    line-height: 1.15;
  }

  .brand-copy strong {
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 700;
  }

  .brand-copy span {
    color: var(--color-text-faint);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: auto;
    flex-shrink: 0;
  }
</style>
