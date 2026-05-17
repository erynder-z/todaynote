import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
	plugins: [sveltekit()],

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
				manualChunks: (/** @type {string | string[]} */ id) => {
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
