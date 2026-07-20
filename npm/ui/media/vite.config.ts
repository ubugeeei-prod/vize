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
      source: "src/media-source.ts",
    },
    format: "esm",
    dts: true,
    clean: true,
  },
});
