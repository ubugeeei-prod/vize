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
      "src/alert.vue",
      "src/announcer-provider.vue",
      "src/checkbox-control.vue",
      "src/deterministic-id-provider.vue",
      "src/error-summary.vue",
      "src/link-anchor.vue",
      "src/live-region.vue",
      "src/locale-provider.vue",
      "src/portal.vue",
      "src/positioner-arrow.vue",
      "src/positioner.vue",
      "src/presence.vue",
      "src/primitive-element.vue",
      "src/search-field.vue",
      "src/switch-control.vue",
      "src/text-input.vue",
      "src/textarea-control.vue",
      "src/toggle-button.vue",
      "src/transition.vue",
      "src/visually-hidden.vue",
    ],
  );
  assert.deepEqual(
    requests,
    [
      "action-button.vue",
      "alert.vue",
      "announcer-provider.vue",
      "checkbox-control.vue",
      "deterministic-id-provider.vue",
      "error-summary.vue",
      "link-anchor.vue",
      "live-region.vue",
      "locale-provider.vue",
      "portal.vue",
      "positioner-arrow.vue",
      "positioner.vue",
      "presence.vue",
      "primitive-element.vue",
      "search-field.vue",
      "switch-control.vue",
      "text-input.vue",
      "textarea-control.vue",
      "toggle-button.vue",
      "transition.vue",
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
