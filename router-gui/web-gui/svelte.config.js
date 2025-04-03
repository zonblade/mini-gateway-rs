import adapter from '@sveltejs/adapter-static';
// import { vitePreprocess } from '@sveltejs/kit/vite';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: null,
  kit: {
    adapter: adapter({
      fallback: 'index.html', // SPA mode
      strict: false
    }),
    prerender: {
      entries: [] // Don't prerender any routes
    }
  }
};

export default config;