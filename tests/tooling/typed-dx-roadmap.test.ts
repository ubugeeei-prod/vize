import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const roadmapPath = path.join(repoRoot, "docs/release/typed-dx-roadmap.md");
const oracleMatrixPath = path.join(repoRoot, "docs/release/typed-editor-oracle-matrix.md");

test("typed DX roadmap keeps every P0 child issue in the execution matrix", () => {
  const roadmap = fs.readFileSync(roadmapPath, "utf8");
  const roadmapIssues = roadmapP0IssueReferences(roadmap);

  assert.match(roadmap, /^# Typed DX production roadmap$/m);
  assert.match(roadmap, /This is the P0 execution lane for #3957 and #4585\./);
  assert.match(roadmap, /`docs\/release\/typed-editor-oracle-matrix\.md`/);
  assert.equal(roadmapIssues.length, 7, "expected seven typed DX P0 roadmap rows");

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
  const roadmap = fs.readFileSync(roadmapPath, "utf8");
  const matrix = fs.readFileSync(oracleMatrixPath, "utf8");
  const requiredIssues = roadmapP0IssueReferences(roadmap);
  const rows = markdownTableRows(matrix);
  const header = rows.find((row) => row.includes("Status") && row.includes("Follow-up"));
  assert.ok(header, "missing external behavior ledger header");

  const statusIndex = header.indexOf("Status");
  const followUpIndex = header.indexOf("Follow-up");
  assert.notEqual(statusIndex, -1, "missing Status column");
  assert.notEqual(followUpIndex, -1, "missing Follow-up column");

  const dataRows = rows.filter((row) => row.length === header.length && row !== header);

  for (const issue of requiredIssues) {
    const issueRows = dataRows.filter((row) =>
      issueReferences(row[followUpIndex] ?? "").includes(issue),
    );
    assert.ok(issueRows.length > 0, `missing external behavior ledger row for ${issue}`);
    assert.ok(
      issueRows.some((row) => row[statusIndex] === "Covered"),
      `${issue} must have at least one covered external behavior ledger row`,
    );
  }
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function roadmapP0IssueReferences(roadmap: string): string[] {
  return markdownTableRows(roadmap)
    .map((row) => row[0] ?? "")
    .filter((cell) => /^#\d+$/.test(cell));
}

function markdownTableRows(markdown: string): string[][] {
  return markdown
    .split("\n")
    .map(markdownTableCells)
    .filter((row) => row.length > 0 && !row.every((cell) => /^-+$/.test(cell)));
}

function markdownTableCells(line: string): string[] {
  const trimmed = line.trim();
  if (!trimmed.startsWith("|") || !trimmed.endsWith("|")) {
    return [];
  }
  return trimmed
    .slice(1, -1)
    .split("|")
    .map((cell) => cell.trim());
}

function issueReferences(cell: string): string[] {
  return cell
    .split(",")
    .map((item) => item.trim())
    .filter((item) => /^#\d+$/.test(item));
}
