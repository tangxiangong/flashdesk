<script lang="ts">
  import Icon from "./Icon.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import checkIcon from "$lib/assets/icons/check.svg?url";
  import downloadIcon from "$lib/assets/icons/download.svg?url";
  import refreshIcon from "$lib/assets/icons/refresh.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";
  import { updates } from "$lib/state/updates.svelte";

  const isBusy = $derived(
    updates.status === "checking" || updates.status === "downloading",
  );

  function formatBytes(value: number) {
    if (value <= 0) {
      return "0 B";
    }

    const units = ["B", "KB", "MB", "GB"];
    const index = Math.min(
      units.length - 1,
      Math.floor(Math.log(value) / Math.log(1024)),
    );

    return `${(value / 1024 ** index).toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && updates.open && !isBusy) {
      updates.close();
    }
  }

  function onBackdropClick(event: MouseEvent) {
    if (!isBusy && event.target === event.currentTarget) {
      updates.close();
    }
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

{#if updates.open}
  <div class="update-backdrop" role="presentation" onclick={onBackdropClick}>
    <div
      class="update-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="update-title"
      tabindex="-1"
    >
      <header class="update-header">
        <div class="update-title">
          <span class="update-mark">
            <Icon src={downloadIcon} size={17} />
          </span>
          <div>
            <h2 id="update-title">软件更新</h2>
            <p>检查、下载并安装新版本</p>
          </div>
        </div>

        <button
          type="button"
          class="ui-btn ui-btn--ghost ui-btn--icon"
          aria-label="关闭"
          disabled={isBusy}
          onclick={() => updates.close()}
        >
          <Icon src={xIcon} size={15} />
        </button>
      </header>

      <div class="update-body">
        {#if updates.status === "checking"}
          <div class="update-state">
            <span class="spinner"><Icon src={refreshIcon} size={20} /></span>
            <strong>正在检查更新</strong>
            <p>正在连接发布通道，请稍候。</p>
          </div>
        {:else if updates.status === "current"}
          <div class="update-state">
            <span class="state-icon state-icon--success">
              <Icon src={checkIcon} size={20} />
            </span>
            <strong>已是最新版本</strong>
            <p>当前安装的 FlashDesk 不需要更新。</p>
          </div>
        {:else if updates.status === "available" && updates.update}
          <div class="update-summary">
            <span class="ui-badge ui-badge--accent">发现新版本</span>
            <h3>{updates.update.version}</h3>
            {#if updates.update.body}
              <p>{updates.update.body}</p>
            {:else}
              <p>新版本已准备好，可以下载并安装。</p>
            {/if}
          </div>
        {:else if updates.status === "downloading"}
          <div class="update-progress">
            <div class="progress-copy">
              <strong>正在下载并安装</strong>
              <span>
                {#if updates.total}
                  {formatBytes(updates.downloaded)} / {formatBytes(
                    updates.total,
                  )}
                {:else}
                  {formatBytes(updates.downloaded)}
                {/if}
              </span>
            </div>
            <div class="progress-track" aria-label="下载进度">
              <span style={`width:${updates.progressPercent}%;`}></span>
            </div>
          </div>
        {:else if updates.status === "ready"}
          <div class="update-state">
            <span class="state-icon state-icon--success">
              <Icon src={checkIcon} size={20} />
            </span>
            <strong>更新已安装</strong>
            <p>重启应用后将使用新版本。</p>
          </div>
        {:else if updates.status === "error"}
          <div class="update-state">
            <span class="state-icon state-icon--danger">
              <Icon src={alertIcon} size={20} />
            </span>
            <strong>更新失败</strong>
            <p>{updates.error || "无法完成更新操作"}</p>
          </div>
        {:else}
          <div class="update-state">
            <span class="state-icon">
              <Icon src={downloadIcon} size={20} />
            </span>
            <strong>检查更新</strong>
            <p>从发布通道获取最新版本信息。</p>
          </div>
        {/if}
      </div>

      <footer class="update-actions">
        {#if updates.status === "available"}
          <button
            type="button"
            class="ui-btn ui-btn--primary"
            onclick={() => void updates.downloadAndInstall()}
          >
            <Icon src={downloadIcon} size={15} />
            下载并安装
          </button>
        {:else if updates.status === "ready"}
          <button
            type="button"
            class="ui-btn ui-btn--primary"
            onclick={() => void updates.restart()}
          >
            重启应用
          </button>
        {:else if updates.status === "checking" || updates.status === "downloading"}
          <button type="button" class="ui-btn ui-btn--primary" disabled>
            请稍候
          </button>
        {:else}
          <button
            type="button"
            class="ui-btn ui-btn--primary"
            onclick={() => void updates.checkForUpdates()}
          >
            <Icon src={refreshIcon} size={15} />
            重新检查
          </button>
        {/if}
      </footer>
    </div>
  </div>
{/if}

<style>
  .update-backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--color-bg) 74%, transparent);
    backdrop-filter: blur(12px);
    padding: var(--space-5);
  }

  .update-dialog {
    display: grid;
    width: min(460px, 100%);
    max-height: min(620px, calc(100dvh - 40px));
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    box-shadow: var(--shadow-pop);
  }

  .update-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-5);
  }

  .update-title {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .update-mark,
  .state-icon {
    display: grid;
    width: 34px;
    height: 34px;
    flex-shrink: 0;
    place-items: center;
    border-radius: var(--radius-sm);
    background: var(--color-accent-soft);
    color: var(--color-accent-strong);
  }

  .state-icon--success {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .state-icon--danger {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .update-title h2 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-lg);
    font-weight: 800;
    letter-spacing: 0;
  }

  .update-title p,
  .update-state p,
  .update-summary p {
    margin: 3px 0 0;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .update-body {
    overflow: auto;
    padding: var(--space-5);
  }

  .update-state {
    display: grid;
    justify-items: center;
    gap: var(--space-2);
    padding: var(--space-5) var(--space-2);
    text-align: center;
  }

  .update-state strong {
    color: var(--color-text);
    font-size: var(--text-base);
    font-weight: 800;
  }

  .spinner {
    display: grid;
    width: 40px;
    height: 40px;
    place-items: center;
    color: var(--color-accent-strong);
    animation: spin 1s linear infinite;
  }

  .update-summary {
    display: grid;
    gap: var(--space-2);
  }

  .update-summary h3 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-2xl);
    font-weight: 800;
    letter-spacing: 0;
  }

  .update-progress {
    display: grid;
    gap: var(--space-3);
    padding: var(--space-3) 0;
  }

  .progress-copy {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    color: var(--color-text);
    font-size: var(--text-sm);
  }

  .progress-copy span {
    color: var(--color-text-muted);
  }

  .progress-track {
    height: 8px;
    overflow: hidden;
    border-radius: var(--radius-pill);
    background: var(--color-surface-inset);
  }

  .progress-track span {
    display: block;
    height: 100%;
    min-width: 3px;
    border-radius: inherit;
    background: var(--color-accent);
    transition: width var(--duration-base) var(--ease-out);
  }

  .update-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    border-top: 1px solid var(--color-border);
    padding: var(--space-4) var(--space-5);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 520px) {
    .update-actions {
      display: grid;
    }
  }
</style>
