import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    compiler: {
      vapor: false,
      ssr: false,
      sourceMap: true,
    },
    vite: {
      scanPatterns: ["src/**/*.vue"],
      ignorePatterns: ["node_modules/**", "dist/**"],
    },
    musea: {
      include: ["src/**/*.vue"],
      basePath: "/__musea__",
      inlineArt: true,
      vrt: {
        threshold: 0.1,
        viewports: [{ width: 1280, height: 720, name: "desktop" }],
      },
    },
  },
  lint: {
    ignorePatterns: [
      "dist/**",
      "node_modules/**",
      "playwright-report/**",
      "e2e/vrt/test-results/**",
    ],
    options: {
      typeAware: true,
    },
  },
  fmt: {
    ignorePatterns: ["dist/**", "playwright-report/**", "e2e/vrt/test-results/**"],
  },
});
