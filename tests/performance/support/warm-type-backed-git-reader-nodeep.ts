import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { gitReaderIncomingPairs } from "./warm-type-backed-git-reader-incoming.ts";

type Pair = readonly [string, string | null, string | null];
const manifests: ReadonlyArray<readonly [string, string]> = [
  [
    "warm-type-backed-git-reader-nodeep.json",
    "bcecdd3cbc3fa8dbaafb7c7f4dc48f491d1a0a82e861ef034e69bdaa83c52adb",
  ],
  [
    "warm-type-backed-git-reader-nodeep-family-1.json",
    "02eba2d44a67fa4977e615b7f381cffe7fdd869e1b770ed9fc9cf1c461d8dba3",
  ],
  [
    "warm-type-backed-git-reader-nodeep-family-2.json",
    "323254f32bcf498c85b8afa365db0a4cdfea994682f5d1004affb24d52d92d17",
  ],
  [
    "warm-type-backed-git-reader-nodeep-family-3.json",
    "c7eb01f4537c4ca639cd28619a2cbae430fb8c9d1b4df2a0eb1fa5adf635979d",
  ],
];
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const loaded = manifests.map(([name, expected]) => {
  const body = fs.readFileSync(new URL(name, import.meta.url));
  assert.equal(sha(body), expected, `complete NoDeep reader manifest: ${name}`);
  return JSON.parse(body.toString("utf8")) as {
    source: string;
    historicalSource: string;
    files: Pair[];
    historicalImports: Array<[string, string]>;
  };
});
const prefix = "tests/performance/support/";
export const gitReaderNoDeepSource = "eafe5bed4f602e0787ff9f2f48ec4155d115ab7f";
export const gitReaderNoDeepGuard = `${prefix}warm-type-backed-git-reader-nodeep.ts`;
export const historicalIncomingSource = "3dda8f9d71fbd7e701f0805187daff6191865190";
assert.equal(loaded[0].historicalSource, historicalIncomingSource);
for (const manifest of loaded) assert.equal(manifest.source, gitReaderNoDeepSource);
export const gitReaderNoDeepPairs: readonly Pair[] = [
  ...loaded.flatMap(({ files }) => files),
  ...manifests.map(([name, after]) => [prefix + name, null, after] as const),
];
export const gitReaderNoDeepChanges = [
  ...loaded[0].files.filter(([, before, after]) => before !== after).map(([file]) => file),
  gitReaderNoDeepGuard,
  ...manifests.map(([name]) => prefix + name),
];
assert.equal(gitReaderNoDeepPairs.length, 102);
assert.equal(new Set(gitReaderNoDeepPairs.map(([file]) => file)).size, 102);
assert.equal(gitReaderNoDeepChanges.length, 14);

/** A complete current tuple; the original79 and both failed NoDeep families stay literal. */
export function qualifyGitReaderNoDeepHarness(
  changed: readonly string[],
  alreadyQualified: ReadonlySet<string>,
  digest: (side: "before" | "after", file: string) => string | null,
) {
  const finite = new Set(gitReaderNoDeepChanges);
  const family = new Set(loaded.slice(1).flatMap(({ files }) => files.map(([file]) => file)));
  if (
    new Set(changed).size !== changed.length ||
    gitReaderNoDeepChanges.some((file) => !changed.includes(file)) ||
    changed.some((file) => family.has(file)) ||
    changed.some((file) => !alreadyQualified.has(file) && !finite.has(file))
  )
    return null;
  for (const [file, before, after] of gitReaderNoDeepPairs)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-nodeep-raw-git-reader-harness",
    incomingSource: gitReaderNoDeepSource,
    completeBodyPairs: 102,
    namedFamilyBodies: 87,
    acceptanceCredit: "none",
    files: gitReaderNoDeepChanges,
  };
}

const rawGit = (root: string, args: string[], input?: string) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input });
export function prepareLiteralSource(root: string, source: string) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  const lookup = () =>
    rawGit(root, ["cat-file", "--batch-check=%(objectname) %(objecttype)"], `${source}\n`).toString(
      "utf8",
    );
  if (lookup() === `${source} missing\n`)
    rawGit(root, ["fetch", "--no-tags", "--depth=1", "origin", source]);
  assert.equal(lookup(), `${source} commit\n`);
}

/** Authenticate every whole historical import before evaluating the unchanged old module. */
export function runHistoricalIncomingLaw(root: string) {
  prepareLiteralSource(root, historicalIncomingSource);
  const expected = [
    ...gitReaderIncomingPairs.map(([file, , after]) => [file, after] as const),
    ...loaded[0].historicalImports,
  ];
  assert.equal(expected.length, 82);
  assert.equal(new Set(expected.map(([file]) => file)).size, 82);
  const bodies = expected.map(([file, digest]) => {
    const entry = rawGit(root, ["ls-tree", "-z", historicalIncomingSource, "--", file]);
    const text = entry.toString("utf8");
    assert.ok(Buffer.from(text).equals(entry));
    const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(text);
    assert.ok(match, `one regular historical blob required: ${file}`);
    assert.equal(match[2], file);
    const body = rawGit(root, ["cat-file", "blob", match[1]]);
    assert.equal(sha(body), digest, `whole historical body changed: ${file}`);
    return [file, body] as const;
  });
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reader-historical-"));
  try {
    for (const [file, body] of bodies) {
      const target = path.join(fixture, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, body, { flag: "wx", mode: 0o644 });
    }
    rawGit(fixture, ["init", "--quiet"]);
    const common = rawGit(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(fixture, ".git/objects/info/alternates"),
      path.resolve(root, common, "objects") + "\n",
    );
    rawGit(fixture, ["update-ref", "refs/heads/historical", historicalIncomingSource]);
    rawGit(fixture, ["symbolic-ref", "HEAD", "refs/heads/historical"]);
    const law = "tests/tooling/warm-type-backed-git-reader-incoming.test.ts";
    const env = { ...process.env };
    delete env.NODE_TEST_CONTEXT;
    const result = spawnSync(process.execPath, ["--test", "--test-reporter=tap", law], {
      cwd: fixture,
      env,
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
      timeout: 60_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    for (const line of ["# tests 4", "# pass 4", "# skipped 0"])
      assert.ok(result.stdout.split("\n").includes(line), result.stdout);
    return {
      source: historicalIncomingSource,
      moduleSHA256: sha(bodies.find(([file]) => file === law)![1]),
      completeBodyPairs: 79,
      additionalImportedBodies: 3,
      passed: 4,
      acceptanceCredit: "none",
    };
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
}
