import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, asString, type JsonRecord } from "../../../support/json-value.ts";
import { githubSnapshot } from "../../../support/release/release/quota/http.ts";
import { readReleaseOwnership } from "../../../support/release/release/quota_ownership.ts";
import { publicUrl, sriSha512 } from "../../../support/release/public_install/registry.ts";
import { profiles, sha256, type Role } from "./contract.ts";
import type { Recorder } from "./execute.ts";

export const sdkNames = [
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
  "virtual-ts-slots.d.ts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
] as const;
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
  const targets: string[] = [];
  for (const value of asArray(capsule.receipts)) {
    const item = asRecord(value),
      origin = asRecord(item.origin);
    assert.equal(String(origin.runId), profile.run);
    assert.ok(Number.isSafeInteger(origin.artifactId) && Number(origin.artifactId) > 0);
    assert.ok(asString(origin.member));
    const raw = Buffer.from(asString(item.rawBase64), "base64");
    assert.equal(raw.toString("base64"), item.rawBase64);
    assert.equal(sha256(raw), item.sha256);
    const receipt = asRecord(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.version, profile.version);
    assert.equal(receipt.source, profile.candidate);
    const provider = asRecord(receipt.provider);
    assert.equal(provider.source, profile.candidate);
    assert.equal(provider.releaseVersion, profile.version);
    targets.push(asString(provider.targetPlatform));
  }
  assert.deepEqual(targets.sort(), ["darwin-arm64", "darwin-x64", "linux-x64", "win32-x64"]);
  const sri = asRecord(capsule.npmSRI);
  for (const name of ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"]) {
    const entry = asRecord(sri[name]);
    sriSha512(entry.integrity);
    publicUrl(asString(entry.tarball));
    publicUrl(asString(entry.metadataOrigin));
  }
}
export function validateNative(
  role: Role,
  capsule: JsonRecord,
  image: { bytes: number; sha256: string },
  token: string,
) {
  validateCapsule(role, capsule);
  const matching = asArray(capsule.receipts)
    .map((value) => {
      const raw = Buffer.from(asString(asRecord(value).rawBase64), "base64");
      return asRecord(asRecord(JSON.parse(raw.toString("utf8"))).provider);
    })
    .filter((provider) => provider.targetPlatform === "linux-x64");
  assert.equal(matching.length, 1);
  const provider = matching[0]!;
  assert.equal(provider.rustTarget, "x86_64-unknown-linux-gnu");
  const files = asArray(provider.files)
    .map((value) => asRecord(value))
    .filter((file) => file.path === provider.nodeFile);
  assert.equal(files.length, 1);
  assert.equal(files[0]!.bytes, image.bytes);
  assert.equal(files[0]!.sha256, image.sha256);
  assert.equal(provider.nativeBuildToken, token);
}
export async function authenticate(role: Role, record: Recorder) {
  const profile = profiles[role],
    observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const response = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...response });
    record.json("github-observations.json", observations);
    return response.value;
  };
  const value = asRecord(await read("git/blobs/" + profile.capsule));
  assert.equal(value.encoding, "base64");
  const raw = Buffer.from(asString(value.content), "base64");
  writeFileSync(join(record.root, "authority-capsule.json"), raw, { flag: "wx" });
  assert.equal(value.sha, profile.capsule);
  assert.equal(gitBlob(raw), profile.capsule);
  const capsule = asRecord(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
  validateCapsule(role, capsule);
  const release = asRecord(await read("releases/tags/v" + profile.version));
  assert.equal(release.tag_name, "v" + profile.version);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert.ok(Number.isFinite(Date.parse(asString(release.published_at))));
  const run = asRecord(await read("actions/runs/" + profile.run));
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  const ownership = await readReleaseOwnership(
    profile.version,
    profile.candidate,
    profile.run,
    read,
  );
  assert.equal(ownership.producerSource, profile.source);
  const sourceFiles = [];
  for (const name of ["package.json", ...sdkNames]) {
    const file = asRecord(await read("contents/npm/cli/" + name + "?ref=" + profile.candidate));
    assert.equal(file.type, "file");
    assert.equal(file.path, "npm/cli/" + name);
    assert.equal(file.encoding, "base64");
    const body = Buffer.from(asString(file.content), "base64");
    assert.equal(gitBlob(body), file.sha);
    sourceFiles.push({
      file: name,
      bytes: body.length,
      sha256: sha256(body),
      gitBlob: file.sha,
      rawBase64: body.toString("base64"),
    });
  }
  record.json("release-authority.json", { role, profile, ownership, sourceFiles });
  return { capsule, ownership, sourceFiles };
}
