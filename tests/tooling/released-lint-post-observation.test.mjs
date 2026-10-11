import assert from "node:assert/strict";
import { test } from "node:test";
import { apiRoot, authenticate } from "../../tools/support/release/typed_defaults/custody.mjs";
import { archiveName } from "../../tools/support/release/typed_defaults/distribution.mjs";
import {
  assertStableFix,
  assertStableOfficial,
} from "../../tools/support/release/typed_defaults/post_observation.mjs";
import { officialFixture } from "./support/released-typed-defaults-fixtures.mjs";

async function baseline() {
  const fixture = officialFixture();
  const release = fixture.rows.get(`${apiRoot}/releases/tags/${fixture.options.tag}`);
  release.url = `${apiRoot}/releases/${release.id}`;
  release.html_url = `https://github.com/ubugeeei-prod/vize/releases/tag/${fixture.options.tag}`;
  release.assets = [archiveName, `${archiveName}.sha256`].map((name, index) => ({
    id: 100 + index,
    name,
    size: index ? 103 : 4096,
    digest: `sha256:${String(index).repeat(64)}`,
    state: "uploaded",
    url: `${apiRoot}/releases/assets/${100 + index}`,
    browser_download_url: `https://github.com/ubugeeei-prod/vize/releases/download/${fixture.options.tag}/${name}`,
    content_type: "application/octet-stream",
    download_count: 0,
    updated_at: "2026-10-11T00:00:00Z",
    label: "",
  }));
  return authenticate(fixture.options, fixture.read);
}

await test("a public download preserves stable release identity while raw mutable metadata changes", async () => {
  const before = await baseline(),
    after = structuredClone(before);
  for (const asset of after.release.assets) {
    asset.download_count++;
    asset.updated_at = "2026-10-12T00:00:00Z";
    asset.label = "downloaded";
  }
  after.source.comments = 3;
  after.source.title = "updated discussion title";
  after.source.state = "open";
  after.fix.updated_at = "2026-10-12T00:00:00Z";
  after.fix.labels = [{ name: "released" }];
  after.ancestry.files = [{ filename: "metadata-only" }];
  assert.throws(() => assert.deepEqual(after.release.assets, before.release.assets));
  assert.throws(() => assert.deepEqual(after.fix, before.fix));
  assertStableOfficial(before, after);
  assertStableFix(before, after);
  assert.equal(before.release.assets[0].download_count, 0);
  assert.equal(after.release.assets[0].download_count, 1);
  after.release.assets.reverse();
  assertStableOfficial(before, after);
});

await test("post-observation rejects changed asset, release, source, run and actual fix authorities", async () => {
  const before = await baseline();
  const mutations = [
    [
      "asset id",
      (f) => {
        f.release.assets[0].id++;
      },
    ],
    [
      "asset name",
      (f) => {
        f.release.assets[0].name += ".foreign";
      },
    ],
    [
      "asset size",
      (f) => {
        f.release.assets[0].size++;
      },
    ],
    [
      "asset digest",
      (f) => {
        f.release.assets[0].digest = `sha256:${"f".repeat(64)}`;
      },
    ],
    [
      "asset state",
      (f) => {
        f.release.assets[0].state = "new";
      },
    ],
    [
      "asset API URL",
      (f) => {
        f.release.assets[0].url = "https://foreign.invalid/asset";
      },
    ],
    [
      "asset download URL",
      (f) => {
        f.release.assets[0].browser_download_url = "https://foreign.invalid/asset";
      },
    ],
    [
      "asset content type",
      (f) => {
        f.release.assets[0].content_type = "text/plain";
      },
    ],
    [
      "removed asset",
      (f) => {
        f.release.assets.pop();
      },
    ],
    [
      "duplicate asset",
      (f) => {
        f.release.assets.push(structuredClone(f.release.assets[0]));
      },
    ],
    [
      "release id",
      (f) => {
        f.release.id++;
      },
    ],
    [
      "release tag",
      (f) => {
        f.release.tag_name = "v0.999.0";
      },
    ],
    [
      "release publication",
      (f) => {
        f.release.draft = true;
      },
    ],
    [
      "release URL",
      (f) => {
        f.release.url = "https://foreign.invalid/release";
      },
    ],
    [
      "tag identity",
      (f) => {
        f.tagObject.sha = "f".repeat(40);
      },
    ],
    [
      "tag target",
      (f) => {
        f.tag.object.sha = "f".repeat(40);
      },
    ],
    [
      "source head",
      (f) => {
        f.source.head.sha = "f".repeat(40);
      },
    ],
    [
      "source repository",
      (f) => {
        f.source.head.repo.full_name = "foreign/vize";
      },
    ],
    [
      "run attempt",
      (f) => {
        f.releaseRun.run_attempt++;
      },
    ],
    [
      "run authority",
      (f) => {
        f.releaseRun.head_repository.full_name = "foreign/vize";
      },
    ],
    [
      "actual job",
      (f) => {
        f.jobs[0].conclusion = "failure";
      },
    ],
    [
      "H source tree",
      (f) => {
        f.commit.tree.sha = "f".repeat(40);
      },
    ],
    [
      "typed merge",
      (f) => {
        f.fix.merge_commit_sha = "f".repeat(40);
      },
    ],
    [
      "typed ancestry",
      (f) => {
        f.ancestry.merge_base_commit.sha = "f".repeat(40);
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const after = structuredClone(before);
    mutate(after);
    assert.throws(() => assertStableOfficial(before, after), undefined, name);
  }
  for (const [name, mutate] of [
    [
      "actual merge",
      (f) => {
        f.fix.merge_commit_sha = "f".repeat(40);
      },
    ],
    [
      "unmerged fix",
      (f) => {
        f.fix.merged = false;
      },
    ],
    [
      "foreign fix",
      (f) => {
        f.fix.head.repo.full_name = "foreign/vize";
      },
    ],
    [
      "base ancestry",
      (f) => {
        f.ancestry.base_commit.sha = "f".repeat(40);
      },
    ],
    [
      "merge ancestry",
      (f) => {
        f.ancestry.merge_base_commit.sha = "f".repeat(40);
      },
    ],
    [
      "diverged ancestry",
      (f) => {
        f.ancestry.status = "diverged";
      },
    ],
  ]) {
    const after = structuredClone(before);
    mutate(after);
    assert.throws(() => assertStableFix(before, after), undefined, name);
  }
});
