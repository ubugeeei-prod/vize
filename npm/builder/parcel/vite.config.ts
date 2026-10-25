import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: { ignorePatterns: ["dist/**"], options: { typeAware: true } },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: ["src/index.ts", "src/style.ts"],
    format: "cjs",
    dts: true,
    clean: true,
    minify: false,
    sourcemap: false,
  },
});
