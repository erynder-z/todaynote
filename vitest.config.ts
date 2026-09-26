import path from "node:path";
import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		environment: "node",
		globals: true,
		include: ["src/**/*.{test,spec}.{js,ts}"],
		coverage: {
			provider: "v8",
			reporter: ["text", "json", "html"],
			include: ["src/lib/utils/**/*.ts"],
		},
	},
	resolve: {
		alias: {
			$lib: path.resolve("./src/lib"),
		},
	},
});
