<script lang="ts">
  import Icon from "./Icon.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";

  /** 二次确认按钮 props。 */
  interface Props {
    /** 初始按钮文案。 */
    label: string;
    /** 进入确认态后的按钮文案。 */
    confirmLabel?: string;
    /** 是否禁用按钮。 */
    disabled?: boolean;
    /** 用户二次确认后的回调。 */
    onconfirm: () => void;
  }

  let {
    label,
    confirmLabel = "确认",
    disabled = false,
    onconfirm,
  }: Props = $props();

  let armed = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function arm() {
    armed = true;
    clearTimeout(timer);
    timer = setTimeout(() => {
      armed = false;
    }, 4000);
  }

  function disarm() {
    armed = false;
    clearTimeout(timer);
  }

  function handleClick() {
    if (armed) {
      disarm();
      onconfirm();
    } else {
      arm();
    }
  }
</script>

<div class="confirm">
  <button
    type="button"
    class="ui-btn ui-btn--danger confirm-btn"
    class:armed
    {disabled}
    onclick={handleClick}
  >
    <Icon src={alertIcon} size={15} />
    {armed ? confirmLabel : label}
  </button>

  {#if armed}
    <button
      type="button"
      class="ui-btn ui-btn--ghost ui-btn--icon"
      aria-label="取消"
      title="取消"
      onclick={disarm}
    >
      <Icon src={xIcon} size={14} />
    </button>
  {/if}
</div>

<style>
  .confirm {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .confirm-btn.armed {
    background: var(--color-danger);
    border-color: var(--color-danger);
    color: var(--color-text-inverse);
    animation: shake 240ms var(--ease-out);
  }

  @keyframes shake {
    0%,
    100% {
      transform: translateX(0);
    }
    25% {
      transform: translateX(-2px);
    }
    75% {
      transform: translateX(2px);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .confirm-btn.armed {
      animation: none;
    }
  }
</style>
