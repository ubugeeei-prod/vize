import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  emptyVueStarBodyPairs,
  emptyVueStarFiles,
  emptyVueStarMetadataPairs,
  qualifyEmptyVueStarHost,
} from "../../performance/support/warm-type-backed-empty-vue-star-host.ts";
import { ensureHostedLiteralSource } from "../../performance/support/warm-type-backed-git-reader-literal-source.ts";

type Read = (file: string) => Buffer | null;
const root = path.resolve(import.meta.dirname, "../../..");
const host = "tests/performance/support/warm-type-backed-host-qualifications.ts";
const originalSHA = "b19f6783e50429d2779efeb7d2d21c64dcd98a67810fc4f6a1ccc698febae337";
const currentSHA = "bec16c021cf11fce5e41180b42eb7df3ec5a38c6c14bdba64d0aba14991e78fe";
const beforeSource = "a93e78147b0990c813e965a7868b32dda341eb77";
const helperPairs = [
  [
    "tests/performance/support/warm-type-backed-empty-vue-star-host.ts",
    "4668a48d816279308c341ff3a7c839dc39ea129a2901b0484b65f4905e00386b",
  ],
  [
    "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts",
    "3dc6a16c6e9869302858df257d21e2e2b03acdc13606460bd7849d9d9ce28fc2",
  ],
] as const;
const clauses = [
  'import { qualifyEmptyVueStarHost } from "./warm-type-backed-empty-vue-star-host.ts";\n\n',
  '  const emptyVueStarHost =\n    enabled &&\n    digest("before", "tests/performance/support/warm-type-backed-empty-vue-star-host.ts") ===\n      null &&\n    digest("after", "tests/performance/support/warm-type-backed-empty-vue-star-host.ts") ===\n      "4668a48d816279308c341ff3a7c839dc39ea129a2901b0484b65f4905e00386b" &&\n    digest("before", "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts") === null &&\n    digest("after", "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts") ===\n      "3dc6a16c6e9869302858df257d21e2e2b03acdc13606460bd7849d9d9ce28fc2"\n      ? qualifyEmptyVueStarHost(production, alreadyQualified, digest)\n      : null;\n',
  "    ...(emptyVueStarHost ? { emptyVueStarHost } : {}),\n",
];
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const blob = (body: Buffer) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const git = (args: string[], input?: string) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    input,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    maxBuffer: 64 * 1024 * 1024,
  });
function physical(file: string) {
  const target = path.join(root, file);
  const stat = fs.lstatSync(target);
  assert(
    stat.isFile() && (stat.mode & 0o777) === 0o644,
    "regular complete current source: " + file,
  );
  return fs.readFileSync(target);
}
let original: Map<string, string | null> | undefined;
function beforeBodies(files: string[]) {
  if (original) return original;
  ensureHostedLiteralSource(root, beforeSource);
  const commit = git(["cat-file", "commit", beforeSource]);
  assert.equal(
    createHash("sha1").update(`commit ${commit.length}\0`).update(commit).digest("hex"),
    beforeSource,
  );
  const entries = git(["ls-tree", "-z", beforeSource, "--", ...files]);
  assert.deepEqual(Buffer.from(entries.toString()), entries);
  const rows = entries
    .toString()
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const match = /^100644 blob ([a-f0-9]{40})\t(.+)$/u.exec(entry);
      assert(match && files.includes(match[2]), "one actual regular predecessor source");
      return { blob: match[1], file: match[2] };
    });
  assert.equal(new Set(rows.map((row) => row.file)).size, rows.length);
  const raw = git(["cat-file", "--batch"], rows.map((row) => row.blob).join("\n") + "\n");
  const result = new Map(files.map((file) => [file, null] as [string, string | null]));
  let offset = 0;
  for (const row of rows) {
    const end = raw.indexOf("\n", offset);
    const header = /^([a-f0-9]{40}) blob ([0-9]+)$/u.exec(raw.subarray(offset, end).toString());
    assert(header && header[1] === row.blob);
    const body = raw.subarray(end + 1, end + 1 + Number(header[2]));
    assert.equal(body.length, Number(header[2]));
    assert.equal(blob(body), row.blob);
    offset = end + 1 + body.length;
    assert.equal(raw[offset++], 10);
    result.set(row.file, sha(body));
  }
  assert.equal(offset, raw.length);
  original = result;
  return result;
}

/** Retain the original historical hook only after complete actual star-source custody. */
export function retainedHostBeforeEmptyVueStars(read: Read): Buffer {
  const actual = physical(host);
  const requested = read(host);
  assert(requested, "whole supplied hook required");
  const actualSHA = sha(actual);
  assert.equal(actualSHA, currentSHA, "unknown physical star hook refused");
  assert(
    requested.equals(actual) || sha(requested) === originalSHA,
    "foreign or hybrid supplied hook refused",
  );
  assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  const pairs = [...emptyVueStarBodyPairs, ...emptyVueStarMetadataPairs];
  const files = [...pairs.map(([file]) => file), ...helperPairs.map(([file]) => file)];
  assert.equal(new Set(files).size, files.length);
  const before = beforeBodies(files);
  const after = new Map<string, string>();
  for (const [file, , expected] of pairs) {
    const body = physical(file);
    assert.equal(sha(body), expected, "complete actual star source: " + file);
    after.set(file, sha(body));
  }
  for (const [file, expected] of helperPairs) {
    assert.equal(before.get(file), null, "genuine predecessor helper absence: " + file);
    const body = physical(file);
    assert.equal(sha(body), expected, "whole qualified star helper: " + file);
    after.set(file, sha(body));
  }
  assert(
    qualifyEmptyVueStarHost([...emptyVueStarFiles], new Set(), (side, file) =>
      side === "before" ? (before.get(file) ?? null) : (after.get(file) ?? null),
    ),
  );
  let inverse = actual.toString("utf8");
  assert.deepEqual(Buffer.from(inverse), actual);
  for (const clause of clauses) {
    assert.equal(inverse.split(clause).length, 2);
    inverse = inverse.replace(clause, "");
  }
  const restored = Buffer.from(inverse);
  assert.equal(sha(restored), originalSHA, "complete actual main hook inverse");
  return restored;
}
