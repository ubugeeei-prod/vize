import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const roadmapPath = path.join(repoRoot, "docs/release/typed-dx-roadmap.md");
const oracleMatrixPath = path.join(repoRoot, "docs/release/typed-editor-oracle-matrix.md");

const requiredIssues = [4586, 4587, 4588, 4589, 4590, 4591, 4592] as const;

test("typed DX roadmap keeps every P0 child issue in the execution matrix", () => {
  const roadmap = fs.readFileSync(roadmapPath, "utf8");

  assert.match(roadmap, /^# Typed DX production roadmap$/m);
  assert.match(roadmap, /This is the P0 execution lane for #3957 and #4585\./);
  assert.match(roadmap, /`docs\/release\/typed-editor-oracle-matrix\.md`/);

  for (const issue of requiredIssues) {
    assert.match(roadmap, new RegExp(`\\\\| #${issue} \\\\|`), `missing #${issue}`);
  }

  for (const phrase of [
    "authored template ranges",
    "Authored script TypeScript diagnostics",
    "Hover is non-empty and checker-backed",
    "Ref<unknown>",
    "TS7026",
    "__vizeComponentMarker",
    "Template hover and navigation",
  ]) {
    assert.match(roadmap, new RegExp(escapeRegExp(phrase)), `missing phrase: ${phrase}`);
  }
});

test("typed DX roadmap forbids umbrella implementation delivery", () => {
  const roadmap = fs.readFileSync(roadmapPath, "utf8");

  assert.match(roadmap, /One P0 invariant per PR\./);
  assert.match(roadmap, /No umbrella implementation PRs\./);
  assert.match(roadmap, /external behavior oracle/);
  assert.match(roadmap, /body files and raw-checked/);
});

test("typed DX roadmap child issues have covered external behavior ledger rows", () => {
  const matrix = fs.readFileSync(oracleMatrixPath, "utf8");
  const rows = matrix.split("\n").filter((line) => line.startsWith("| "));

  for (const issue of requiredIssues) {
    const issueRows = rows.filter((line) => line.includes(`#${issue}`));
    assert.ok(issueRows.length > 0, `missing external behavior ledger row for #${issue}`);
    assert.ok(
      issueRows.some((line) => /\|\s*Covered\s*\|/.test(line)),
      `#${issue} must have at least one covered external behavior ledger row`,
    );
  }
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
