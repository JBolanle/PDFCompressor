import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";
import { fileURLToPath, URL } from "url";

export default defineConfig({
  plugins: [sveltekit()],

  resolve: {
    conditions: ["browser"],
    alias: {
      "@tauri-apps/plugin-dialog": fileURLToPath(
        new URL("./src/lib/mocks/tauri-plugin-dialog.ts", import.meta.url)
      ),
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
