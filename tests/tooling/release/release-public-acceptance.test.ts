import assert from "node:assert/strict";
import { test } from "node:test";
import { retirementAbsenceLaw } from "../support/release-retirement-absence-fixtures.ts";
import {
  derivePublicationPlan,
  readRawBlob,
} from "../../../tools/support/release/public_acceptance/plan.ts";
import { verifyPublication as verify } from "../../../tools/support/release/public_acceptance/registry.ts";
import { publicInputs } from "../../../tools/support/release/public_acceptance/inputs.ts";
import {
  installedPackages,
  rejectOverrides,
  runtimeOverrides,
} from "../../../tools/support/release/public_acceptance/installed.ts";
import {
  version,
  tag,
  repository,
  gh,
  sha256,
  sourceFixture,
  publicFixture,
  consumerFixture,
  stalledBodyFixture,
  refuse,
  fixtureUrl,
} from "../support/release-public-acceptance-fixtures.ts";

test("retirement refuses incomplete or ambiguous public absence", retirementAbsenceLaw);
test("raw H plan derives fixture counts and retained authority, ignoring dirty working files", (t) => {
  const s = sourceFixture(t);
  const manifestPath = "npm/mcp-musea/package.json";
  s.write(manifestPath, '{"name":"@vizejs/mcp-musea","version":"9.9.9"}');
  const p = derivePublicationPlan(s.root, s.head);
  assert.equal(p.parentCut, s.cut);
  assert.equal(p.npm.length, 4);
  assert.equal(p.crates.length, 2);
  assert.deepEqual(p.githubAssets, ["zed-vize-extension.tar.gz"]);
  assert.equal(p.authority.commitSha256, sha256(s.git("cat-file", "commit", s.head) + "\n"));
  const blob = p.authority.blobs.find((item) => item.path === manifestPath)!;
  assert.equal(blob.oid, s.git("rev-parse", `${s.head}:${manifestPath}`));
  assert.equal(blob.sha256, sha256(readRawBlob(s.root, s.head, manifestPath)));
  s.git("add", ".");
  s.git("commit", "-qm", "wrong working version");
  assert.throws(
    () => derivePublicationPlan(s.root, s.git("rev-parse", "HEAD")),
    /source npm version/,
  );
});

test("raw authority refuses commit/blob replacements, symlinks and unsupported command shapes", (t) => {
  const s = sourceFixture(t);
  s.write("npm/mcp-musea/package.json", '{"name":"@vizejs/mcp-musea","version":"9.9.9"}');
  s.git("add", ".");
  s.git("commit", "-qm", "replacement fixture");
  s.git("replace", s.head, s.git("rev-parse", "HEAD"));
  assert.throws(() => derivePublicationPlan(s.root, s.head), /replacement objects/);
  assert.throws(() => readRawBlob(s.root, s.head, "Cargo.toml"), /replacement objects/);
  s.git("replace", "-d", s.head);
  const blob = s.git("rev-parse", `${s.head}:npm/mcp-musea/package.json`);
  s.git("replace", blob, s.git("rev-parse", "HEAD:npm/mcp-musea/package.json"));
  assert.throws(() => derivePublicationPlan(s.root, s.head), /replacement objects/);
  s.git("replace", "-d", blob);
  s.git("reset", "--hard", s.head);
  assert.throws(() => derivePublicationPlan(s.root, blob), /H must be a commit/);
  s.git("update-index", "--cacheinfo", `120000,${blob},npm/mcp-musea/package.json`);
  s.git("commit", "-qm", "symlink fixture");
  assert.throws(
    () => readRawBlob(s.root, s.git("rev-parse", "HEAD"), "npm/mcp-musea/package.json"),
    /normal source blob/,
  );
  s.git("reset", "--hard", s.head);
  s.write(
    ".github/workflows/release.yml",
    "run: moon run --target native tools/moon/cmd/publish_npm_package -- npm/native --unknown\n",
  );
  s.git("add", ".");
  s.git("commit", "-qm", "unsupported publish fixture");
  assert.throws(
    () => derivePublicationPlan(s.root, s.git("rev-parse", "HEAD")),
    /unsupported npm publish/,
  );
});

test("public fixture receipt checks every archive and separate channels without tokens or execution claims", async (t) => {
  const f = publicFixture(t);
  const result = await verify(f.plan, f.options);
  assert.equal(result.success, true);
  assert.equal(result.source.cut, f.cut);
  assert.equal(result.source.head, f.head);
  assert.equal(result.npm.length, f.plan.npm.length);
  assert.equal(result.crates.length, f.plan.crates.length);
  for (const item of [...result.npm, ...result.crates])
    assert.equal(item.sha256, sha256(f.responses.get(item.url) as Buffer));
  assert.equal(result.marketplace.exactVersionAvailable, true);
  assert.equal(result.openVsx.status, "available");
  assert.equal(
    result.githubAssets[0].sha256,
    sha256(f.responses.get(result.githubAssets[0].url) as Buffer),
  );
  assert.match(result.evidence, /no signature or installed-product execution claim/);
  for (const call of f.calls) {
    assert.equal(call.init?.credentials, "omit");
    assert.equal(new Headers(call.init?.headers).has("authorization"), false);
    assert.ok(!call.init?.method || call.init.method === "GET");
    assert.doesNotMatch(call.url, /\/latest(?:[/?]|$)/);
  }
});

test("wrong C/H/tag/source PR/R inputs refuse before public reads", async (t) => {
  const f = publicFixture(t);
  for (const { bad, reason } of [
    { bad: { cut: "b".repeat(40) }, reason: /public source C mismatch/ },
    { bad: { head: "b".repeat(40) }, reason: /public source H mismatch/ },
    { bad: { tag: "v0.439.0" }, reason: /tag\/version mismatch/ },
    { bad: { sourcePr: 0 }, reason: /positive source PR required/ },
    { bad: { run: 0 }, reason: /positive Release run required/ },
  ])
    await assert.rejects(verify(f.plan, { ...f.options, ...bad }), reason);
  assert.equal(f.calls.length, 0);
});

test("public tag, frozen source PR and terminal R require exact own-repository identity", async (t) => {
  const f = publicFixture(t);
  await refuse(f, gh(`/git/ref/tags/${tag}`), { "object.type": "commit" }, /annotated tag/);
  await refuse(
    f,
    gh(`/git/tags/${"a".repeat(40)}`),
    { "object.sha": "b".repeat(40) },
    /target H mismatch/,
  );
  await refuse(
    f,
    gh(`/pulls/${f.sourcePr}`),
    { "head.ref": "main", "head.sha": "b".repeat(40), "head.repo.full_name": "other/vize" },
    /source PR/,
  );
  await refuse(
    f,
    gh(`/actions/runs/${f.run}`),
    {
      head_sha: "b".repeat(40),
      head_branch: "main",
      event: "push",
      display_title: "wrong source PR",
      path: ".github/workflows/check.yml",
      status: "in_progress",
      conclusion: "failure",
      "repository.full_name": "other/vize",
      "head_repository.full_name": "other/vize",
    },
    /Pinned Release/,
  );
  await refuse(
    f,
    gh(`/releases/tags/${tag}`),
    {
      draft: true,
      prerelease: true,
      published_at: null,
      tag_name: "v0.439.0",
    },
    /Release identity mismatch/,
  );
});

test("npm exact-version, SHA512 SRI and actual archive bytes are mandatory", async (t) => {
  const f = publicFixture(t),
    item = f.plan.npm[0];
  const url = `https://registry.npmjs.org/${encodeURIComponent(item.name)}/${version}`;
  await refuse(f, url, { version: "0.439.0" }, /npm exact version/);
  await refuse(f, url, { "dist.integrity": "sha1-legacy" }, /SHA512 SRI/);
  await refuse(f, url, { "dist.tarball": "http://127.0.0.1/internal" }, /download host/);
  const metadata = f.responses.get(url) as { dist: { tarball: string } };
  f.responses.set(metadata.dist.tarball, Buffer.from("substituted archive fixture"));
  await assert.rejects(verify(f.plan, f.options), /npm SRI mismatch/);
});

test("non-yanked exact crates and checksum bytes are mandatory; Marketplace and Open VSX are separate", async (t) => {
  const f = publicFixture(t),
    item = f.plan.crates[0];
  await refuse(
    f,
    `https://crates.io/api/v1/crates/${item.name}/${version}`,
    {
      "version.num": "0.439.0",
      "version.crate": "other",
      "version.yanked": true,
      "version.checksum": "a".repeat(64),
    },
    /crate .*mismatch/,
  );
  await refuse(f, f.marketplaceUrl, { versions: [{ version: "0.439.0" }] }, /Marketplace/);
  (f.responses.get(f.openVsxUrl) as { version: string }).version = "0.439.0";
  const result = await verify(f.plan, f.options);
  assert.equal(result.success, true);
  assert.equal(result.openVsx.optional, true);
  assert.equal(result.openVsx.status, "missing");
});

test("public HTTP failure makes one bounded attempt without publication or retry", async (t) => {
  const f = publicFixture(t);
  let reads = 0;
  const fetch: typeof globalThis.fetch = async () => {
    reads += 1;
    return new Response("not available", { status: 503 });
  };
  await assert.rejects(verify(f.plan, { ...f.options, fetch }), /public HTTP 503/);
  assert.equal(reads, 1);
});

test("own GitHub Zed archive must match its returned sidecar; no upstream Zed endpoint", async (t) => {
  const f = publicFixture(t);
  const url = `https://github.com/${repository}/releases/download/${tag}/zed-vize-extension.tar.gz`;
  f.responses.set(url, Buffer.from("substituted own archive fixture"));
  await assert.rejects(verify(f.plan, f.options), /editor asset checksum mismatch/);
  assert.ok(f.calls.every((call) => !call.url.includes("zed-industries")));
  f.responses.set(f.openVsxUrl, new Response("optional registry unavailable", { status: 503 }));
  f.plan.githubAssets = [];
  const result = await verify(f.plan, f.options);
  assert.equal(result.success, true);
  assert.equal(result.openVsx.status, "error");
});

test("public inputs require full C/H and positive exact PR/R; runtime overrides always refuse", () => {
  const environment = {
    PUBLIC_CUT: "c".repeat(40),
    PUBLIC_HEAD: "b".repeat(40),
    PUBLIC_TAG: tag,
    PUBLIC_SOURCE_PR: "42",
    PUBLIC_RELEASE_RUN: "77",
  };
  assert.deepEqual(publicInputs(environment), {
    cut: environment.PUBLIC_CUT,
    head: environment.PUBLIC_HEAD,
    tag,
    sourcePr: "42",
    run: "77",
  });
  for (const key of Object.keys(environment)) {
    assert.throws(() => publicInputs({ ...environment, [key]: "" }));
  }
  assert.throws(() => publicInputs({ ...environment, PUBLIC_RELEASE_RUN: "9007199254740992" }));
  assert.throws(() => publicInputs({ ...environment, PUBLIC_TAG: "v0.438.1" }));
  assert.doesNotThrow(() => rejectOverrides({}));
  for (const key of runtimeOverrides) {
    assert.doesNotThrow(() => rejectOverrides({ [key]: "" }));
    assert.throws(() => rejectOverrides({ [key]: "1" }), /runtime override/);
  }
});

test(
  "public read timeout covers the body and aborts without waiting for a stalled fixture",
  stalledBodyFixture,
);

test("pin markers, annotation fields and every latest Release job preserve immutable custody", async (t) => {
  const f = publicFixture(t);
  const prUrl = gh(`/pulls/${f.sourcePr}`);
  const pr = f.responses.get(prUrl) as { body: string; state: string };
  await refuse(
    f,
    prUrl,
    { merged: true, body: pr.body.replace(f.cut, "b".repeat(40)) },
    /source PR|marker/,
  );
  await refuse(
    f,
    prUrl,
    { body: pr.body + "<!-- vize-release-pin: immutable-v1 -->" },
    /ambiguous source marker/,
  );
  const annotationUrl = gh(`/git/tags/${"a".repeat(40)}`);
  const annotation = f.responses.get(annotationUrl) as { message: string };
  await refuse(
    f,
    annotationUrl,
    { message: annotation.message + `Validated-run: ${f.run}\n` },
    /annotated pin field/,
  );
  const jobsUrl = gh(`/actions/runs/${f.run}/jobs?filter=latest&per_page=100&page=1`);
  const packet = f.responses.get(jobsUrl) as {
    total_count: number;
    jobs: Record<string, unknown>[];
  };
  await refuse(
    f,
    jobsUrl,
    {
      "jobs.0.conclusion": "failure",
      "jobs.1.status": "in_progress",
      "jobs.0.run_id": 88,
      total_count: 1,
    },
    /Release job/,
  );
  pr.state = "closed";
  (f.responses.get(gh(`/actions/runs/${f.run}`)) as { run_attempt: number }).run_attempt = 2;
  packet.total_count = 3;
  f.responses.set(gh(`/actions/runs/${f.run}/jobs?filter=latest&per_page=100&page=2`), {
    total_count: 3,
    jobs: [{ ...packet.jobs[0], id: 3, run_attempt: 2 }],
  });
  const result = await verify(f.plan, f.options);
  assert.equal(result.github.jobs.length, 3);
  assert.equal(result.github.sourceState, "closed");
  let runReads = 0;
  const fetch: typeof globalThis.fetch = (url, init) =>
    fixtureUrl(url) === gh(`/actions/runs/${f.run}`) && ++runReads === 2
      ? Promise.resolve(
          new Response(
            JSON.stringify({
              ...(f.responses.get(fixtureUrl(url)) as object),
              head_sha: "b".repeat(40),
            }),
          ),
        )
      : f.options.fetch(url, init);
  await assert.rejects(verify(f.plan, { ...f.options, fetch }), /changed during jobs read/);
});

test("lock3 missing optional platforms preserve four installed packages; required payloads never skip", (t) => {
  const f = consumerFixture(t);
  const records = installedPackages(version);
  assert.deepEqual(
    records
      .filter((record) => record.installed)
      .map((record) => record.name)
      .sort(),
    [...f.names].sort(),
  );
  assert.deepEqual(
    records.find((record) => record.name === "@vizejs/native-darwin-arm64"),
    {
      name: "@vizejs/native-darwin-arm64",
      version,
      resolved: f.packages[f.optionalLocation].resolved,
      integrity: f.packages[f.optionalLocation].integrity,
      installed: false,
    },
  );
  f.packages[f.optionalLocation].optional = false;
  f.writeLock();
  assert.throws(() => installedPackages(version), /ENOENT|missing/);
  f.packages[f.optionalLocation].optional = true;
  f.packages["node_modules/@vizejs/native-linux-x64-gnu"].optional = true;
  f.writeLock();
  f.remove("@vizejs/native-linux-x64-gnu");
  assert.throws(() => installedPackages(version), assert.AssertionError);
});
