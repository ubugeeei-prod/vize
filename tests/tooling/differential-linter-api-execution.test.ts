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
import { expectedNativeReason } from "../differential/linter-native.ts";

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
      nativeUnsupported: 34,
      nativeFailures: 0,
      nativeHandled: 10,
      nativeEquivalent: 10,
      pairedComparisons: 10,
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
      .filter(
        (fixture: any) =>
          fixture.id.startsWith("linter/component-name/") ||
          ["linter/current-api/ref-string-untyped", "linter/current-api/ref-string-typed"].includes(
            fixture.id,
          ),
      )
      .map((fixture: any) => fixture.id)
      .sort(),
  );
  for (const row of report.rows) {
    assert.equal(row.native.attempts.length, 2);
    if (row.native.state === "unsupported") {
      const fixture = loaded.cases.find((fixture: any) => fixture.id === row.id);
      assert.deepEqual(row.native.reason, expectedNativeReason(fixture), row.id);
    }
  }
});
