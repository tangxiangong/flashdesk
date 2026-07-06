<script lang="ts">
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";
  import type { ThemePreference } from "$lib/state/theme.svelte";
  import flashIcon from "$lib/assets/icons/flash.svg?url";
  import sunIcon from "$lib/assets/icons/sun.svg?url";
  import moonIcon from "$lib/assets/icons/moon.svg?url";
  import monitorIcon from "$lib/assets/icons/monitor.svg?url";
  import paletteIcon from "$lib/assets/icons/palette.svg?url";
  import { theme } from "$lib/state/theme.svelte";

  let themeOpen = $state(false);

  const themeOptions: {
    value: ThemePreference;
    label: string;
    icon: string;
  }[] = [
    { value: "light", label: "浅色", icon: sunIcon },
    { value: "dark", label: "深色", icon: moonIcon },
    { value: "system", label: "系统", icon: monitorIcon },
  ];

  function activeThemeLabel() {
    return (
      themeOptions.find((option) => option.value === theme.mode)?.label ??
      "系统"
    );
  }
</script>

<header class="header">
  <div class="brand">
    <span class="mark"><Icon src={flashIcon} size={16} /></span>
    <strong>FlashDesk</strong>
  </div>

  <div class="actions">
    <Popover bind:open={themeOpen} align="end" width={176}>
      {#snippet trigger({ toggle })}
        <button
          type="button"
          class="ui-btn ui-btn--ghost ui-btn--icon"
          title={`主题：${activeThemeLabel()}`}
          aria-label={`主题：${activeThemeLabel()}`}
          aria-haspopup="menu"
          aria-expanded={themeOpen}
          onclick={toggle}
        >
          <Icon src={paletteIcon} size={16} />
        </button>
      {/snippet}

      {#snippet content({ close })}
        <div class="theme-menu" role="menu" aria-label="主题">
          {#each themeOptions as option}
            <button
              type="button"
              class="theme-option"
              class:is-active={theme.mode === option.value}
              role="menuitemradio"
              aria-checked={theme.mode === option.value}
              onclick={() => {
                theme.set(option.value);
                close();
              }}
            >
              <Icon src={option.icon} size={15} />
              <span>{option.label}</span>
            </button>
          {/each}
        </div>
      {/snippet}
    </Popover>
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

  .theme-menu {
    display: grid;
    gap: 3px;
  }

  .theme-option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 34px;
    width: 100%;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-sm);
    font-weight: 650;
    padding: 0 var(--space-3);
    text-align: left;
    transition:
      background var(--duration-base) var(--ease-out),
      color var(--duration-base) var(--ease-out);
  }

  .theme-option:hover {
    background: var(--color-surface-muted);
    color: var(--color-text);
  }

  .theme-option.is-active {
    background: var(--color-accent-soft);
    color: var(--color-accent-strong);
  }
</style>
