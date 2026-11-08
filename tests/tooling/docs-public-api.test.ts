import assert from "node:assert/strict";
import { test } from "node:test";

import {
  auditSourceDirectory,
  auditSourceGraph,
} from "../../npm/compose/core/src/test-support/api-documentation.ts";
import { uiFamilyCatalog } from "../../npm/ui/src/catalog/family-catalog.ts";

const uiRoot = new URL("../../npm/ui/", import.meta.url);
const npmRoot = new URL("../../npm/", import.meta.url);

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
    name: "@vizejs/state",
    source: new URL("../../npm/compose/state/src/", import.meta.url),
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
    name: "@vizejs/unplugin",
    sourceRoot: npmRoot,
    minimumDeclarations: 20,
    minimumOptionalMembers: 30,
    entrypoints: [
      new URL("builder/unplugin/src/index.ts", npmRoot),
      new URL("builder/unplugin/src/esbuild.ts", npmRoot),
      new URL("builder/unplugin/src/bun.ts", npmRoot),
      new URL("builder/unplugin/src/rollup.ts", npmRoot),
      new URL("builder/unplugin/src/rolldown.ts", npmRoot),
      new URL("builder/unplugin/src/webpack.ts", npmRoot),
      new URL("builder/unplugin/src/babel.ts", npmRoot),
    ],
  },
  {
    name: "eslint-plugin-vize",
    source: new URL("../../npm/linter/eslint/src/", import.meta.url),
  },
  {
    name: "@vizejs/nuxt-lint-config",
    source: new URL("../../npm/framework/nuxt-lint-config/src/", import.meta.url),
  },
  {
    name: "oxlint-plugin-vize",
    sourceRoot: npmRoot,
    minimumDeclarations: 20,
    minimumOptionalMembers: 10,
    entrypoints: [
      new URL("linter/oxlint/src/index.ts", npmRoot),
      new URL("linter/oxlint/src/eslint.ts", npmRoot),
    ],
  },
  {
    name: "@vizejs/volar-plugin",
    source: new URL("../../npm/volar/src/", import.meta.url),
  },
  {
    name: "@vizejs/ui",
    sourceRoot: uiRoot,
    minimumDeclarations: 3_000,
    minimumOptionalMembers: 500,
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
  {
    name: "@vizejs/plugin-sdk",
    sourceRoot: new URL("../../npm/plugin-sdk/", import.meta.url),
    minimumDeclarations: 100,
    entrypoints: [
      new URL("../../npm/plugin-sdk/index.d.ts", import.meta.url),
      new URL("../../npm/plugin-sdk/fixes.d.ts", import.meta.url),
      new URL("../../npm/plugin-sdk/sandbox.d.ts", import.meta.url),
      new URL("../../npm/plugin-sdk/visitor.d.ts", import.meta.url),
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
    if ("minimumDeclarations" in packageContract) {
      assert.ok(
        audit.declarations >= packageContract.minimumDeclarations,
        `${packageContract.name} must retain its public export graph`,
      );
    }
    if ("minimumOptionalMembers" in packageContract) {
      assert.ok(
        audit.optionalMembers >= packageContract.minimumOptionalMembers,
        `${packageContract.name} must retain its public option contracts`,
      );
    }
    assert.deepEqual(audit.problems, []);
  });
}
