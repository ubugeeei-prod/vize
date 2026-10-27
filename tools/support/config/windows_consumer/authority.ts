import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { asRecord, asString, parseJson } from "../../json-value.ts";
import { Recorder, success } from "./common.ts";
assert.equal(
  process.env.GITHUB_REF,
  "refs/heads/main",
  "explicit public mode must fail outside the merged workflow",
);
const [version, source, blob, output, evidence] = process.argv.slice(2);
assert.ok(version && source && blob && output && evidence);
assert.match(version, /^[0-9]+\.[0-9]+\.[0-9]+$/);
assert.match(source, /^[0-9a-f]{40}$/);
assert.match(blob, /^[0-9a-f]{40}$/);
const record = new Recorder(evidence);
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
const pr = record.run(
  "actual-10990-merge",
  ["gh", "api", "repos/ubugeeei-prod/vize/pulls/10990"],
  process.cwd(),
);
success(pr);
const merged = asRecord(parseJson(pr.stdout));
assert.equal(merged.merged, true);
assert.equal(authority.required10990MergeSha, merged.merge_commit_sha);
success(
  record.run(
    "immutable-tag-fetch",
    ["git", "fetch", "--no-tags", "--depth=1", "origin", "refs/tags/v" + version],
    process.cwd(),
  ),
);
const tag = record.run(
  "immutable-tag-source",
  ["git", "rev-parse", "FETCH_HEAD^{commit}"],
  process.cwd(),
);
success(tag);
assert.equal(tag.stdout.trim(), source);
writeFileSync(output, raw);
