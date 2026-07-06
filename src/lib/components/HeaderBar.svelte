<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import AboutDialog from "./AboutDialog.svelte";
  import Popover from "./Popover.svelte";
  import UpdateDialog from "./UpdateDialog.svelte";
  import type { ThemePreference } from "$lib/state/theme.svelte";
  import { listenToAppMenuEvents } from "$lib/api/tauri";
  import appIcon from "$lib/assets/app-icon.png";
  import downloadIcon from "$lib/assets/icons/download.svg?url";
  import infoIcon from "$lib/assets/icons/info.svg?url";
  import sunIcon from "$lib/assets/icons/sun.svg?url";
  import moonIcon from "$lib/assets/icons/moon.svg?url";
  import monitorIcon from "$lib/assets/icons/monitor.svg?url";
  import paletteIcon from "$lib/assets/icons/palette.svg?url";
  import { theme } from "$lib/state/theme.svelte";
  import { updates } from "$lib/state/updates.svelte";

  let themeOpen = $state(false);
  let aboutOpen = $state(false);

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

  onMount(() => {
    const unlisten = listenToAppMenuEvents((action) => {
      if (action === "about") {
        aboutOpen = true;
      } else if (action === "check-update") {
        updates.openPanel();
      }
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  });
</script>

<header class="header">
  <div class="brand" aria-label="FlashDesk">
    <img class="brand-icon" src={appIcon} alt="" aria-hidden="true" />
    <span class="brand-label">固件烧录工具</span>
  </div>

  <div class="actions">
    <button
      type="button"
      class="ui-btn ui-btn--ghost ui-btn--icon"
      title="检查更新"
      aria-label="检查更新"
      onclick={() => updates.openPanel()}
    >
      <Icon src={downloadIcon} size={16} />
    </button>

    <button
      type="button"
      class="ui-btn ui-btn--ghost ui-btn--icon"
      title="关于"
      aria-label="关于"
      onclick={() => (aboutOpen = true)}
    >
      <Icon src={infoIcon} size={16} />
    </button>

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

<UpdateDialog />
<AboutDialog bind:open={aboutOpen} />

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

  .brand-icon {
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    display: block;
    box-shadow: 0 1px 2px color-mix(in srgb, var(--color-text) 10%, transparent);
  }

  .brand-label {
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 800;
    letter-spacing: 0;
    white-space: nowrap;
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
