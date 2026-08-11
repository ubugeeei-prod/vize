import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

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
      context: "src/context.ts",
      "controllable-state": "src/controllable-state.ts",
      id: "src/id.ts",
      "interaction-modality": "src/interaction-modality.ts",
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
    },
    clean: true,
    deps: {
      neverBundle: ["vue"],
    },
  },
});
