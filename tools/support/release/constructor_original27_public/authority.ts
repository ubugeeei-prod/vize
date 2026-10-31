import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  asArray,
  asRecord,
  asString,
  parseJson,
  type JsonRecord,
} from "../../json-value.ts";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { githubSnapshot } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";

export const requiredMerge = "ccd079c6dfb5c4ff50feddb23a6777154f33fab1";
export const sdkFiles = [
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
] as const;
export const gitBlob = (raw: Buffer) =>
  createHash("sha1")
    .update("blob " + raw.length + "\0")
    .update(raw)
    .digest("hex");
export function validateCapsule(
  capsule: JsonRecord,
  version: string,
  candidate: string,
  source: string,
) {
  assert.equal(capsule.version, version);
  assert.equal(capsule.tagSourceSha, candidate);
  assert.equal(capsule.producerSource, source);
  asRecord(capsule.npmSRI);
  assert.ok(asArray(capsule.receipts).length > 0);
}
export function validateNativeReceipt(
  capsule: JsonRecord,
  identity: { target: string; bytes: number; sha256: string; token: string },
  version: string,
  candidate: string,
  run: string,
) {
  // The manual release owner pins a reviewed capsule after authenticating whole
  // producer ZIPs. This role binds those retained receipt bytes to the installed
  // image; it does not independently download the platform archives again.
  let matched = 0;
  for (const value of asArray(capsule.receipts)) {
    const item = asRecord(value),
      origin = asRecord(item.origin);
    assert.equal(String(origin.runId), run);
    assert.ok(
      Number.isSafeInteger(origin.artifactId) && Number(origin.artifactId) > 0,
    );
    assert.ok(asString(origin.member));
    const raw = Buffer.from(asString(item.rawBase64), "base64");
    assert.equal(sha256(raw), item.sha256);
    const receipt = asRecord(
      parseJson(new TextDecoder("utf-8", { fatal: true }).decode(raw)),
    );
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.version, version);
    assert.equal(receipt.source, candidate);
    const provider = asRecord(receipt.provider);
    assert.equal(provider.source, candidate);
    assert.equal(provider.releaseVersion, version);
    if (provider.targetPlatform !== identity.target) continue;
    const files = asArray(provider.files)
      .map((file) => asRecord(file))
      .filter((file) => file.path === provider.nodeFile);
    assert.equal(files.length, 1);
    assert.equal(files[0].bytes, identity.bytes);
    assert.equal(files[0].sha256, identity.sha256);
    assert.equal(provider.nativeBuildToken, identity.token);
    matched++;
  }
  assert.equal(
    matched,
    1,
    "exactly one genuine producer receipt must match this host",
  );
}
export function assertLockedPackage(
  lock: JsonRecord,
  capsule: JsonRecord,
  name: string,
  version: string,
) {
  const row = asRecord(asRecord(lock.packages)["node_modules/" + name]);
  assert.equal(row.version, version);
  assert.equal(row.link, undefined);
  const expected = asRecord(asRecord(capsule.npmSRI)[name]);
  assert.equal(
    new URL(asString(expected.metadataOrigin)).origin,
    "https://registry.npmjs.org",
  );
  assert.match(asString(expected.integrity), /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
  assert.equal(row.integrity, expected.integrity);
  assert.equal(row.resolved, expected.tarball);
  assert.equal(
    new URL(asString(row.resolved)).origin,
    "https://registry.npmjs.org",
  );
}
export async function authenticate(
  version: string,
  run: string,
  blob: string,
  record: Recorder,
) {
  assert.match(version, /^\d+\.\d+\.\d+$/u);
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
    Date.parse(asString(publication.published_at)) >=
      Date.parse(asString(producer.created_at)),
  );
  const ownership = await readReleaseOwnership(version, candidate, run, read);
  record.json("producer-ownership.json", ownership);
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
    const file = asRecord(
      await read("contents/npm/cli/" + name + "?ref=" + candidate),
    );
    assert.equal(file.type, "file");
    assert.equal(file.path, "npm/cli/" + name);
    assert.equal(file.encoding, "base64");
    const bytes = Buffer.from(asString(file.content), "base64");
    assert.equal(gitBlob(bytes), file.sha);
    if (name === "package.json")
      packageManifest = asRecord(parseJson(bytes.toString("utf8")));
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
  return { capsule, candidate, ownership, sdk, packageManifest };
}
