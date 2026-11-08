import assert from "node:assert/strict";
import { test } from "node:test";

import { auditSourceDirectory } from "../../npm/compose/core/src/test-support/api-documentation.ts";

const packages = [
  {
    name: "@vizejs/composable",
    source: new URL("../../npm/compose/core/src/", import.meta.url),
  },
  {
    name: "@vizejs/data",
    source: new URL("../../npm/compose/data/src/", import.meta.url),
  },
  {
    name: "@vizejs/router",
    source: new URL("../../npm/compose/router/src/", import.meta.url),
  },
  {
    name: "@vizejs/marquette",
    source: new URL("../../npm/marquette/src/", import.meta.url),
  },
  {
    name: "@vizejs/devtools",
    source: new URL("../../npm/devtools/src/", import.meta.url),
  },
] as const;

for (const packageContract of packages) {
  void test(`${packageContract.name} documents every public declaration and option default`, () => {
    const audit = auditSourceDirectory(packageContract.source);
    assert.ok(
      audit.declarations > 0,
      `${packageContract.name} must expose documented declarations`,
    );
    assert.deepEqual(audit.problems, []);
  });
}
