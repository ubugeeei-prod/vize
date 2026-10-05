import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { satisfiesVersionRange } from "../../tools/support/compat/npm/smoke-release-semver.mjs";

const manifest = JSON.parse(
  fs.readFileSync(new URL("../../npm/cli/package.json", import.meta.url), "utf8"),
);
const corpus = JSON.parse(
  fs.readFileSync(
    new URL("../_fixtures/differential/package/vue-peer-prereleases/case.json", import.meta.url),
    "utf8",
  ),
);

test("the optional vize Vue peer admits the reported 3.6 prereleases", () => {
  assert.equal(manifest.name, "vize");
  assert.deepEqual(manifest.peerDependenciesMeta.vue, { optional: true });
  assert.deepEqual(
    corpus.versions.map((version: string) =>
      satisfiesVersionRange(version, manifest.peerDependencies.vue),
    ),
    corpus.accepted,
  );
});

test("the original peer range reproduces prerelease refusal with npm SemVer defaults", () => {
  assert.deepEqual(
    corpus.versions.map((version: string) =>
      satisfiesVersionRange(version, corpus.originalPeerRange),
    ),
    corpus.originalAccepted,
  );
});
