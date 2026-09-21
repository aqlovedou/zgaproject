import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  // Tauri 固定用 5173，端口被占时直接报错而不是悄悄换一个
  server: { port: 5173, strictPort: true },
  build: { target: "es2021", sourcemap: false },
});
