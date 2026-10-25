import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: { ignorePatterns: ["dist/**"], options: { typeAware: true } },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: ["src/resolver.ts"],
    format: "cjs",
    dts: true,
    deps: { neverBundle: [/^@parcel\//] },
    clean: true,
    minify: false,
    sourcemap: false,
  },
});
