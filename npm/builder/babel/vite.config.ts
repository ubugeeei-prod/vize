import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: { ignorePatterns: ["dist/**"], options: { typeAware: true } },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: ["src/index.ts"],
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
  },
});
