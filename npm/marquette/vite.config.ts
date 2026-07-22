import { readFileSync } from "node:fs";
import { defineConfig } from "vite-plus";

/**
 * The canonical contract schemas live next to the native implementation in
 * `crates/vize_marquette/schema`. Emitting them through the bundle keeps
 * `vp pack` self-contained: a post-pack copy step's outputs would escape the
 * task cache, so a cache hit could restore `dist/` without the schemas.
 */
const contractSchemas = [
  "application-contract.schema.json",
  "test-run-evidence.schema.json",
  "test-run-admission.schema.json",
];

export default defineConfig({
  lint: {
    ignorePatterns: ["dist/**"],
    options: {
      typeAware: true,
    },
  },
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: {
    entry: [
      "src/index.ts",
      "src/validate.ts",
      "src/test-run.ts",
      "src/test-run-validate.ts",
      "src/test-run-canonical.ts",
      "src/test-run-admission.ts",
    ],
    format: "esm",
    dts: true,
    clean: true,
    plugins: [
      {
        name: "marquette:emit-contract-schemas",
        generateBundle() {
          for (const schema of contractSchemas) {
            this.emitFile({
              type: "asset",
              fileName: schema,
              source: readFileSync(
                new URL(`../../crates/vize_marquette/schema/${schema}`, import.meta.url),
                "utf8",
              ),
            });
          }
        },
      },
    ],
  },
});
