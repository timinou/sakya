import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const host = process.env.TAURI_DEV_HOST;
const port = parseInt(process.env.VITE_DEV_PORT || "1420", 10);

// https://vite.dev/config/
export default defineConfig({
  // @ts-expect-error sveltekit plugin types may diverge from vite's vendored types
  plugins: [sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. use the port selected by scripts/dev.ts (defaults to 1420)
  server: {
    port,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: port + 1,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: [
        "**/src-tauri/**",
        "**/playwright-report/**",
        "**/test-results/**",
      ],
    },
  },
});
