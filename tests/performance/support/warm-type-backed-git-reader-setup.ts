import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import {
  gitReaderNoDeepChanges,
  gitReaderNoDeepGuard,
  gitReaderNoDeepPairs,
  gitReaderNoDeepSource,
  prepareLiteralSource,
} from "./warm-type-backed-git-reader-nodeep.ts";

type Pair = readonly [string, string | null, string | null];
type Delta = readonly [string, string | null, string | null, string | null];
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const name = "warm-type-backed-git-reader-setup.json";
const body = fs.readFileSync(new URL(name, import.meta.url));
const manifestSHA = "2a098a6c73610cf6b3ba31d8f37e22da0e57fc6dd748260b3ad7adbdcef93933";
assert.equal(sha(body), manifestSHA, "complete finite reader setup successor");
const manifest = JSON.parse(body.toString("utf8")) as {
  originalSource: string;
  previousSource: string;
  previousGuardSHA256: string;
  deltas: Delta[];
};
export const gitReaderSetupSource = gitReaderNoDeepSource;
export const gitReaderSetupPrevious = "6354e9b92ef842816f8f8be0d104b067f4175e96";
export const gitReaderSetupGuard = "tests/performance/support/warm-type-backed-git-reader-setup.ts";
assert.equal(manifest.originalSource, gitReaderSetupSource);
assert.equal(manifest.previousSource, gitReaderSetupPrevious);
assert.equal(
  sha(fs.readFileSync(new URL("warm-type-backed-git-reader-nodeep.ts", import.meta.url))),
  manifest.previousGuardSHA256,
  "complete previous reader guard remains literal",
);
assert.equal(
  manifest.previousGuardSHA256,
  "ad25378cc7272fb2fde533d6a6796a9462dec3d3e157217abef13a5a69cf6a96",
);
assert.equal(new Set(manifest.deltas.map(([file]) => file)).size, manifest.deltas.length);
const previous = new Map(
  gitReaderNoDeepPairs.map(([file, before, after]) => [file, { before, after }]),
);
for (const [file, before, oldAfter] of manifest.deltas) {
  const original = previous.get(file);
  if (original) assert.deepEqual([before, oldAfter], [original.before, original.after]);
}

const rawGit = (root: string, args: string[]) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root });

/** Prove literal ancestry despite shallow markers without borrowing replacements. */
export function rawFirstParentProof(cwd: string, ancestor: string, head: string) {
  for (const source of [ancestor, head]) assert.match(source, /^[a-f0-9]{40}$/u);
  const visited: string[] = [];
  for (let current = head; visited.length < 64;) {
    prepareLiteralSource(cwd, current);
    const body = rawGit(cwd, ["cat-file", "commit", current]);
    assert.equal(
      createHash("sha1").update(`commit ${body.length}\0`).update(body).digest("hex"),
      current,
      "complete raw commit identity",
    );
    const boundary = body.indexOf("\n\n");
    assert.ok(boundary > 0, "complete commit headers");
    const bytes = body.subarray(0, boundary);
    const headers = bytes.toString("utf8");
    assert.ok(Buffer.from(headers).equals(bytes));
    const lines = headers.split("\n");
    assert.match(lines[0], /^tree [a-f0-9]{40}$/u);
    const parents = lines.filter((line) => line.startsWith("parent "));
    for (const parent of parents) assert.match(parent, /^parent [a-f0-9]{40}$/u);
    visited.push(current);
    if (current === ancestor) return visited;
    assert.ok(parents.length > 0, "disconnected raw first-parent ancestry");
    current = parents[0].slice(7);
  }
  assert.fail("raw first-parent ancestry exceeds 64 commits");
}
const deltas = new Map(
  manifest.deltas.map(([file, before, , after]) => [file, [file, before, after] as Pair]),
);
export const gitReaderSetupPairs: readonly Pair[] = [
  ...gitReaderNoDeepPairs.map((row) => deltas.get(row[0]) ?? row),
  ...manifest.deltas
    .filter(([file]) => !previous.has(file))
    .map(([file, before, , after]) => [file, before, after] as Pair),
  ["tests/performance/support/" + name, null, manifestSHA],
];
export const gitReaderSetupPreviousDeltas = manifest.deltas;
export const gitReaderSetupChanges = [
  ...new Set([
    ...gitReaderNoDeepChanges,
    ...manifest.deltas.filter(([, before, , after]) => before !== after).map(([file]) => file),
    gitReaderSetupGuard,
    "tests/performance/support/" + name,
  ]),
];
assert.equal(new Set(gitReaderSetupPairs.map(([file]) => file)).size, gitReaderSetupPairs.length);
assert.ok(gitReaderSetupPairs.some(([file]) => file === gitReaderNoDeepGuard));

/** A separate source-only successor; no previous tuple receives current-source credit. */
export function qualifyGitReaderSetupHarness(
  changed: readonly string[],
  alreadyQualified: ReadonlySet<string>,
  digest: (side: "before" | "after", file: string) => string | null,
) {
  const finite = new Set(gitReaderSetupChanges);
  const preserved = new Set(
    gitReaderSetupPairs.map(([file]) => file).filter((file) => !finite.has(file)),
  );
  if (
    new Set(changed).size !== changed.length ||
    gitReaderSetupChanges.some((file) => !changed.includes(file)) ||
    changed.some((file) => preserved.has(file)) ||
    changed.some((file) => !finite.has(file) && !alreadyQualified.has(file))
  )
    return null;
  for (const [file, before, after] of gitReaderSetupPairs)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-raw-git-reader-setup-successor",
    incomingSource: gitReaderSetupSource,
    previousSource: gitReaderSetupPrevious,
    completeBodyPairs: gitReaderSetupPairs.length,
    preservedPreviousPairs: gitReaderNoDeepPairs.length,
    namedFamilyBodies: 87,
    acceptanceCredit: "none",
    files: gitReaderSetupChanges,
  };
}
