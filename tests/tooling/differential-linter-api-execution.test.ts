import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { buildProductObserver } from "../differential/observer-build.ts";
import { LINTER_OBSERVER, runLinterApiPack } from "../differential/linter-api.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("shared linter corpus retains all 36 complete actual public observations from source-built Rust", (t) => {
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
      plannedCases: 36,
      legacyMatches: 36,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: 36,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
    JSON.stringify(report.rows, null, 2),
  );
});
