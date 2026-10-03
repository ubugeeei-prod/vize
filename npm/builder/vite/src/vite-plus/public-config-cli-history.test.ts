import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { runPublicViteFormatter } from "../../../../../tests/differential/formatter-vite.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../../..");
void test(
  "public Vite config retains actual npm CLI JSON, addon, streams and whole files",
  { skip: process.env.GITHUB_ACTIONS !== "true" || process.platform !== "linux" },
  async () => {
    const report = await runPublicViteFormatter(root);
    assert.equal(report.rows.length, 9);
    assert.equal(report.rows.filter(({ state }) => state === "matched-reference").length, 9);
    assert.equal(
      report.rows.reduce((count, row) => count + row.passes.length, 0),
      27,
    );
    assert.equal(report.nativeHandled, 0);
    assert.equal(report.nativeEquivalent, 0);
    assert.equal(report.pairedComparisons, 0);
  },
);
