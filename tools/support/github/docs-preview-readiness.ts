import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const expectedCounts = { tests: 8, pass: 8, fail: 0, cancelled: 0, skipped: 0, todo: 0 };

export function parseTapCounts(tap: string): Record<string, number> {
  return Object.fromEntries(
    [...tap.matchAll(/^# (tests|pass|fail|cancelled|skipped|todo) (\d+)$/gm)].map(
      ([, name, count]) => [name, Number(count)],
    ),
  );
}

export function writePreviewReadinessReceipt(tapPath: string, outputDirectory: string): void {
  const tap = readFileSync(tapPath, "utf8");
  const counts = parseTapCounts(tap);
  assert.deepEqual(counts, expectedCounts);
  const sourceSha = process.env.GITHUB_SHA;
  assert.match(sourceSha ?? "", /^[a-f0-9]{40}$/, "GITHUB_SHA must identify the exact source");
  writeFileSync(
    path.join(outputDirectory, "receipt.json"),
    `${JSON.stringify(
      {
        schema: "vize.docs-preview-resource-readiness",
        sourceSha,
        testFile: "tests/tooling/docs-preview-readiness-browser.test.ts",
        counts,
        tapSha256: createHash("sha256").update(tap).digest("hex"),
      },
      null,
      2,
    )}\n`,
  );
}

const scriptPath = fileURLToPath(import.meta.url);
if (process.argv[1] != null && path.resolve(process.argv[1]) === scriptPath) {
  const [tapPath, outputDirectory] = process.argv.slice(2);
  assert.ok(
    tapPath != null && outputDirectory != null,
    "usage: docs-preview-readiness.ts TAP OUTPUT",
  );
  writePreviewReadinessReceipt(tapPath, outputDirectory);
}
