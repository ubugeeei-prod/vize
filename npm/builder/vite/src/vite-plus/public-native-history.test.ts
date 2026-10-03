import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { runPublicNativeFormatter } from "../../../../../tests/differential/formatter-native.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../../..");
void test(
  "public NAPI sorting retains whole independent results/errors and actual build custody",
  {
    skip: process.env.GITHUB_ACTIONS !== "true" || process.platform !== "linux",
  },
  () => {
    const report = runPublicNativeFormatter(root);
    assert.deepEqual(report.summary, {
      plannedCases: 9,
      legacyByteMatches: 8,
      legacyErrorMatches: 1,
      legacyFailures: 0,
      nativeUnsupported: 9,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    });
    assert.equal(
      report.rows.reduce((count, row) => count + row.passes.length, 0),
      25,
    );
  },
);
