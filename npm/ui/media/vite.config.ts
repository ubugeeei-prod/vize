import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: ["dist/**"],
    options: { typeAware: true },
  },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: {
      index: "src/index.ts",
      pdf: "src/pdf.ts",
      qr: "src/qr.ts",
      source: "src/media-source.ts",
    },
    format: "esm",
    dts: { vue: true },
    plugins: [vue()],
    clean: true,
    deps: {
      neverBundle: ["vue", "uqr"],
    },
  },
});
