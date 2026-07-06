<script lang="ts">
  /** 图标组件 props。 */
  interface Props {
    /** 作为 CSS mask 使用的 SVG 资源 URL。 */
    src: string;
    /** 图标正方形尺寸，单位 px。 */
    size?: number;
    /** 无障碍标签；装饰性图标应省略。 */
    label?: string;
  }

  let { src, size = 18, label }: Props = $props();
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
