import assert from "node:assert/strict";
import { test } from "node:test";

import { formatSfcLintResults, lintSfcFiles } from "@vizeui/tooling/lint-sfc";
import type { SfcLintFunction } from "@vizeui/tooling/lint-sfc";

void test("discovers every SFC with the opinionated Vize contract", async () => {
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
  assert.deepEqual(requests, [
    {
      filename: new URL("./ActionButton.vue", import.meta.url).pathname,
      preset: "opinionated",
      typeAware: true,
      helpLevel: "short",
    },
    {
      filename: new URL("./CheckboxControl.vue", import.meta.url).pathname,
      preset: "opinionated",
      typeAware: true,
      helpLevel: "short",
    },
    {
      filename: new URL("./PrimitiveElement.vue", import.meta.url).pathname,
      preset: "opinionated",
      typeAware: true,
      helpLevel: "short",
    },
    {
      filename: new URL("./VisuallyHidden.vue", import.meta.url).pathname,
      preset: "opinionated",
      typeAware: true,
      helpLevel: "short",
    },
  ]);
  assert.equal(formatSfcLintResults(results), "");
});
