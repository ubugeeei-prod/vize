import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

/**
 * Declared browser floor for the packaged stylesheet.
 *
 * Styles are authored in native CSS (nesting, cascade layers, logical
 * properties, native color functions) and down-compiled to this floor by the
 * package build, so `dist/style.css` never depends on the consumer's own CSS
 * toolchain. The floor is the earliest evergreen release line where cascade
 * layers, `:where()`, logical properties, and `oklch()` are all native; only
 * CSS Nesting is newer than the floor and is therefore always flattened.
 * `src/style-pipeline.behavior.md` documents the policy and how consumers
 * override it; `src/style-pipeline.test.ts` pins the observable output.
 */
export const cssBrowserFloor = ["chrome111", "edge111", "firefox113", "safari16.4"];

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "happy-dom",
    include: ["src/**/*.test.ts"],
  },
  lint: {
    ignorePatterns: ["dist/**"],
    options: { typeAware: true },
  },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: {
      index: "src/index.ts",
      button: "src/button.ts",
      checkbox: "src/checkbox.ts",
      collection: "src/collection.ts",
      "composite-navigation": "src/composite-navigation.ts",
      context: "src/context.ts",
      "controllable-state": "src/controllable-state.ts",
      "dismissable-layer": "src/dismissable-layer.ts",
      catalog: "src/family-catalog.ts",
      id: "src/id.ts",
      "inert-outside": "src/inert-outside.ts",
      "interaction-modality": "src/interaction-modality.ts",
      focus: "src/focus.ts",
      "focus-scope": "src/focus-scope.ts",
      "focus-guards": "src/focus-guards.ts",
      hover: "src/hover.ts",
      "long-press": "src/long-press.ts",
      move: "src/move.ts",
      press: "src/press.ts",
      "scroll-lock": "src/scroll-lock.ts",
      "spatial-navigation": "src/spatial-navigation.ts",
      typeahead: "src/typeahead.ts",
      primitive: "src/primitive.ts",
      "visually-hidden": "src/visually-hidden.ts",
      media: "src/media.ts",
      "media-pdf": "src/pdf.ts",
      "media-source": "src/media-source.ts",
    },
    format: "esm",
    dts: { vue: true },
    plugins: [vue()],
    css: {
      inject: true,
      minify: true,
      target: cssBrowserFloor,
    },
    clean: true,
    deps: {
      neverBundle: ["vue"],
    },
  },
});
