import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({
  vue: {
    compiler: {
      vapor: false,
      ssr: false,
      sourceMap: true,
      hoistStatic: true,
      cacheHandlers: true,
    },
    vite: {
      scanPatterns: ["src/**/*.vue", "src/**/*.jsx", "src/**/*.tsx"],
      ignorePatterns: ["node_modules/**", "dist/**"],
    },
    linter: {
      enabled: true,
      categories: {
        correctness: "error",
        suspicious: "warn",
      },
    },
    typeChecker: {
      enabled: true,
      jsxTypecheck: true,
      strict: false,
      checkProps: true,
      checkEmits: true,
    },
    formatter: {
      printWidth: 100,
      tabWidth: 2,
      semi: true,
      singleQuote: true,
      trailingComma: "all",
    },
    musea: {
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
      vrt: {
        threshold: 0.1,
        viewports: [{ width: 1280, height: 720, name: "desktop" }],
      },
    },
    globalTypes: {
      $t: "(key: string) => string",
      $router: { type: 'import("vue-router").Router', defaultValue: "undefined" },
    },
  },
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  lint: {
    ignorePatterns: ["dist/**"],
  },
});
