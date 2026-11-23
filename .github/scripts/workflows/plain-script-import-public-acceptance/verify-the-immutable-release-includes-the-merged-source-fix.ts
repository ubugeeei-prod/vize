import { requiredEnv } from "../shared/files.ts";

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
const api = (path: string) => JSON.parse(execFileSync("gh", ["api", path], { encoding: "utf8" }));
assert(/^0\.\d+\.\d+$/.test(requiredEnv("PUBLIC_VERSION")));
assert(/^[a-f0-9]{40}$/.test(requiredEnv("SOURCE_SHA")));
const pr = api("repos/ubugeeei-prod/vize/pulls/10982");
assert.equal(pr.merged, true);
const comparison = api(
  "repos/ubugeeei-prod/vize/compare/" + pr.merge_commit_sha + "..." + process.env.SOURCE_SHA,
);
assert(["ahead", "identical"].includes(comparison.status));
assert.equal(comparison.merge_base_commit.sha, pr.merge_commit_sha);
mkdirSync(requiredEnv("VIZE10903_PROOF"), { recursive: true });
writeFileSync(
  process.env.VIZE10903_PROOF + "/release-provenance.json",
  JSON.stringify(
    {
      version: process.env.PUBLIC_VERSION,
      source: process.env.SOURCE_SHA,
      fix: pr.html_url,
      merge: pr.merge_commit_sha,
      comparison: comparison.html_url,
      status: comparison.status,
    },
    null,
    2,
  ) + "\n",
);
