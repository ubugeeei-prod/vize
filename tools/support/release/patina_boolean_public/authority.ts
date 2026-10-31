import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { asRecord, asString, parseJson } from "../../json-value.ts";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { githubSnapshot } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
import { gitBlob, validateCapsule } from "../constructor_original27_public/authority.ts";
import { requiredMerge } from "./contract.ts";

export function assertAncestry(value: unknown, base: string): void {
  const comparison = asRecord(value);
  assert.equal(asRecord(comparison.base_commit).sha, base);
  assert.equal(asRecord(comparison.merge_base_commit).sha, base);
  assert.equal(comparison.behind_by, 0);
  assert(["ahead", "identical"].includes(asString(comparison.status)));
}

/** Issue-owned authority; no constructor/editor literal release or source is admitted. */
export async function authenticate(version: string, run: string, blob: string, record: Recorder) {
  assert.match(version, /^0\.[1-9]\d*\.0$/u);
  assert.match(run, /^[1-9]\d*$/u);
  assert.match(blob, /^[a-f0-9]{40}$/u);
  const observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const snapshot = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...snapshot });
    record.json("github-observations.json", observations);
    return snapshot.value;
  };
  const response = asRecord(await read("git/blobs/" + blob));
  assert.equal(response.encoding, "base64");
  const raw = Buffer.from(asString(response.content), "base64");
  assert.equal(gitBlob(raw), blob);
  assert.equal(response.sha, blob);
  writeFileSync(join(record.root, "authority-capsule.json"), raw, { flag: "wx" });
  const capsule = asRecord(parseJson(raw.toString("utf8")));
  const tag = asRecord(await read("git/ref/tags/v" + version));
  assert.equal(tag.ref, "refs/tags/v" + version);
  assert.equal(asRecord(tag.object).type, "commit");
  const candidate = asString(asRecord(tag.object).sha);
  const publication = asRecord(await read("releases/tags/v" + version));
  assert.equal(publication.tag_name, "v" + version);
  assert.equal(publication.draft, false);
  assert.equal(publication.prerelease, false);
  assert(Number.isFinite(Date.parse(asString(publication.published_at))));
  const producer = asRecord(await read("actions/runs/" + run));
  assert.equal(producer.status, "completed");
  assert.equal(producer.conclusion, "success");
  const ownership = await readReleaseOwnership(version, candidate, run, read);
  validateCapsule(capsule, version, candidate, ownership.producerSource);
  for (const [base, head] of [
    [requiredMerge, ownership.producerSource],
    [ownership.producerSource, candidate],
  ]) {
    assertAncestry(await read("compare/" + base + "..." + head), base);
  }
  const lock = asRecord(await read("contents/Cargo.lock?ref=" + ownership.producerSource));
  assert.equal(lock.path, "Cargo.lock");
  assert.equal(lock.encoding, "base64");
  const lockBytes = Buffer.from(asString(lock.content), "base64");
  assert.equal(gitBlob(lockBytes), lock.sha);
  const sdk = /\[\[package\]\]\nname = "corsa"\n[\s\S]*?(?=\n\[\[package\]\]|$)/u.exec(
    lockBytes.toString("utf8"),
  )?.[0];
  assert(sdk, "genuine producer Cargo.lock must identify the Corsa SDK");
  writeFileSync(join(record.root, "producer-Cargo.lock"), lockBytes, { flag: "wx" });
  record.json("release-authority.json", {
    version,
    run,
    capsuleBlob: blob,
    capsuleSha256: sha256(raw),
    candidate,
    ownership,
    requiredMerge,
    sdk: {
      source: ownership.producerSource,
      cargoLockBlob: lock.sha,
      cargoLockSha256: sha256(lockBytes),
      packageBlock: sdk,
      limit:
        "Source-locked SDK identity bound to the genuine native producer; not runtime SDK self-introspection.",
    },
  });
  return { capsule, candidate, ownership };
}
