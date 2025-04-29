// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    // Configure SvelteKit to use the custom source directory
    files: {
      lib: 'src-custom/lib',
      routes: 'src-custom/routes',
      appTemplate: 'src-custom/app.html',
    }
  },
};

export default config;
