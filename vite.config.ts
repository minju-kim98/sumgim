import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri prefers port 1420 and does not expose env vars to frontend by default.
// See https://tauri.app/v2/guides/build-your-frontend
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1",
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
