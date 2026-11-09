import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

export const lspMarkerProductionPaths = [
  "crates/vize_maestro/src/server/state/project_contexts/root_discovery.rs",
  "crates/vize_maestro/src/server/state/project_contexts/root_discovery/",
] as const;

interface LspMarkerSourceSelection {
  head: string;
  parents: string[];
  requestedBase: string;
  expectedPrHead: string | null;
  selectedBase: string;
  eventBaseMatchesActualParent: boolean | null;
  policy: string;
}

export interface LspMarkerSourceReceipt extends LspMarkerSourceSelection {
  productionChanged: boolean;
  productionFiles: string[];
  productionDiff: {
    selectedBase: string;
    head: string;
    paths: readonly string[];
  };
}

const isMarkerProductionFile = (file: string) =>
  file === lspMarkerProductionPaths[0] || file.startsWith(lspMarkerProductionPaths[1]);

export function recordLspMarkerProductionDiff(
  selection: LspMarkerSourceSelection,
  changedFiles: string[],
): LspMarkerSourceReceipt {
  const productionFiles = [...new Set(changedFiles)].sort();
  assert.ok(
    productionFiles.every(isMarkerProductionFile),
    "production diff receipt may contain only marker discovery sources",
  );
  return {
    ...selection,
    productionChanged: productionFiles.length > 0,
    productionFiles,
    productionDiff: {
      selectedBase: selection.selectedBase,
      head: selection.head,
      paths: lspMarkerProductionPaths,
    },
  };
}

export function verifyLspMarkerProductionReceipt(
  receipt: LspMarkerSourceReceipt,
  expectedBase: string,
  expectedHead: string,
) {
  assert.equal(receipt.selectedBase, expectedBase, "wrong top-level selected baseline");
  assert.equal(receipt.head, expectedHead, "wrong top-level measured merge source");
  assert.equal(receipt.productionDiff.selectedBase, expectedBase, "wrong selected baseline");
  assert.equal(receipt.productionDiff.head, expectedHead, "wrong measured merge source");
  assert.deepEqual(receipt.productionDiff.paths, lspMarkerProductionPaths, "wrong diff paths");
  assert.deepEqual(receipt.productionFiles, [...new Set(receipt.productionFiles)].sort());
  assert.ok(
    receipt.productionFiles.every(isMarkerProductionFile),
    "production diff receipt contains an unrelated path",
  );
  assert.equal(
    receipt.productionChanged,
    receipt.productionFiles.length > 0,
    "production change flag disagrees with the immutable path diff",
  );
  return receipt;
}

export function selectLspMarkerSources(options: {
  head: string;
  parents: string[];
  requestedBase: string;
  expectedPrHead?: string;
}) {
  const valid = (sha: string) =>
    assert.match(sha, /^[0-9a-f]{40}$/u, "immutable 40-hex source required");
  valid(options.head);
  valid(options.requestedBase);
  options.parents.forEach(valid);
  let base = options.requestedBase;
  if (options.expectedPrHead) {
    valid(options.expectedPrHead);
    assert.equal(
      options.parents.length,
      2,
      "PR comparison must check out an actual two-parent merge",
    );
    assert.equal(
      options.parents[1],
      options.expectedPrHead,
      "the actual merge must contain the requested candidate head",
    );
    base = options.parents[0];
  }
  assert.notEqual(base, options.head, "baseline and measured candidate must differ");
  return {
    head: options.head,
    parents: options.parents,
    requestedBase: options.requestedBase,
    expectedPrHead: options.expectedPrHead ?? null,
    selectedBase: base,
    eventBaseMatchesActualParent: options.expectedPrHead ? base === options.requestedBase : null,
    policy: options.expectedPrHead
      ? "actual checked-out merge first parent"
      : "explicit dispatch baseline",
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const git = (args: string[]) => execFileSync("git", args, { encoding: "utf8" }).trim();
  const receiptPath = process.argv[2];
  assert.ok(receiptPath);
  if (process.argv[3] === "--record-production-diff") {
    const selection = JSON.parse(fs.readFileSync(receiptPath, "utf8")) as LspMarkerSourceSelection;
    assert.equal(selection.head, git(["rev-parse", "HEAD"]), "receipt is not this checkout");
    const changed = git([
      "diff",
      "--name-only",
      selection.selectedBase,
      selection.head,
      "--",
      ...lspMarkerProductionPaths,
    ]);
    const receipt = recordLspMarkerProductionDiff(selection, changed ? changed.split("\n") : []);
    fs.writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + "\n");
    console.log(JSON.stringify(receipt));
    process.exit(0);
  }
  // Raw commit metadata retains parents even when checkout is shallow.
  const header = git(["cat-file", "commit", "HEAD"]).split("\n\n", 1)[0];
  const parents = header
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice(7));
  const selected = selectLspMarkerSources({
    head: git(["rev-parse", "HEAD"]),
    parents,
    requestedBase: process.env.REQUESTED_BASE_SHA ?? "",
    expectedPrHead: process.env.EXPECTED_PR_HEAD_SHA || undefined,
  });
  assert.ok(process.env.GITHUB_ENV);
  fs.writeFileSync(receiptPath, JSON.stringify(selected, null, 2) + "\n");
  fs.appendFileSync(process.env.GITHUB_ENV, `BASE_SHA=${selected.selectedBase}\n`);
  console.log(JSON.stringify(selected));
}
