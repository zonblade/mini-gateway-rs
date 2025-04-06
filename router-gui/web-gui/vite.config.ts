import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit()
	],
	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
	  port: 24041,
	  strictPort: true,
	  host: host || false,
	  hmr: host
		? {
			protocol: "ws",
			host,
			port: 24041,
		  }
		: undefined,
	  watch: {
		// 3. tell vite to ignore watching `src-tauri`
		ignored: ["**/src-tauri/**"],
	  },
	},
});
