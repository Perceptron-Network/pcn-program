import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  build: {
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: "react-vendor",
              test: /node_modules[\\/](react|react-dom)/,
              priority: 20,
            },
            {
              name: "solana-vendor",
              test: /node_modules[\\/](@solana|@wallet-standard)/,
              maxSize: 280_000,
              priority: 15,
            },
            {
              name: "vendor",
              test: /node_modules/,
              maxSize: 260_000,
              priority: 10,
            },
          ],
        },
        strictExecutionOrder: true,
      },
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@pcn-client": path.resolve(__dirname, "../codama/codama-ts"),
      buffer: path.resolve(__dirname, "./node_modules/buffer/index.js"),
    },
    dedupe: ["@solana/kit", "@solana/web3.js", "react", "react-dom"],
  },
  server: {
    fs: {
      allow: [path.resolve(__dirname, "..")],
    },
  },
});
