import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { BUILD_RECIPE, LEGACY_BUILD_RECIPE } from "../differential/build-receipt.ts";
import {
  COMPILER_ARGV,
  compilerArgv,
  loadCompilerManifest,
  runCompilerPack,
  validateCompilerReport,
} from "../differential/compiler.mjs";
import { sha256 } from "../differential/harness.mjs";

interface CompilerFixture {
  readonly id: string;
  readonly adapters: { readonly legacy: string };
  readonly requiredBuildRecipe: string | null;
  readonly argv: readonly string[];
  readonly inputs: {
    readonly dialectInput: {
      readonly selector: { readonly vueVersion: string; readonly templateSyntax: string };
    };
  };
  readonly dialectInputId?: string;
  readonly dialectInput: Buffer;
  readonly dialectInputSha256: string;
  readonly dialectTransport: string;
  readonly dialectConfig: Buffer;
  readonly dialectConfigPath: string;
  readonly dialectConfigSha256: string;
  readonly dialectCoverage: {
    readonly state: string;
    readonly evidence: readonly {
      readonly dialects: readonly string[];
      readonly selector: string;
    }[];
  };
}

interface CompilerReportRow {
  readonly id: string;
  legacy: {
    state: string;
    verdict?: string;
    readonly observation: unknown;
    readonly error: string;
  };
}

interface CompilerReport {
  readonly sourceRevision: string;
  readonly buildRecipe: string;
  readonly rows: CompilerReportRow[];
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath = path.join(root, "tests/_fixtures/differential/compiler/manifest.json");
const loaded = loadCompilerManifest(manifestPath) as {
  readonly manifest: { readonly baseRevision: string };
  readonly cases: readonly CompilerFixture[];
};
const vue1 = loaded.cases.filter((fixture) => fixture.dialectInputId?.includes("vue1-"));

void test("Vue 1 compiler rows bind every grammar input to the legacy recipe", () => {
  assert.deepEqual(
    vue1.map((fixture) => fixture.dialectInputId),
    ["dialect/vue1-attribute-interpolation", "dialect/vue1-list", "dialect/vue1-raw-html"],
  );
  for (const fixture of vue1) {
    assert.equal(fixture.adapters.legacy, "compiler-cli-ssr-template-v1-legacy");
    assert.equal(fixture.requiredBuildRecipe, LEGACY_BUILD_RECIPE);
    assert.deepEqual(fixture.argv, [
      ...COMPILER_ARGV.slice(0, -1),
      "--template-syntax",
      "standard",
      "--config",
      "vite.config.mjs",
    ]);
    assert.deepEqual(
      compilerArgv({ dialectInput: { selector: fixture.inputs.dialectInput.selector } }),
      fixture.argv,
    );
    assert.equal(sha256(fixture.dialectInput), fixture.dialectInputSha256);
    assert.equal(fixture.dialectTransport, "sfc-template-wrapper-v1");
    assert.equal(fixture.dialectConfigPath, "vite.config.mjs");
    assert.equal(sha256(fixture.dialectConfig), fixture.dialectConfigSha256);
    assert.equal(fixture.dialectConfig.toString(), 'export default { vue: { version: "1" } };\n');
    assert.equal(fixture.dialectCoverage.state, "verified");
    const evidence = fixture.dialectCoverage.evidence[0];
    assert(evidence);
    assert.deepEqual(evidence.dialects, ["vue1-template"]);
    assert.match(evidence.selector, /--features legacy$/);
  }
});

void test("ordinary source builds retain three failed Vue 1 rows without execution credit", () => {
  const report = runCompilerPack({
    manifestPath,
    binaryPath: path.join(root, "target/ci/missing-vize"),
    sourceRevision: loaded.manifest.baseRevision,
    repoRoot: root,
  }) as CompilerReport;
  const rows = report.rows.filter((row) => row.id.startsWith("compiler/dialect/vue1-"));
  assert.equal(report.buildRecipe, BUILD_RECIPE);
  assert.equal(rows.length, 3);
  for (const row of rows) {
    assert.equal(row.legacy.state, "failed");
    assert.equal(row.legacy.observation, null);
    assert.match(row.legacy.error, /requires source-build recipe .* --features legacy/);
  }

  const forged = structuredClone(report);
  const firstRow = rows[0];
  assert(firstRow);
  const forgedRow = forged.rows.find((row) => row.id === firstRow.id);
  assert(forgedRow);
  forgedRow.legacy.state = "completed";
  forgedRow.legacy.verdict = "matched-reference";
  assert.throws(
    () => validateCompilerReport(loaded, forged, { sourceRevision: report.sourceRevision }),
    /incompatible build recipe cannot get credit/,
  );
  assert.throws(
    () =>
      runCompilerPack({
        manifestPath,
        binaryPath: path.join(root, "target/ci/missing-vize"),
        sourceRevision: loaded.manifest.baseRevision,
        repoRoot: root,
        buildRecipe: "cargo build -p vize --all-features",
      }),
    /registered source-build recipe/,
  );
});
