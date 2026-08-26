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
      "src/DeterministicIdProvider.vue",
      "src/PrimitiveElement.vue",
      "src/VisuallyHidden.vue",
      "src/announcer-provider.vue",
      "src/error-summary.vue",
      "src/live-region.vue",
      "src/locale-provider.vue",
      "src/portal.vue",
      "src/positioner-arrow.vue",
      "src/positioner.vue",
      "src/presence.vue",
      "src/transition.vue",
    ],
  );
  assert.deepEqual(
    requests,
    [
      "ActionButton.vue",
      "CheckboxControl.vue",
      "DeterministicIdProvider.vue",
      "PrimitiveElement.vue",
      "VisuallyHidden.vue",
      "announcer-provider.vue",
      "error-summary.vue",
      "live-region.vue",
      "locale-provider.vue",
      "portal.vue",
      "positioner-arrow.vue",
      "positioner.vue",
      "presence.vue",
      "transition.vue",
    ].map((basename) => ({
      filename: path.resolve("src", basename),
      preset: "opinionated" as const,
      typeAware: true as const,
      helpLevel: "short" as const,
    })),
  );
  assert.equal(formatSfcLintResults(results), "");
});
