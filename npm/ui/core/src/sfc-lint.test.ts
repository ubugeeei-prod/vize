import assert from "node:assert/strict";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import { formatSfcLintResults, lintSfcFiles } from "@vizejs/ui-tooling/lint-sfc";
import type { SfcLintFunction } from "@vizejs/ui-tooling/lint-sfc";

test("discovers every SFC with the opinionated Vize contract", async () => {
  const requests: Parameters<SfcLintFunction>[1][] = [];
  const lint: SfcLintFunction = (_source, options) => {
    requests.push(options);
    return { diagnostics: [] };
  };
  const results = await lintSfcFiles(lint);

  assert.deepEqual(
    results.map((result) => result.filename),
    [
      "src/ActionButton.vue",
      "src/CheckboxControl.vue",
      "src/PrimitiveElement.vue",
      "src/VisuallyHidden.vue",
    ],
  );
  assert.deepEqual(
    requests,
    ["ActionButton.vue", "CheckboxControl.vue", "PrimitiveElement.vue", "VisuallyHidden.vue"].map(
      (basename) => ({
        filename: path.resolve("src", basename),
        preset: "opinionated" as const,
        typeAware: true as const,
        helpLevel: "short" as const,
      }),
    ),
  );
  assert.equal(formatSfcLintResults(results), "");
});
