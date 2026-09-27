import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { buildFormatterObserver } from "../differential/formatter-api-build.mjs";
import { runFormatterApiPack } from "../differential/formatter-api.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("shared formatter API corpus observes all six exact outputs from a source-built artifact", (t) => {
  const evidenceDir = path.resolve(
    root,
    process.env.VIZE_FORMATTER_API_EVIDENCE_DIR ?? "target/differential/formatter-api",
  );
  const built = buildFormatterObserver({
    repoRoot: root,
    targetDir: path.resolve(root, process.env.CARGO_TARGET_DIR ?? "target"),
    evidenceDir,
    profile: process.env.CI ? "ci" : "dev",
    offline: !process.env.CI,
  });
  const report = runFormatterApiPack({
    manifestPath: path.join(
      root,
      "tests/_fixtures/differential/formatter-history/script-manifest.json",
    ),
    repoRoot: root,
    ...built,
  });
  const artifact = path.join(evidenceDir, "report.json");
  fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`Raw public API observations and build receipt: ${evidenceDir}`);
  assert.deepEqual(
    report.summary,
    {
      plannedCases: 6,
      legacyByteMatches: 4,
      legacyInternalObservations: 2,
      legacyFailures: 0,
      nativeUnsupported: 6,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
    JSON.stringify(report.rows, null, 2),
  );
});
