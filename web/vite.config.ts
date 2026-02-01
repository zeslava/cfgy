import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    outDir: "dist",
    // Для SPA: все роуты должны вести на index.html
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      // Прокси всех API запросов на сервер cfgy-server
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
        secure: false,
        ws: true, // Поддержка WebSocket для HMR
      },
      // Прокси health check эндпоинта
      "/health": {
        target: "http://localhost:8080",
        changeOrigin: true,
        secure: false,
      },
      // Прокси ZID OAuth маршрутов (server -> ZID -> callback -> server)
      "/auth/zid": {
        target: "http://localhost:8080",
        changeOrigin: true,
        secure: false,
      },
      "/oauth/zid": {
        target: "http://localhost:8080",
        changeOrigin: true,
        secure: false,
      },
    },
  },
});
