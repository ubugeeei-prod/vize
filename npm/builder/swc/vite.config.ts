import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: { ignorePatterns: ["dist/**"], options: { typeAware: true } },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: ["src/sfc.ts"],
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
    deps: { alwaysBundle: ["@vizejs/builder-shared"] },
  },
});
