import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, asString, parseJson } from "../../json-value.ts";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { githubSnapshot } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
import { gitBlob, sdkFiles, validateCapsule } from "../constructor_original27_public/authority.ts";

export const release = {
  version: "0.620.0",
  run: "41216373606",
  candidate: "5695be444d8e5d889d2337fc8e22a974458b93ea",
  source: "e361d3873712eb11db018827713b1434be898261",
} as const;
export { sdkFiles };

export function assertRole(version: string, run: string, candidate: string, source: string) {
  assert.deepEqual(
    { version, run, candidate, source },
    release,
    "this is the immutable C12 nine-declaration role",
  );
}

export async function authenticate(version: string, run: string, blob: string, record: Recorder) {
  assert.equal(version, release.version);
  assert.equal(run, release.run);
  assert.match(blob, /^[a-f0-9]{40}$/u);
  const observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const observed = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...observed });
    record.json("github-observations.json", observations);
    return observed.value;
  };
  const encoded = asRecord(await read("git/blobs/" + blob));
  assert.equal(encoded.encoding, "base64");
  const bytes = Buffer.from(asString(encoded.content), "base64");
  writeFileSync(join(record.root, "authority-capsule.json"), bytes, { flag: "wx" });
  assert.equal(gitBlob(bytes), blob);
  assert.equal(encoded.sha, blob);
  const capsule = asRecord(parseJson(new TextDecoder("utf-8", { fatal: true }).decode(bytes)));
  const tag = asRecord(await read("git/ref/tags/v" + version));
  assert.equal(tag.ref, "refs/tags/v" + version);
  assert.deepEqual(asRecord(tag.object).type, "commit");
  assert.equal(asRecord(tag.object).sha, release.candidate);
  const publication = asRecord(await read("releases/tags/v" + version));
  assert.equal(publication.tag_name, "v" + version);
  assert.equal(publication.draft, false);
  assert.equal(publication.prerelease, false);
  assert.ok(Number.isFinite(Date.parse(asString(publication.published_at))));
  const producer = asRecord(await read("actions/runs/" + run));
  assert.equal(producer.status, "completed");
  assert.equal(producer.conclusion, "success");
  const ownership = await readReleaseOwnership(version, release.candidate, run, read);
  assertRole(version, run, ownership.candidateSha, ownership.producerSource);
  assert.equal(ownership.workflowHead, release.source);
  validateCapsule(capsule, version, release.candidate, release.source);
  assert.ok(asArray(capsule.receipts).length > 0);
  const sdk = [];
  let packageManifest: ReturnType<typeof asRecord> | undefined;
  for (const file of ["package.json", ...sdkFiles]) {
    const remote = asRecord(await read(`contents/npm/cli/${file}?ref=${release.candidate}`));
    assert.equal(remote.type, "file");
    assert.equal(remote.path, "npm/cli/" + file);
    assert.equal(remote.encoding, "base64");
    const raw = Buffer.from(asString(remote.content), "base64");
    assert.equal(gitBlob(raw), remote.sha);
    if (file === "package.json") packageManifest = asRecord(parseJson(raw.toString("utf8")));
    sdk.push({ file, bytes: raw.length, sha256: sha256(raw), gitBlob: remote.sha });
  }
  assert.ok(packageManifest);
  record.json("release-authority.json", {
    role: "C12-original-10980",
    ...release,
    blob,
    ownership,
    sdk,
  });
  return { capsule, candidate: release.candidate, ownership, sdk, packageManifest };
}
