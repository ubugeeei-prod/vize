import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

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
  assert.ok(process.env.GITHUB_ENV && process.argv[2]);
  fs.writeFileSync(process.argv[2], JSON.stringify(selected, null, 2) + "\n");
  fs.appendFileSync(process.env.GITHUB_ENV, `BASE_SHA=${selected.selectedBase}\n`);
  console.log(JSON.stringify(selected));
}
