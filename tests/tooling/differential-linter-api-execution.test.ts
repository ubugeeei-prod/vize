import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { buildProductObserver } from "../differential/observer-build.ts";
import {
  loadLinterManifest,
  LINTER_OBSERVER,
  runLinterApiPack,
} from "../differential/linter-api.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("shared linter corpus retains all 44 complete actual public observations from source-built Rust", (t) => {
  const evidenceDir = path.resolve(
    root,
    process.env.VIZE_LINTER_API_EVIDENCE_DIR ?? "target/differential/linter-api",
  );
  const built = buildProductObserver({
    spec: LINTER_OBSERVER,
    repoRoot: root,
    targetDir: path.resolve(root, process.env.CARGO_TARGET_DIR ?? "target"),
    evidenceDir,
    profile: process.env.CI ? "ci" : "dev",
    offline: !process.env.CI,
  });
  const report = runLinterApiPack({
    manifestPath: path.join(root, "tests/_fixtures/differential/linter/manifest.json"),
    repoRoot: root,
    ...built,
  });
  fs.writeFileSync(path.join(evidenceDir, "report.json"), `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(
    `Complete raw public observations, Cargo artifact and build receipt: ${evidenceDir}`,
  );
  assert.deepEqual(
    report.summary,
    {
      plannedCases: 44,
      legacyMatches: 44,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: 36,
      nativeFailures: 0,
      nativeHandled: 8,
      nativeEquivalent: 8,
      pairedComparisons: 8,
    },
    JSON.stringify(report.rows, null, 2),
  );
  const loaded = loadLinterManifest(
    path.join(root, "tests/_fixtures/differential/linter/manifest.json"),
    root,
  );
  const handled = report.rows.filter((row: any) => row.native.state === "completed");
  assert.deepEqual(
    handled.map((row: any) => row.id).sort(),
    loaded.cases
      .filter((fixture: any) => fixture.id.startsWith("linter/component-name/"))
      .map((fixture: any) => fixture.id)
      .sort(),
  );
  for (const row of report.rows) {
    assert.equal(row.native.attempts.length, 2);
    if (row.native.state === "unsupported") {
      const fixture = loaded.cases.find((fixture: any) => fixture.id === row.id);
      const input = JSON.parse(fixture.input.toString());
      const kind =
        fixture.argv[0] === "--report"
          ? "ApiUnavailable"
          : input.entry !== "template"
            ? "EntryUnavailable"
            : input.vue_version === "2"
              ? "UnsupportedVueVersion"
              : input.vapor === true
                ? "UnsupportedVaporMode"
                : "UnprovidedRule";
      assert.equal(row.native.reason.kind, kind, row.id);
      if (kind === "UnprovidedRule")
        assert.equal(
          row.native.reason.detail,
          `UnprovidedRule { rule: "${input.rule ?? "vapor/prefer-static-class"}" }`,
        );
    }
  }
});
