import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: { ignorePatterns: ["dist/**"], options: { typeAware: true } },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: ["src/index.ts", "src/loader.ts", "src/style-loader.ts", "src/scope-loader.ts"],
    format: ["esm", "cjs"],
    dts: { resolver: "tsc", eager: true },
    clean: true,
    deps: { alwaysBundle: ["@vizejs/builder-shared"] },
  },
});
