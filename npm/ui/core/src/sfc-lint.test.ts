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
      "src/action-button.vue",
      "src/checkbox-control.vue",
      "src/deterministic-id-provider.vue",
      "src/primitive-element.vue",
      "src/visually-hidden.vue",
    ],
  );
  assert.deepEqual(
    requests,
    [
      "action-button.vue",
      "checkbox-control.vue",
      "deterministic-id-provider.vue",
      "primitive-element.vue",
      "visually-hidden.vue",
    ].map((basename) => ({
      filename: path.resolve("src", basename),
      preset: "opinionated" as const,
      typeAware: true as const,
      helpLevel: "short" as const,
    })),
  );
  assert.equal(formatSfcLintResults(results), "");
});
