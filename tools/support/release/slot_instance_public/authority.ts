import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { sha256 } from "../computed_event_custody.ts";
import { githubSnapshot } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
import { expected, profiles, type Role } from "./contract.ts";

export const gitBlob = (raw: Buffer) =>
  createHash("sha1")
    .update("blob " + raw.length + "\0")
    .update(raw)
    .digest("hex");
export function validateCapsule(role: Role, capsule: JsonRecord) {
  const profile = profiles[role];
  assert.equal(capsule.version, profile.version);
  assert.equal(capsule.tagSourceSha, profile.candidate);
  assert.equal(capsule.producerSource, profile.source);
  assert.ok(asArray(capsule.receipts).length > 0);
  asRecord(capsule.npmSRI);
}
export function assertLockedPackage(
  lock: JsonRecord,
  capsule: JsonRecord,
  name: string,
  version: string,
) {
  const row = asRecord(asRecord(lock.packages)["node_modules/" + name]);
  const wanted = asRecord(asRecord(capsule.npmSRI)[name]);
  assert.equal(row.version, version);
  assert.equal(row.link, undefined);
  assert.equal(new URL(asString(wanted.metadataOrigin)).origin, "https://registry.npmjs.org");
  assert.match(asString(wanted.integrity), /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
  assert.equal(row.integrity, wanted.integrity);
  assert.equal(row.resolved, wanted.tarball);
  assert.equal(new URL(asString(row.resolved)).origin, "https://registry.npmjs.org");
}
export function validateNative(
  role: Role,
  capsule: JsonRecord,
  image: { bytes: number; sha256: string },
  token: string,
) {
  let matches = 0;
  for (const value of asArray(capsule.receipts)) {
    const item = asRecord(value),
      origin = asRecord(item.origin);
    assert.equal(String(origin.runId), profiles[role].run);
    assert.ok(Number.isSafeInteger(origin.artifactId) && Number(origin.artifactId) > 0);
    assert.ok(asString(origin.member));
    const raw = Buffer.from(asString(item.rawBase64), "base64");
    assert.equal(sha256(raw), item.sha256);
    const receipt = asRecord(parseJson(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.version, profiles[role].version);
    assert.equal(receipt.source, profiles[role].candidate);
    const provider = asRecord(receipt.provider);
    assert.equal(provider.source, profiles[role].candidate);
    assert.equal(provider.releaseVersion, profiles[role].version);
    if (provider.targetPlatform !== "linux-x64") continue;
    const files = asArray(provider.files)
      .map((value) => asRecord(value))
      .filter((file) => file.path === provider.nodeFile);
    assert.equal(files.length, 1);
    assert.equal(files[0].bytes, image.bytes);
    assert.equal(files[0].sha256, image.sha256);
    assert.equal(provider.nativeBuildToken, token);
    matches++;
  }
  assert.equal(matches, 1, "exactly one genuine Linux producer image joins this installation");
}
export async function authenticate(role: Role, blob: string, record: Recorder) {
  assert.match(blob, /^[a-f0-9]{40}$/u);
  const profile = profiles[role],
    observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const snapshot = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...snapshot });
    record.json("github-observations.json", observations);
    return snapshot.value;
  };
  const rawBlob = asRecord(await read("git/blobs/" + blob));
  assert.equal(rawBlob.encoding, "base64");
  const raw = Buffer.from(asString(rawBlob.content), "base64");
  writeFileSync(join(record.root, "authority-capsule.json"), raw, { flag: "wx" });
  assert.equal(gitBlob(raw), blob);
  assert.equal(rawBlob.sha, blob);
  const capsule = asRecord(parseJson(raw.toString("utf8")));
  validateCapsule(role, capsule);
  const publication = asRecord(await read("releases/tags/v" + profile.version));
  assert.equal(publication.tag_name, "v" + profile.version);
  assert.equal(publication.draft, false);
  assert.equal(publication.prerelease, false);
  assert.ok(Number.isFinite(Date.parse(asString(publication.published_at))));
  const producer = asRecord(await read("actions/runs/" + profile.run));
  assert.equal(producer.status, "completed");
  assert.equal(producer.conclusion, "success");
  const ownership = await readReleaseOwnership(
    profile.version,
    profile.candidate,
    profile.run,
    read,
  );
  assert.equal(ownership.producerSource, profile.source);
  const sdk = asArray(expected(role, "esm").sdk).map((value) => asRecord(value)),
    sourceFiles = [];
  for (const name of ["package.json", ...sdk.map((row) => asString(row.file))]) {
    const file = asRecord(await read("contents/npm/cli/" + name + "?ref=" + profile.candidate));
    assert.equal(file.type, "file");
    assert.equal(file.path, "npm/cli/" + name);
    assert.equal(file.encoding, "base64");
    const body = Buffer.from(asString(file.content), "base64");
    assert.equal(gitBlob(body), file.sha);
    const wanted = sdk.find((row) => row.file === name);
    if (wanted) {
      assert.equal(body.length, wanted.bytes);
      assert.equal(sha256(body), wanted.sha256);
    }
    sourceFiles.push({
      file: name,
      bytes: body.length,
      sha256: sha256(body),
      gitBlob: file.sha,
      rawBase64: body.toString("base64"),
    });
  }
  const packageManifest = asRecord(
    parseJson(Buffer.from(sourceFiles[0].rawBase64, "base64").toString("utf8")),
  );
  record.json("release-authority.json", { role, profile, blob, ownership, sourceFiles });
  return { role, profile, capsule, ownership, packageManifest, sourceFiles };
}
