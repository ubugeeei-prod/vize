import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertReleaseCommitIsCurrentMain,
  assertReleaseMetadata,
  findReleaseBlockers,
  remoteTagCommit,
  workspaceVersionFromCargoToml,
} from "../../tools/github/release-preflight-core.mjs";

const sha = "a".repeat(40);
const cargoToml = `[workspace]\n\n[workspace.package]\nversion = "1.2.3"\nedition = "2024"\n`;
const packageManifests = [
  { path: "npm/cli/package.json", content: '{"name":"vize","version":"1.2.3"}' },
  { path: "editors/vscode/package.json", content: '{"name":"vize-vscode","version":"1.2.3"}' },
];

test("workspace release version comes only from the workspace package table", () => {
  assert.equal(workspaceVersionFromCargoToml(cargoToml), "1.2.3");
  assert.throws(
    () => workspaceVersionFromCargoToml('[package]\nversion = "9.9.9"\n'),
    /missing \[workspace\.package\]\.version/,
  );
});

test("release metadata binds the tag and every publishable package to one version", () => {
  assert.equal(assertReleaseMetadata({ tag: "v1.2.3", sha, cargoToml, packageManifests }), "1.2.3");
});

test("release metadata rejects a tag that diverges from the workspace version", () => {
  assert.throws(
    () => assertReleaseMetadata({ tag: "v1.2.4", sha, cargoToml, packageManifests }),
    /does not match workspace version/,
  );
});

test("release metadata rejects a divergent publishable package version", () => {
  assert.throws(
    () =>
      assertReleaseMetadata({
        tag: "v1.2.3",
        sha,
        cargoToml,
        packageManifests: [
          ...packageManifests,
          { path: "npm/wasm/package.json", content: '{"version":"1.2.2"}' },
        ],
      }),
    /npm\/wasm\/package\.json=1\.2\.2/,
  );
});

test("release metadata rejects a non-full commit SHA", () => {
  assert.throws(
    () => assertReleaseMetadata({ tag: "v1.2.3", sha: "main", cargoToml, packageManifests }),
    /full commit SHA/,
  );
});

test("release metadata identifies malformed package manifests", () => {
  assert.throws(
    () =>
      assertReleaseMetadata({
        tag: "v1.2.3",
        sha,
        cargoToml,
        packageManifests: [{ path: "npm/broken/package.json", content: "{" }],
      }),
    /Failed to parse release package manifest npm\/broken\/package\.json/,
  );
});

test("release metadata rejects private manifests in the publish inventory", () => {
  assert.throws(
    () =>
      assertReleaseMetadata({
        tag: "v1.2.3",
        sha,
        cargoToml,
        packageManifests: [
          { path: "npm/private/package.json", content: '{"private":true,"version":"1.2.3"}' },
        ],
      }),
    /npm\/private\/package\.json is private/,
  );
});

test("release commit must remain the exact current main tip", () => {
  assert.doesNotThrow(() => assertReleaseCommitIsCurrentMain(sha, sha));
  assert.throws(
    () => assertReleaseCommitIsCurrentMain(sha, "b".repeat(40)),
    /not the current origin\/main/,
  );
});

test("P0, P1, and fuzz reproducer issues block without treating PRs as issues", () => {
  const issues = [
    { number: 1, title: "release break", labels: [{ name: "priority:p0" }] },
    { number: 2, title: "correctness break", labels: ["PRIORITY:P1"] },
    { number: 3, title: "fix(fuzz): parser crash", labels: [] },
    { number: 4, title: "regular work", labels: [{ name: "area:ci" }] },
    { number: 5, title: "fix(fuzz): open pull request", labels: [], pull_request: {} },
  ];
  assert.deepEqual(
    findReleaseBlockers(issues).map((issue) => issue.number),
    [1, 2, 3],
  );
});

test("annotated remote tags resolve to their peeled commit", () => {
  const tag = "v1.2.3";
  const tagObject = "b".repeat(40);
  assert.equal(
    remoteTagCommit(`${tagObject}\trefs/tags/${tag}\n${sha}\trefs/tags/${tag}^{}\n`, tag),
    sha,
  );
  assert.equal(remoteTagCommit(`${sha}\trefs/tags/${tag}\n`, tag), sha);
  assert.equal(remoteTagCommit("", tag), undefined);
});
