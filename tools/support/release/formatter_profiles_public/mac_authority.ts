import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { asRecord, asString, parseJson } from "../../json-value.ts";
import { type Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { githubSnapshot } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
import { gitBlob, validateCapsule } from "../constructor_original27_public/authority.ts";
import { requiredFix } from "./mac_contract.ts";

/** Each role has its own genuine producer and public-package authority; no inferred old tag. */
export async function authenticateMac(
  version: "0.627.0" | "0.628.0",
  run: string,
  blob: string,
  record: Recorder,
) {
  assert.match(run, /^[1-9]\d*$/u);
  assert.match(blob, /^[a-f0-9]{40}$/u);
  const observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const snapshot = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...snapshot });
    record.json("github-observations.json", observations);
    return snapshot.value;
  };
  const encoded = asRecord(await read("git/blobs/" + blob));
  assert.equal(encoded.encoding, "base64");
  assert.equal(encoded.sha, blob);
  const raw = Buffer.from(asString(encoded.content), "base64");
  assert.equal(gitBlob(raw), blob);
  writeFileSync(join(record.root, "authority-capsule.json"), raw, { flag: "wx" });
  const capsule = asRecord(parseJson(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
  const tag = asRecord(await read("git/ref/tags/v" + version));
  assert.equal(tag.ref, "refs/tags/v" + version);
  const target = asRecord(tag.object);
  assert.equal(target.type, "commit");
  const candidate = asString(target.sha);
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
  if (version === "0.628.0") {
    const ancestry = asRecord(
      await read("compare/" + requiredFix + "..." + ownership.producerSource),
    );
    assert.equal(asRecord(ancestry.base_commit).sha, requiredFix);
    assert.equal(asRecord(ancestry.merge_base_commit).sha, requiredFix);
    assert.equal(ancestry.behind_by, 0);
    assert(["ahead", "identical"].includes(asString(ancestry.status)));
  }
  record.json("release-authority.json", {
    version,
    run,
    blob,
    capsuleSha256: sha256(raw),
    candidate,
    ownership,
    controllerSource: process.env.GITHUB_SHA,
    limitation:
      "Baseline identity is authenticated, but absence of the later fix is not inferred from version ordering",
  });
  return { version, run, capsule, candidate, ownership };
}
export type MacAuthority = Awaited<ReturnType<typeof authenticateMac>>;
