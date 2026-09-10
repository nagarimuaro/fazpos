import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  root: ".",
  clearScreen: false,
  resolve: {
    alias: { "@": import.meta.dirname + "/src-ui" },
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: [
        "**/src-tauri/**",
        "**/src/**",
        "**/target/**",
        "**/*.db*",
        "**/*.bin",
        "**/.git/**",
      ],
    },
  },
});
