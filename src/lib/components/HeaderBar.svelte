<script lang="ts">
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";
  import flashIcon from "$lib/assets/icons/flash.svg?url";
  import targetIcon from "$lib/assets/icons/target.svg?url";
  import cpuIcon from "$lib/assets/icons/cpu.svg?url";
  import chevronIcon from "$lib/assets/icons/chevron-down.svg?url";
  import sunIcon from "$lib/assets/icons/sun.svg?url";
  import moonIcon from "$lib/assets/icons/moon.svg?url";
  import ProbePicker from "$lib/features/target/ProbePicker.svelte";
  import ChipPicker from "$lib/features/target/ChipPicker.svelte";
  import LinkPopover from "$lib/features/target/LinkPopover.svelte";
  import { target } from "$lib/state/target.svelte";
  import { theme } from "$lib/state/theme.svelte";

  let probeOpen = $state(false);
  let chipOpen = $state(false);
  let linkOpen = $state(false);

  let linkTone = $derived(
    target.link.state === "ok"
      ? "var(--color-success)"
      : target.link.state === "error"
        ? "var(--color-danger)"
        : "var(--color-text-faint)",
  );
</script>

<header class="header">
  <div class="brand">
    <span class="mark"><Icon src={flashIcon} size={16} /></span>
    <div class="brand-copy">
      <strong>FlashDesk</strong>
      <span>固件烧录工具</span>
    </div>
  </div>

  <div class="target-strip">
    <Popover bind:open={probeOpen} width={300}>
      {#snippet trigger({ toggle })}
        <button
          type="button"
          class="strip-btn"
          aria-expanded={probeOpen}
          onclick={toggle}
        >
          <Icon src={targetIcon} size={15} />
          <span class="strip-label"
            >{target.selectedProbeSummary?.product ??
              (target.probe ? target.probe : "自动选择探针")}</span
          >
          <Icon src={chevronIcon} size={12} />
        </button>
      {/snippet}
      {#snippet content({ close })}
        <ProbePicker {close} />
      {/snippet}
    </Popover>

    <Popover bind:open={chipOpen} width={320}>
      {#snippet trigger({ toggle })}
        <button
          type="button"
          class="strip-btn"
          class:muted={!target.chip.trim()}
          aria-expanded={chipOpen}
          onclick={toggle}
        >
          <Icon src={cpuIcon} size={15} />
          <span class="strip-label ui-mono"
            >{target.chip.trim() || "选择目标芯片"}</span
          >
          <Icon src={chevronIcon} size={12} />
        </button>
      {/snippet}
      {#snippet content({ close })}
        <ChipPicker {close} />
      {/snippet}
    </Popover>

    <Popover bind:open={linkOpen} align="end" width={280}>
      {#snippet trigger({ toggle })}
        <button
          type="button"
          class="strip-btn"
          aria-expanded={linkOpen}
          onclick={toggle}
          title="连接参数与状态"
        >
          <span class="ui-dot" style={`--dot-color:${linkTone}`}></span>
          <span class="strip-label"
            >{target.protocol.toUpperCase()} · {target.speedKhz} kHz</span
          >
          <Icon src={chevronIcon} size={12} />
        </button>
      {/snippet}
      {#snippet content()}
        <LinkPopover />
      {/snippet}
    </Popover>
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
    width: 30px;
    height: 30px;
    place-items: center;
    border-radius: var(--radius-md);
    background: var(--color-accent-soft);
    color: var(--color-accent-strong);
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
  }

  .target-strip {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: 1;
    min-width: 0;
  }

  .strip-btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    max-width: 220px;
    min-height: 32px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    background: var(--color-surface-muted);
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 600;
    padding: 0 var(--space-3);
    transition: border-color var(--duration-base) var(--ease-out);
  }

  .strip-btn:hover {
    border-color: var(--color-border-strong);
  }

  .strip-btn.muted {
    color: var(--color-text-muted);
  }

  .strip-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  @media (max-width: 780px) {
    .strip-btn {
      max-width: 140px;
    }
  }
</style>
