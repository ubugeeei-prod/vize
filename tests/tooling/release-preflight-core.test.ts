import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertReleaseCommitIsOnMainFirstParent,
  assertReleaseMetadata,
  assertReleaseVersionStillOwnsMain,
  findReleaseBlockers,
  remoteTagCommit,
  workspaceVersionFromCargoToml,
} from "../../legacy-tools/github/release-preflight-core.mjs";

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

test("release commit may remain on main's first-parent history when main advances", () => {
  assert.doesNotThrow(() => assertReleaseCommitIsOnMainFirstParent(sha, sha, true));
  assert.doesNotThrow(() => assertReleaseCommitIsOnMainFirstParent(sha, "b".repeat(40), true));
});

test("release commit rejects ancestry that exists only through a merge's second parent", () => {
  assert.throws(
    () => assertReleaseCommitIsOnMainFirstParent("c".repeat(40), "b".repeat(40), false),
    new Error(
      `Release commit ${"c".repeat(40)} is not on the first-parent history of current origin/main ${"b".repeat(40)}`,
    ),
  );
});

// The merge automation lands PRs throughout the preflight's 30-40 minute gate
// wait, and none of them changes what this release publishes: the artifacts are
// built from the immutable tagged tree and every required gate is evaluated at
// the release SHA. A second release commit is different -- it moves the version
// line on, and finishing the older release then ships a lower version after a
// higher one.
test("ordinary drift on main keeps the release, a newer release takes it away", () => {
  const mainSha = "b".repeat(40);
  assert.doesNotThrow(() =>
    assertReleaseVersionStillOwnsMain({
      tag: "v1.2.3",
      sha,
      mainSha,
      releaseVersion: "1.2.3",
      mainVersion: "1.2.3",
    }),
  );
  assert.throws(
    () =>
      assertReleaseVersionStillOwnsMain({
        tag: "v1.2.3",
        sha,
        mainSha,
        releaseVersion: "1.2.3",
        mainVersion: "1.3.0",
      }),
    new Error(
      `Release v1.2.3 (${sha}) is superseded: origin/main ${mainSha} is at workspace version 1.3.0, not 1.2.3. Publishing it now would ship an older version after a newer one; cut the next release instead.`,
    ),
  );
});

const blockerIssues = [
  { number: 1, title: "release break", labels: [{ name: "priority:p0" }] },
  { number: 2, title: "correctness break", labels: ["PRIORITY:P1"] },
  { number: 3, title: "fix(fuzz): parser crash", labels: [] },
  { number: 4, title: "regular work", labels: [{ name: "area:ci" }] },
  { number: 5, title: "fix(fuzz): open pull request", labels: [], pull_request: {} },
];

test("P0, P1, and fuzz reproducer issues block without treating PRs as issues", () => {
  assert.deepEqual(
    findReleaseBlockers(blockerIssues).map((issue) => issue.number),
    [1, 2, 3],
  );
});

test("readiness labels block v1 releases", () => {
  for (const tag of ["v1.0.0", "v1.4.2", "v2.0.0", "v1.0.0-alpha.1"]) {
    assert.deepEqual(
      findReleaseBlockers(blockerIssues, tag).map((issue) => issue.number),
      [1, 2, 3],
      tag,
    );
  }
});

test("readiness labels do not block pre-1.0 releases, but fuzz reproducers still do", () => {
  // `priority:p0`/`priority:p1` are defined as "v1 alpha production
  // readiness" markers, so they must not gate `0.x`; a `fix(fuzz):`
  // reproducer is a live defect and blocks every release.
  for (const tag of ["v0.302.0", "v0.1.0", "v0.302.0-rc.1"]) {
    assert.deepEqual(
      findReleaseBlockers(blockerIssues, tag).map((issue) => issue.number),
      [3],
      tag,
    );
  }
});

test("the current open-issue shape blocks v1 but not the pre-1.0 line", () => {
  // Shape mirrors the repository as of this change: campaign umbrellas and
  // readiness-labelled defects, no open `fix(fuzz):` reproducer.
  const openIssues = [
    { number: 2971, title: "test(editor): production gates", labels: [{ name: "priority:p0" }] },
    { number: 3222, title: "roadmap(canon): vue-tsc parity", labels: [{ name: "priority:p0" }] },
    { number: 3131, title: "roadmap: universal atelier", labels: [{ name: "priority:p1" }] },
    { number: 3334, title: "fix(glyph): formatter re-indents", labels: [{ name: "area:glyph" }] },
  ];
  assert.deepEqual(findReleaseBlockers(openIssues, "v0.302.0"), []);
  assert.deepEqual(
    findReleaseBlockers(openIssues, "v1.0.0").map((issue) => issue.number),
    [2971, 3222, 3131],
  );
  // An open fuzz reproducer would still stop the same pre-1.0 release.
  assert.deepEqual(
    findReleaseBlockers(
      [
        ...openIssues,
        { number: 9999, title: "fix(fuzz): css_parse found a reproducer", labels: [] },
      ],
      "v0.302.0",
    ).map((issue) => issue.number),
    [9999],
  );
});

test("a malformed release tag is rejected rather than silently unblocking", () => {
  assert.throws(() => findReleaseBlockers(blockerIssues, "not-a-version"), /Release tag must look/);
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
