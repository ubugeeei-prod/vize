import assert from "node:assert/strict";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
import type { ReleaseAuthority, ReleaseRequest } from "./model.ts";
import { array, gitBlob, hash, object, sha256, text } from "./value.ts";
import { discard, observedFetch, type Sink } from "./evidence.ts";
import { authenticateFixCustody } from "./fix-custody.ts";
export const before = {
  capsuleBlob: "0dfa8a25e40e4c045d270171026d46b7471dd4e1",
  version: "0.627.0",
  run: "41236623057",
  H: "c2263205e7cf18ee0d738615285de073773bb513",
  S: "c2263205e7cf18ee0d738615285de073773bb513",
  C: "c59b55f18fe9c1b967061167b0901238abba08b3",
} as const;
export type GithubRead = (endpoint: string) => Promise<unknown>;
function requireRequest(request: ReleaseRequest): void {
  assert([11252, 11253].includes(request.issue));
  assert.match(request.version, /^\d+\.\d+\.\d+$/u);
  assert.match(request.releaseRun, /^[1-9]\d*$/u);
  hash(request.capsuleBlob, 40);
  if (request.phase === "before") {
    assert.equal(request.version, before.version);
    assert.equal(request.releaseRun, before.run);
    assert.equal(request.capsuleBlob, before.capsuleBlob);
    assert.deepEqual(request.fixes, []);
  } else {
    assert.equal(request.phase, "after");
    assert.notEqual(request.version, before.version);
    const issues = request.fixes.map((f) => f.issue);
    assert.equal(new Set(issues).size, issues.length);
    assert(
      issues.includes(request.issue),
      "the selected issue's actual merged fix descriptor is required",
    );
    assert(issues.every((issue) => issue === 11252 || issue === 11253));
    for (const fix of request.fixes) {
      hash(fix.originalHead, 40);
      hash(fix.merge, 40);
      assert.equal(fix.pull, fix.issue === 11252 ? 11257 : 11258);
      assert(fix.paths.length > 0);
      assert.equal(new Set(fix.paths).size, fix.paths.length);
    }
  }
}
export async function authenticateRelease(
  request: ReleaseRequest,
  github: GithubRead,
  sink: Sink = discard,
): Promise<ReleaseAuthority> {
  requireRequest(request); // No external read before all required authority inputs exist.
  const observations: ReleaseAuthority["observations"] = [],
    cache = new Map<string, unknown>();
  const read: GithubRead = async (endpoint) => {
    if (!cache.has(endpoint)) {
      const value = await github(endpoint);
      sink("release-api", { endpoint, value });
      cache.set(endpoint, value);
      observations.push({ endpoint, value });
    }
    return cache.get(endpoint);
  };
  const tag = object(await read(`git/ref/tags/v${request.version}`)),
    target = object(tag.object);
  assert.equal(tag.ref, `refs/tags/v${request.version}`);
  assert.equal(target.type, "commit");
  const candidate = hash(target.sha, 40);
  const release = object(await read(`releases/tags/v${request.version}`));
  assert.equal(release.tag_name, `v${request.version}`);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert(Number.isFinite(Date.parse(text(release.published_at))));
  const run = object(await read(`actions/runs/${request.releaseRun}`));
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  const ownership = await readReleaseOwnership(
    request.version,
    candidate,
    request.releaseRun,
    read,
    observedFetch(sink),
  );
  sink("genuine-producer-proof", ownership);
  if (request.phase === "before") {
    assert.equal(candidate, before.C);
    assert.equal(ownership.workflowHead, before.H);
    assert.equal(ownership.producerSource, before.S);
  }
  for (const fix of request.fixes) {
    await authenticateFixCustody(fix, ownership.producerSource, candidate, read);
    const pull = object(await read(`pulls/${fix.pull}`));
    assert.equal(pull.merged, true);
    assert.equal(pull.state, "closed");
    assert.equal(object(pull.base).ref, "main");
    assert.equal(pull.merge_commit_sha, fix.merge);
    assert.equal(object(pull.head).sha, fix.originalHead);
    const compare = object(await read(`compare/${fix.merge}...${ownership.producerSource}`));
    assert.equal(object(compare.merge_base_commit).sha, fix.merge);
    assert.equal(compare.behind_by, 0);
    assert(["ahead", "identical"].includes(text(compare.status)));
    for (const path of fix.paths) {
      const original = object(await read(`contents/${path}?ref=${fix.originalHead}`));
      const integrated = object(await read(`contents/${path}?ref=${fix.merge}`));
      assert.equal(original.path, path);
      assert.equal(integrated.path, path);
      assert.equal(original.type, "file");
      assert.equal(integrated.type, "file");
      assert.equal(
        hash(original.sha, 40),
        hash(integrated.sha, 40),
        "actual squash changed fix source",
      );
      const selected = object(await read(`contents/${path}?ref=${ownership.producerSource}`));
      const candidateFile = object(await read(`contents/${path}?ref=${candidate}`));
      for (const current of [selected, candidateFile]) {
        assert.equal(current.path, path);
        assert.equal(current.type, "file");
        assert.equal(
          hash(current.sha, 40),
          hash(integrated.sha, 40),
          "reviewed fix bytes changed in actual selected source/candidate; explicit reviewed successor required",
        );
      }
    }
  }
  const blob = object(await read(`git/blobs/${request.capsuleBlob}`));
  assert.equal(blob.encoding, "base64");
  const raw = Buffer.from(text(blob.content), "base64");
  assert.equal(gitBlob(raw), request.capsuleBlob);
  const capsule = object(JSON.parse(raw.toString("utf8")));
  assert.equal(capsule.version, request.version);
  assert.equal(capsule.tagSourceSha, candidate);
  assert.equal(capsule.producerSource, ownership.producerSource);
  object(capsule.npmSRI);
  assert(array(capsule.receipts).length > 0);
  const packageFile = object(await read(`contents/npm/cli/package.json?ref=${candidate}`));
  const packageRaw = Buffer.from(text(packageFile.content), "base64");
  assert.equal(gitBlob(packageRaw), packageFile.sha);
  const packageJson = object(JSON.parse(packageRaw.toString("utf8")));
  const names = array(packageJson.files)
    .map(text)
    .filter((name) => /\.d\.(?:ts|mts|cts)$/u.test(name));
  assert.equal(names.length, 10);
  assert.equal(new Set(names).size, 10);
  assert(names.includes("virtual-ts-slots.d.ts"));
  const sdk: ReleaseAuthority["sdk"] = [];
  for (const path of ["package.json", ...names]) {
    const file =
      path === "package.json"
        ? packageFile
        : object(await read(`contents/npm/cli/${path}?ref=${candidate}`));
    assert.equal(file.path, `npm/cli/${path}`);
    assert.equal(file.encoding, "base64");
    const rawFile = Buffer.from(text(file.content), "base64");
    assert.equal(gitBlob(rawFile), file.sha);
    sdk.push({ path, bytes: rawFile.length, sha256: sha256(rawFile) });
  }
  return {
    version: request.version,
    candidate,
    source: ownership.producerSource,
    workflowHead: ownership.workflowHead,
    run: request.releaseRun,
    capsule,
    sdk,
    observations,
    produced: ownership.produced,
  };
}
