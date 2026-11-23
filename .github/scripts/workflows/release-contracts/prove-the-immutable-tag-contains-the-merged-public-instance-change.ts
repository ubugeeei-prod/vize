import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
const comparison = JSON.parse(
  readFileSync(process.env.RUNNER_TEMP + "/cpp-source-comparison.json", "utf8"),
);
assert.equal(comparison.base_commit.sha, process.env.REQUIRED_CPP_SOURCE);
assert.ok(["ahead", "identical"].includes(comparison.status));
writeFileSync(
  process.env.RUNNER_TEMP + "/cpp-release-identity.json",
  JSON.stringify(
    {
      version: process.env.PUBLIC_VERSION,
      sourceSha: process.env.PUBLIC_SOURCE_SHA,
      requiredSource: process.env.REQUIRED_CPP_SOURCE,
      containsRequiredSource: true,
      comparisonStatus: comparison.status,
      workflowSha: process.env.GITHUB_SHA,
      runId: process.env.GITHUB_RUN_ID,
    },
    null,
    2,
  ) + "\n",
  { flag: "wx" },
);
