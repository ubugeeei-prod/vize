import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { summarizeNativeAcceptance } from "./acceptance-rates.mjs";
import { LEGACY_BUILD_RECIPE } from "./build-receipt.ts";
import { loadCompilerManifest, runCompilerPack } from "./compiler.mjs";

interface CompilerRow {
  readonly id: string;
  readonly legacy: {
    readonly state: string;
    readonly verdict: string;
    readonly requiredBuildRecipe: string | null;
    readonly observation: null | {
      readonly facetError: unknown;
      readonly codeBase64: string;
    };
  };
}

interface CompilerReport {
  readonly buildReceipt: null | { readonly recipe: string };
  readonly summary: null | {
    readonly legacyMatches: number;
    readonly legacyFailures: number;
    readonly baselineDrift: number;
  };
  readonly rows: readonly CompilerRow[];
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("legacy source-built compiler executes every Vue 1 dialect row", (t) => {
  const revision = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(revision.status, 0, revision.stderr);
  const sourceRevision = revision.stdout.trim();
  const manifestPath = path.join(root, "tests/_fixtures/differential/compiler/manifest.json");
  const loaded = loadCompilerManifest(manifestPath) as { readonly cases: readonly unknown[] };
  const report = runCompilerPack({
    manifestPath,
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
    sourceRevision,
    repoRoot: root,
    buildRecipe: LEGACY_BUILD_RECIPE,
  }) as CompilerReport;
  const artifact = path.join(root, "target/differential/compiler.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`Legacy compiler observation: ${artifact}`);

  assert(report.buildReceipt, "legacy execution requires an exact build receipt");
  assert(report.summary, "legacy execution requires a complete compiler summary");
  assert.equal(report.buildReceipt.recipe, LEGACY_BUILD_RECIPE);
  assert.equal(
    report.summary.legacyMatches,
    loaded.cases.length,
    JSON.stringify(report.rows, null, 2),
  );
  assert.equal(report.summary.legacyFailures, 0);
  assert.equal(report.summary.baselineDrift, 0);

  const vue1 = report.rows.filter((row) => row.id.startsWith("compiler/dialect/vue1-"));
  assert.deepEqual(
    vue1.map((row) => row.id),
    [
      "compiler/dialect/vue1-attribute-interpolation",
      "compiler/dialect/vue1-list",
      "compiler/dialect/vue1-raw-html",
    ],
  );
  for (const row of vue1) {
    assert.equal(row.legacy.state, "completed");
    assert.equal(row.legacy.verdict, "matched-reference");
    assert.equal(row.legacy.requiredBuildRecipe, LEGACY_BUILD_RECIPE);
    assert(row.legacy.observation);
    assert.equal(row.legacy.observation.facetError, null);
  }
  const raw = vue1.find((row) => row.id.endsWith("/vue1-raw-html"));
  assert(raw);
  assert(raw.legacy.observation);
  const rawCode = Buffer.from(raw.legacy.observation.codeBase64, "base64").toString("utf8");
  assert.match(rawCode, /\$\{_ctx\.html\}/);
  assert.doesNotMatch(rawCode, /_ssrInterpolate/);

  const acceptance = summarizeNativeAcceptance(loaded, report, { sourceRevision });
  assert.deepEqual(acceptance.dialectCoverage.verified["vue1-template"], {
    planned: 3,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 3,
    legacyBacked: 0,
    unverified: 0,
  });
});
