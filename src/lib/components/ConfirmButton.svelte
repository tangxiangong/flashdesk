<script lang="ts">
  import Icon from "./Icon.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";

  let {
    label,
    confirmLabel = "再次点击确认",
    disabled = false,
    onconfirm,
  }: {
    label: string;
    confirmLabel?: string;
    disabled?: boolean;
    onconfirm: () => void;
  } = $props();

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
