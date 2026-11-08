import assert from "node:assert/strict";
import { test } from "node:test";

import {
  auditSourceDirectory,
  auditSourceGraph,
} from "../../npm/compose/core/src/test-support/api-documentation.ts";
import { uiFamilyCatalog } from "../../npm/ui/src/catalog/family-catalog.ts";

const uiRoot = new URL("../../npm/ui/", import.meta.url);

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
    name: "@vizejs/fresco",
    source: new URL("../../npm/fresco/src/", import.meta.url),
  },
  {
    name: "@vizejs/marquette",
    source: new URL("../../npm/marquette/src/", import.meta.url),
  },
  {
    name: "@vizejs/devtools",
    source: new URL("../../npm/devtools/src/", import.meta.url),
  },
  {
    name: "@vizejs/ui",
    sourceRoot: uiRoot,
    entrypoints: [
      new URL("src/index.ts", uiRoot),
      ...uiFamilyCatalog.map(({ entryFile }) => new URL(entryFile, uiRoot)),
      new URL("src/families/foundations/theme/theme-scope.ts", uiRoot),
      new URL("src/resolver/resolver.ts", uiRoot),
      new URL("src/families/media/qr-code/qr-code-kanji.ts", uiRoot),
      new URL("src/catalog/family-catalog.ts", uiRoot),
      new URL("src/media/media.ts", uiRoot),
      new URL("src/media/pdf.ts", uiRoot),
      new URL("src/media/media-source.ts", uiRoot),
    ],
  },
] as const;

for (const packageContract of packages) {
  void test(`${packageContract.name} documents every public declaration and option default`, () => {
    const audit =
      "entrypoints" in packageContract
        ? auditSourceGraph(packageContract.entrypoints, packageContract.sourceRoot)
        : auditSourceDirectory(packageContract.source);
    assert.ok(
      audit.declarations > 0,
      `${packageContract.name} must expose documented declarations`,
    );
    if ("entrypoints" in packageContract) {
      assert.ok(
        audit.declarations >= 3_000,
        `${packageContract.name} must retain its public export graph`,
      );
      assert.ok(
        audit.optionalMembers >= 500,
        `${packageContract.name} must retain its public option contracts`,
      );
    }
    assert.deepEqual(audit.problems, []);
  });
}
