import assert from "node:assert/strict";
import { globSync } from "node:fs";
import { readFile } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import { formatSfcLintResults, lintSfcFiles } from "../../scripts/source-quality/lint-sfc.ts";
import type { SfcLintFunction } from "../../scripts/source-quality/lint-sfc.ts";

/**
 * Every public SFC under `src`, listed independently of the lint walker.
 * Example SFCs are documentation fixtures and stay outside the gate.
 */
function expectedSfcFiles(): string[] {
  return globSync("src/**/*.vue")
    .map((filename) => filename.split(path.sep).join("/"))
    .filter((filename) => !filename.split("/").includes("examples"))
    .sort();
}

test("discovers every SFC with the opinionated Vize contract", async () => {
  const requests: Parameters<SfcLintFunction>[1][] = [];
  const lint: SfcLintFunction = (_source, options) => {
    requests.push(options);
    return { diagnostics: [] };
  };
  const results = await lintSfcFiles(lint);
  const expected = expectedSfcFiles();

  // Guard the oracle itself so an empty or misrooted listing cannot pass.
  assert.ok(expected.length > 300, `expected the package SFC inventory, got ${expected.length}`);
  for (const anchor of [
    "src/families/actions/button/button.vue",
    "src/families/menus/menu/menu-content.vue",
    "src/families/typography/text/text.vue",
  ]) {
    assert.ok(expected.includes(anchor), `${anchor} must be in the SFC inventory`);
  }
  assert.equal(
    expected.some((filename) => filename.includes("/examples/")),
    false,
    "example SFCs stay outside the gate",
  );

  assert.deepEqual(
    results.map((result) => result.filename.split(path.sep).join("/")),
    expected,
  );
  assert.deepEqual(
    requests,
    expected.map((filename) => ({
      filename: path.resolve(filename),
      preset: "opinionated" as const,
      typeAware: true as const,
      helpLevel: "short" as const,
    })),
  );
  assert.equal(formatSfcLintResults(results), "");
});

test("package lint:sfc stays wired to the shared Vize SFC gate", async () => {
  const manifest = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
    scripts?: Record<string, string>;
  };
  const lintScript = await readFile(path.resolve("scripts/lint-sfc.ts"), "utf8");

  assert.equal(
    manifest.scripts?.["lint:sfc"],
    "vp exec node scripts/lint-sfc.ts src && vp exec node scripts/check-renderers.ts src",
  );
  assert.match(lintScript, /import \{ lintPatinaSfc \} from "@vizejs\/native";/);
  assert.match(lintScript, /const lintCoreSfc: SfcLintFunction = \(source, options\) =>/);
  assert.match(lintScript, /corsaPath: workspaceCorsaPath/);
  assert.match(lintScript, /lintFiles\(lintCoreSfc, sourceRoots\)/);
  assert.match(
    lintScript,
    /runSfcLintCli\(lintCoreSfc, sourceRoots\.length > 0 \? sourceRoots : "src"\)/,
  );
});
