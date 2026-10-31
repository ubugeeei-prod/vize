import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { asRecord, asString, parseJson } from "../../json-value.ts";
import { Recorder, sha256, success } from "../../config/windows_consumer/common.ts";

export const requiredMerge = "66eba1da97f1b94b3a01f79b3ec9c630e696bb33";
export function authenticate(version: string, source: string, blob: string, record: Recorder) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(
    process.env.GITHUB_REF,
    "refs/heads/main",
    "public execution requires merged workflow",
  );
  assert.match(version, /^\d+\.\d+\.\d+$/);
  for (const sha of [source, blob]) assert.match(sha, /^[0-9a-f]{40}$/);
  const response = record.run(
    "authority-blob",
    ["gh", "api", "repos/ubugeeei-prod/vize/git/blobs/" + blob],
    process.cwd(),
  );
  success(response);
  const raw = Buffer.from(asString(asRecord(parseJson(response.stdout)).content), "base64");
  const authority = asRecord(parseJson(raw.toString("utf8")));
  assert.equal(authority.version, version);
  assert.equal(authority.tagSourceSha, source);
  assert.match(asString(authority.producerSource), /^[0-9a-f]{40}$/);
  // Use the actual merged fix, not a caller's assertion that a version contains it.
  for (const [label, base, head] of [
    ["fix-to-producer", requiredMerge, asString(authority.producerSource)],
    ["producer-to-tag", asString(authority.producerSource), source],
  ]) {
    const result = record.run(
      label,
      ["gh", "api", "repos/ubugeeei-prod/vize/compare/" + base + "..." + head],
      process.cwd(),
    );
    success(result);
    const parsed = asRecord(parseJson(result.stdout));
    assert.ok(["ahead", "identical"].includes(asString(parsed.status)));
    assert.equal(asRecord(parsed.base_commit).sha, base);
  }
  success(
    record.run(
      "tag-fetch",
      ["git", "fetch", "--no-tags", "--depth=1", "origin", "refs/tags/v" + version],
      process.cwd(),
    ),
  );
  const tag = record.run("tag-source", ["git", "rev-parse", "FETCH_HEAD^{commit}"], process.cwd());
  success(tag);
  assert.equal(tag.stdout.trim(), source);
  writeFileSync(record.root + "/authority.json", raw);
  record.json("authority-custody.json", {
    blob,
    bytes: raw.length,
    sha256: sha256(raw),
    version,
    source,
    requiredMerge,
  });
  return authority;
}
