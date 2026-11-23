import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const comparison = JSON.parse(
  readFileSync(
    process.env.RUNNER_TEMP + "/public-release-proof/icons-source-comparison.json",
    "utf8",
  ),
);
assert.equal(comparison.base_commit.sha, process.env.REQUIRED_ICON_SOURCE);
assert.ok(["ahead", "identical"].includes(comparison.status));
