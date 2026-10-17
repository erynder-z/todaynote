// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
	plugins: [
		sveltekit({
			preprocess: vitePreprocess(),
			adapter: adapter({ fallback: "index.html" }),
		}),
	],
	clearScreen: false,

	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: "ws",
					host,
					port: 1421,
				}
			: undefined,
		watch: {
			ignored: ["**/src-tauri/**"],
		},
	},

	build: {
		chunkSizeWarningLimit: 1000,
		// Optimize chunking strategy
		rollupOptions: {
			output: {
				manualChunks /** @type {string | string[]} */: (
					/** @type {string} */ id,
				) => {
					// Put all milkdown/prosemirror libraries in a dedicated vendor chunk
					if (
						id.includes("node_modules/@milkdown") ||
						id.includes("node_modules/prosemirror")
					) {
						return "editor-vendor";
					}
				},
			},
		},
	},
}));
