import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { derivePublicationPlan } from "../../tools/support/release/public_acceptance/plan.ts";
import {
  jsrAuthorityPaths,
  jsrJobNames,
} from "../../tools/support/release/public_acceptance/jsr.ts";
import { verifyPublication } from "../../tools/support/release/public_acceptance/registry.ts";
import { jsrPublicFixture } from "./support/jsr-public-acceptance-fixture.ts";
import {
  gh,
  publicFixture,
  sha256,
  sourceFixture,
  version,
} from "./support/release-public-acceptance-fixtures.ts";

test("pre-JSR H preserves the legacy plan and receipt shape without any JSR requests", async (t) => {
  const fixture = publicFixture(t);
  assert.equal(Object.hasOwn(fixture.plan, "jsr"), false);
  assert.deepEqual(Object.keys(fixture.plan), [
    "head",
    "parentCut",
    "version",
    "npm",
    "crates",
    "editor",
    "githubAssets",
    "authority",
  ]);
  const result = await verifyPublication(fixture.plan, fixture.options);
  assert.equal(Object.hasOwn(result, "jsr"), false);
  assert.deepEqual(Object.keys(result), [
    "schema",
    "checkedAt",
    "source",
    "github",
    "npm",
    "crates",
    "githubAssets",
    "marketplace",
    "openVsx",
    "evidence",
    "success",
  ]);
  assert.equal(
    fixture.calls.some(({ url }) => url.startsWith("https://jsr.io/")),
    false,
  );
});

test("disabled frozen policy captures all authorities and actual producer bytes without public publication claims", async (t) => {
  const fixture = jsrPublicFixture(t, false);
  for (const name of jsrAuthorityPaths)
    assert.ok(fixture.plan.authority.blobs.some((blob) => blob.path === name));
  const sourceNames = [...fixture.sources.keys()].sort();
  assert.deepEqual(sourceNames, [
    "/LICENSE",
    "/README.md",
    "/config.ts",
    "/jsr.json",
    "/mod.ts",
    "/native.ts",
    "/vite.ts",
  ]);
  for (const source of fixture.plan.jsr!.sources) {
    const bytes = fixture.sources.get(source.path)!;
    assert.equal(source.sha256, sha256(bytes));
    assert.equal(source.bytes, bytes.length);
  }
  fixture.write("jsr/vize/channel.json", '{"schema":"vize-jsr-channel-v1","enabled":true}');
  fixture.write("jsr/vize/README.md", "working-tree substitution");
  assert.deepEqual(derivePublicationPlan(fixture.root, fixture.head), fixture.plan);
  fixture.responses.delete(fixture.metadataUrl);
  const result = await verifyPublication(fixture.plan, fixture.options);
  assert.deepEqual(result.jsr, { required: false, name: "@vizejs/vize", version });
  assert.equal(
    fixture.calls.some(({ url }) => url.startsWith("https://jsr.io/")),
    false,
  );
});

test("enabled frozen H requires exact public seven-file bytes and all five official consumer/publication jobs", async (t) => {
  const fixture = jsrPublicFixture(t);
  const result = await verifyPublication(fixture.plan, fixture.options);
  assert.equal(result.jsr?.required, true);
  assert.deepEqual(
    result.jsr?.jobs?.map((job) => job.name),
    [...jsrJobNames],
  );
  assert.equal(result.jsr?.sources?.length, 7);
  assert.equal(fixture.calls.filter(({ url }) => url.startsWith("https://jsr.io/")).length, 9);
  for (const { url, init } of fixture.calls.filter(({ url }) =>
    url.startsWith("https://jsr.io/"),
  )) {
    assert.equal(init?.credentials, "omit", url);
    assert.equal(init?.redirect, "error", url);
  }
});

test("partial JSR authority, malformed policy and tracked symlinks cannot silently disable the lane", async (t) => {
  const legacy = sourceFixture(t);
  legacy.write("jsr/vize/channel.json", '{"schema":"vize-jsr-channel-v1","enabled":false}');
  legacy.git("add", ".");
  legacy.git("commit", "--amend", "--no-edit", "-q");
  assert.throws(
    () => derivePublicationPlan(legacy.root, legacy.git("rev-parse", "HEAD")),
    /normal source blob required/,
  );
  const fixture = jsrPublicFixture(t, false);
  for (const policy of [
    { enabled: false },
    { schema: "other", enabled: false },
    { schema: "vize-jsr-channel-v1", enabled: "false" },
    { schema: "vize-jsr-channel-v1", enabled: false, optional: true },
  ]) {
    fixture.write("jsr/vize/channel.json", JSON.stringify(policy));
    fixture.git("add", ".");
    fixture.git("commit", "--amend", "--no-edit", "-q");
    assert.throws(
      () => derivePublicationPlan(fixture.root, fixture.git("rev-parse", "HEAD")),
      /exact JSR channel policy/,
    );
  }
  fs.rmSync(path.join(fixture.root, "jsr/vize/channel.json"));
  fs.symlinkSync("README.md", path.join(fixture.root, "jsr/vize/channel.json"));
  fixture.git("add", ".");
  fixture.git("commit", "--amend", "--no-edit", "-q");
  assert.throws(
    () => derivePublicationPlan(fixture.root, fixture.git("rev-parse", "HEAD")),
    /normal source blob required/,
  );
});

test("disabled lane still rejects unsupported exports/includes and unaligned or unpublished npm dependencies", async (t) => {
  const fixture = jsrPublicFixture(t, false);
  const template = JSON.parse(
    fs.readFileSync(path.join(fixture.root, "jsr/vize/jsr.json"), "utf8"),
  );
  for (const bad of [
    { ...template, exports: { ...template.exports, "./other": "./extra.ts" } },
    { ...template, publish: { include: ["**/*"] } },
  ]) {
    fixture.write("jsr/vize/jsr.json", JSON.stringify(bad));
    fixture.git("add", ".");
    fixture.git("commit", "--amend", "--no-edit", "-q");
    assert.throws(
      () => derivePublicationPlan(fixture.root, fixture.git("rev-parse", "HEAD")),
      /exact.*JSR/,
    );
  }
  fixture.write("jsr/vize/jsr.json", JSON.stringify(template));
  fixture.write(
    "npm/builder/vite/package.json",
    JSON.stringify({ name: "@vizejs/vite-plugin", version: "0.1.0" }),
  );
  fixture.git("add", ".");
  fixture.git("commit", "--amend", "--no-edit", "-q");
  assert.throws(
    () => derivePublicationPlan(fixture.root, fixture.git("rev-parse", "HEAD")),
    /source npm version|JSR npm dependency/,
  );
  fixture.write(
    "npm/builder/vite/package.json",
    JSON.stringify({ name: "@vizejs/vite-plugin", version }),
  );
  const release = fs.readFileSync(path.join(fixture.root, ".github/workflows/release.yml"), "utf8");
  fixture.write(
    ".github/workflows/release.yml",
    release
      .split("\n")
      .filter((line) => !line.includes(" -- npm/builder/vite --provenance"))
      .join("\n"),
  );
  fixture.git("add", ".");
  fixture.git("commit", "--amend", "--no-edit", "-q");
  assert.throws(
    () => derivePublicationPlan(fixture.root, fixture.git("rev-parse", "HEAD")),
    /exact published JSR npm dependency/,
  );
});

test("exact JSR package scope, linked repository, version and non-yanked identity are mandatory", async (t) => {
  const fixture = jsrPublicFixture(t);
  const valid = fixture.responses.get(fixture.metadataUrl) as Record<string, unknown>;
  for (const bad of [
    { ...valid, scope: "other" },
    { ...valid, name: "other" },
    { ...valid, githubRepository: { owner: "foreign", name: "vize" } },
    { ...valid, versions: { "0.1.0": {} } },
    { ...valid, versions: { [version]: { yanked: true } } },
    { ...valid, versions: { [version]: { yanked: "false" } } },
  ]) {
    fixture.responses.set(fixture.metadataUrl, bad);
    await assert.rejects(
      verifyPublication(fixture.plan, fixture.options),
      /JSR (object|exact package)/,
    );
  }
});

test("exports, missing/extraneous files, metadata checksums and downloaded bytes must match frozen source", async (t) => {
  const fixture = jsrPublicFixture(t);
  const valid = fixture.responses.get(fixture.versionMetadataUrl) as {
    exports: Record<string, string>;
    manifest: Record<string, unknown>;
  };
  for (const bad of [
    { ...valid, exports: { ...valid.exports, "./native": "./other.ts" } },
    {
      ...valid,
      manifest: {
        ...valid.manifest,
        "/extra.ts": { size: 1, checksum: `sha256-${"a".repeat(64)}` },
      },
    },
    {
      ...valid,
      manifest: Object.fromEntries(
        Object.entries(valid.manifest).filter(([name]) => name !== "/jsr.json"),
      ),
    },
    {
      ...valid,
      manifest: {
        ...valid.manifest,
        "/mod.ts": { size: 75, checksum: `sha256-${"a".repeat(64)}` },
      },
    },
  ]) {
    fixture.responses.set(fixture.versionMetadataUrl, bad);
    await assert.rejects(
      verifyPublication(fixture.plan, fixture.options),
      /JSR (exact exports|exact published|frozen source)/,
    );
  }
  fixture.responses.set(fixture.versionMetadataUrl, valid);
  const url = `https://jsr.io/@vizejs/vize/${version}/mod.ts`;
  fixture.responses.set(url, Buffer.from("changed public module bytes"));
  await assert.rejects(
    verifyPublication(fixture.plan, fixture.options),
    /JSR downloaded source checksum/,
  );
});

test("every exact official JSR job must succeed; missing, skipped, duplicate, foreign R/H fail closed", async (t) => {
  const fixture = jsrPublicFixture(t);
  const setJobs = (jobs: typeof fixture.jobs) =>
    fixture.responses.set(fixture.jobsUrl, { total_count: jobs.length, jobs });
  for (const index of fixture.jobs.keys()) {
    setJobs(fixture.jobs.filter((_, i) => i !== index));
    await assert.rejects(
      verifyPublication(fixture.plan, fixture.options),
      /successful official Release JSR job/,
    );
    setJobs(fixture.jobs.map((job, i) => (i === index ? { ...job, conclusion: "skipped" } : job)));
    await assert.rejects(
      verifyPublication(fixture.plan, fixture.options),
      /successful official Release JSR job/,
    );
  }
  setJobs([...fixture.jobs, { ...fixture.jobs[0], id: 999 }]);
  await assert.rejects(
    verifyPublication(fixture.plan, fixture.options),
    /successful official Release JSR job/,
  );
  for (const fields of [{ head_sha: "f".repeat(40) }, { run_id: fixture.run + 1 }]) {
    setJobs(fixture.jobs.map((job, index) => (index === 0 ? { ...job, ...fields } : job)));
    await assert.rejects(
      verifyPublication(fixture.plan, fixture.options),
      /exact-H\/R Release job/,
    );
  }
  setJobs(fixture.jobs);
  const run = fixture.responses.get(gh(`/actions/runs/${fixture.run}`)) as Record<string, unknown>;
  run.run_attempt = 2;
  // Successful reused jobs from an earlier attempt remain bound to the same R/H.
  assert.equal((await verifyPublication(fixture.plan, fixture.options)).jsr?.required, true);
});
