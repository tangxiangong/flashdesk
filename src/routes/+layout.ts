/**
 * 关闭 SvelteKit 服务端渲染，让 Tauri 桌面端以静态 SPA 方式加载前端。
 *
 * Tauri 没有 Node.js 服务端可用于常规 SSR，因此项目使用 adapter-static
 * 和 index.html fallback。
 *
 * @see https://svelte.dev/docs/kit/single-page-apps
 * @see https://v2.tauri.app/start/frontend/sveltekit/
 */
export const ssr = false;
