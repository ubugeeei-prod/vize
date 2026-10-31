import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { githubSnapshot } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
// Share only pure capsule/image/SRI primitives. No constructor population,
// source-reference vectors or unrelated merge requirements are inherited.
import { sdkFiles, gitBlob, validateCapsule } from "../constructor_original27_public/authority.ts";
export {
  sdkFiles,
  gitBlob,
  validateCapsule,
  validateNativeReceipt,
  assertLockedPackage,
} from "../constructor_original27_public/authority.ts";
export const publicVersion = "0.616.0";
export const requiredMerge = "b06a78673f41df01fe29d02bcb42b6b30073e451";
export const requiredSource = "1a65bbd5e95da036bd7e6a0fee7f8b214023a6b0";
export function assertPublishedManifest(manifest: JsonRecord, version: string) {
  assert.equal(version, publicVersion);
  assert.equal(manifest.name, "vize");
  assert.equal(manifest.version, version);
  assert.deepEqual(manifest.bin, { vize: "bin/vize", vz: "bin/vize" });
  assert.deepEqual(
    asRecord(asRecord(manifest.typescript).contentMapper).exec,
    ["node", "./bin/vize", "content-mapper"],
    "source or executable overrides are forbidden",
  );
}
export async function authenticate(version: string, run: string, blob: string, record: Recorder) {
  assert.equal(version, publicVersion, "this receiver freezes the literal nine-file 0.616 SDK");
  assert.match(run, /^[1-9]\d*$/u);
  assert.match(blob, /^[a-f0-9]{40}$/u);
  const observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const snapshot = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...snapshot });
    record.json("github-observations.json", observations);
    return snapshot.value;
  };
  const rawBlob = asRecord(await read("git/blobs/" + blob));
  assert.equal(rawBlob.encoding, "base64");
  const raw = Buffer.from(asString(rawBlob.content), "base64");
  writeFileSync(join(record.root, "authority-capsule.json"), raw, {
    flag: "wx",
  });
  assert.equal(gitBlob(raw), blob);
  assert.equal(rawBlob.sha, blob);
  const capsule = asRecord(parseJson(raw.toString("utf8")));
  const tag = asRecord(await read("git/ref/tags/v" + version));
  const target = asRecord(tag.object);
  assert.equal(tag.ref, "refs/tags/v" + version);
  assert.equal(target.type, "commit");
  const candidate = asString(target.sha);
  const publication = asRecord(await read("releases/tags/v" + version));
  assert.equal(publication.tag_name, "v" + version);
  assert.equal(publication.draft, false);
  assert.equal(publication.prerelease, false);
  assert.ok(Number.isFinite(Date.parse(asString(publication.published_at))));
  const producer = asRecord(await read("actions/runs/" + run));
  assert.equal(producer.status, "completed");
  assert.equal(producer.conclusion, "success");
  assert.ok(
    Date.parse(asString(publication.published_at)) >= Date.parse(asString(producer.created_at)),
  );
  const ownership = await readReleaseOwnership(version, candidate, run, read);
  record.json("producer-ownership.json", ownership);
  assert.equal(
    ownership.producerSource,
    requiredSource,
    "first public role requires the reviewed R8 source",
  );
  validateCapsule(capsule, version, candidate, ownership.producerSource);
  const ancestry = asRecord(
    await read("compare/" + requiredMerge + "..." + ownership.producerSource),
  );
  assert.equal(asRecord(ancestry.base_commit).sha, requiredMerge);
  assert.equal(asRecord(ancestry.merge_base_commit).sha, requiredMerge);
  assert.equal(ancestry.behind_by, 0);
  assert.ok(["ahead", "identical"].includes(asString(ancestry.status)));
  const sdk = [];
  let packageManifest: JsonRecord | undefined;
  for (const name of ["package.json", ...sdkFiles]) {
    const file = asRecord(await read("contents/npm/cli/" + name + "?ref=" + candidate));
    assert.equal(file.type, "file");
    assert.equal(file.path, "npm/cli/" + name);
    assert.equal(file.encoding, "base64");
    const bytes = Buffer.from(asString(file.content), "base64");
    assert.equal(gitBlob(bytes), file.sha);
    if (name === "package.json") packageManifest = asRecord(parseJson(bytes.toString("utf8")));
    sdk.push({
      file: name,
      bytes: bytes.length,
      sha256: sha256(bytes),
      gitBlob: file.sha,
    });
  }
  record.json("release-authority.json", {
    version,
    run,
    blob,
    candidate,
    ownership,
    sdk,
  });
  assert.ok(packageManifest);
  assertPublishedManifest(packageManifest, version);
  return { capsule, candidate, ownership, sdk, packageManifest };
}
