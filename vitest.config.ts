import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";

export default defineConfig({
  plugins: [sveltekit()],

  resolve: {
    conditions: ["browser"],
    alias: {
      "@tauri-apps/plugin-dialog": "/Users/k4iju/Projects/PDFCompressor/src/lib/__mocks__/tauri-plugin-dialog.ts",
    },
  },

  test: {
    globals: true,
    environment: "happy-dom",
    setupFiles: ["src/test/setup.ts"],
    include: ["src/test/**/*.test.ts"],
    environmentMatchGlobs: [
      ["src/test/queueStore.test.ts", "node"],
      ["src/test/settingsStore.test.ts", "node"],
    ],
    server: {
      deps: {
        inline: [/svelte/],
      },
    },
  },
});
