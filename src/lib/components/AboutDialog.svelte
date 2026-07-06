<script lang="ts">
  import Icon from "./Icon.svelte";
  import appPackage from "../../../package.json";
  import appIcon from "$lib/assets/app-icon.png";
  import infoIcon from "$lib/assets/icons/info.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";

  /** 关于窗口 props。 */
  interface Props {
    /** 是否打开窗口，支持双向绑定。 */
    open?: boolean;
  }

  let { open = $bindable(false) }: Props = $props();

  const appDescription = "用于固件烧录流程的桌面工具。";

  const aboutItems = [
    {
      label: "版本",
      value: `v${appPackage.version}`,
    },
    {
      label: "简介",
      value: appDescription,
    },
    {
      label: "作者",
      value: "tangxiangong <tangxiangong@gmail.com>",
    },
    {
      label: "License",
      value: appPackage.license,
    },
  ];

  function close() {
    open = false;
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && open) {
      close();
    }
  }

  function onBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

{#if open}
  <div class="about-backdrop" role="presentation" onclick={onBackdropClick}>
    <div
      class="about-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-title"
      tabindex="-1"
    >
      <header class="about-header">
        <div class="about-title">
          <span class="about-mark">
            <Icon src={infoIcon} size={17} />
          </span>
          <div>
            <h2 id="about-title">关于 FlashDesk</h2>
            <p>应用信息</p>
          </div>
        </div>

        <button
          type="button"
          class="ui-btn ui-btn--ghost ui-btn--icon"
          aria-label="关闭"
          onclick={close}
        >
          <Icon src={xIcon} size={15} />
        </button>
      </header>

      <div class="about-body">
        <div class="about-hero">
          <img class="app-icon" src={appIcon} alt="FlashDesk icon" />
          <div class="app-copy">
            <span class="ui-badge ui-badge--accent">v{appPackage.version}</span>
            <strong>FlashDesk</strong>
            <p>{appDescription}</p>
          </div>
        </div>

        <dl class="about-list">
          {#each aboutItems as item}
            <div>
              <dt>{item.label}</dt>
              <dd>{item.value}</dd>
            </div>
          {/each}
        </dl>
      </div>
    </div>
  </div>
{/if}

<style>
  .about-backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--color-bg) 74%, transparent);
    backdrop-filter: blur(12px);
    padding: var(--space-5);
  }

  .about-dialog {
    display: grid;
    width: min(460px, 100%);
    max-height: min(620px, calc(100dvh - 40px));
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    box-shadow: var(--shadow-pop);
  }

  .about-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-5);
  }

  .about-title {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .about-mark {
    display: grid;
    width: 34px;
    height: 34px;
    flex-shrink: 0;
    place-items: center;
    border-radius: var(--radius-sm);
    background: var(--color-accent-soft);
    color: var(--color-accent-strong);
  }

  .about-title h2 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-lg);
    font-weight: 800;
    letter-spacing: 0;
  }

  .about-title p {
    margin: 3px 0 0;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .about-body {
    display: grid;
    gap: var(--space-5);
    overflow: auto;
    padding: var(--space-5);
  }

  .about-hero {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: var(--space-4);
  }

  .app-icon {
    width: 82px;
    height: 82px;
    border-radius: var(--radius-lg);
    box-shadow:
      0 10px 24px color-mix(in srgb, var(--color-accent) 18%, transparent),
      0 1px 2px color-mix(in srgb, var(--color-text) 10%, transparent);
  }

  .app-copy {
    display: grid;
    justify-items: start;
    gap: var(--space-2);
    min-width: 0;
  }

  .app-copy strong {
    color: var(--color-text);
    font-size: var(--text-2xl);
    font-weight: 850;
    letter-spacing: 0;
    line-height: 1.1;
  }

  .app-copy p {
    margin: 0;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .about-list {
    display: grid;
    gap: 0;
    margin: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-surface-inset);
  }

  .about-list div {
    display: grid;
    grid-template-columns: 88px minmax(0, 1fr);
    gap: var(--space-3);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-3) var(--space-4);
  }

  .about-list div:last-child {
    border-bottom: 0;
  }

  .about-list dt {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    font-weight: 800;
  }

  .about-list dd {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-sm);
    line-height: 1.45;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  @media (max-width: 520px) {
    .about-hero {
      grid-template-columns: 1fr;
    }

    .about-list div {
      grid-template-columns: 1fr;
      gap: 3px;
    }
  }
</style>
