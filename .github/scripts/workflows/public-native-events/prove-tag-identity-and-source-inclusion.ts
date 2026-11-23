import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
const directory = `${process.env.RUNNER_TEMP}/event-receipts`;
const comparison = JSON.parse(readFileSync(`${directory}/source-comparison.json`, "utf8"));
assert.equal(comparison.base_commit.sha, process.env.REQUIRED_SOURCE);
const containsRequiredSource = ["ahead", "identical"].includes(comparison.status);
assert.equal(containsRequiredSource, process.env.PUBLIC_PHASE === "after");
if (process.env.PUBLIC_PHASE === "before") assert.equal(process.env.PUBLIC_VERSION, "0.599.0");
writeFileSync(
  `${directory}/release-identity.json`,
  JSON.stringify(
    {
      version: process.env.PUBLIC_VERSION,
      tag: `v${process.env.PUBLIC_VERSION}`,
      sourceSha: process.env.PUBLIC_SOURCE_SHA,
      requiredSource: process.env.REQUIRED_SOURCE,
      containsRequiredSource,
      comparisonStatus: comparison.status,
      workflowSha: process.env.GITHUB_SHA,
      runId: process.env.GITHUB_RUN_ID,
    },
    null,
    2,
  ) + "\n",
  { flag: "wx" },
);
