<script lang="ts">
  let {
    src,
    size = 18,
    label,
  }: { src: string; size?: number; label?: string } = $props();
</script>

<span
  class="icon"
  style={`width:${size}px;height:${size}px;-webkit-mask-image:url("${src}");mask-image:url("${src}");`}
  role={label ? "img" : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : "true"}
></span>

<style>
  .icon {
    display: inline-block;
    flex-shrink: 0;
    background-color: currentColor;
    -webkit-mask-repeat: no-repeat;
    mask-repeat: no-repeat;
    -webkit-mask-position: center;
    mask-position: center;
    -webkit-mask-size: contain;
    mask-size: contain;
  }
</style>
