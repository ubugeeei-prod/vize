import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, writeJson } from "../shared/files.ts";

import { execFileSync } from "node:child_process";
const root = requiredEnv("EVIDENCE");
const eventBase = requiredEnv("EVENT_BASE_SHA"),
  pullRequestHead = requiredEnv("PR_HEAD_SHA"),
  productBase = requiredEnv("PRODUCT_BASE_SHA"),
  productHead = requiredEnv("PRODUCT_HEAD_SHA");
const actualParents = execFileSync("git", ["show", "-s", "--format=%P", "HEAD"], {
  encoding: "utf8",
})
  .trim()
  .split(/\s+/u);
assert.deepEqual(actualParents, [productBase, pullRequestHead]);
writeJson(join(root, "provenance.json"), {
  eventBase,
  pullRequestHead,
  productBase,
  productHead,
  driver: productHead,
  actualParents,
  normalSource: "unmodified committed products; separate targets from all profiling overlays",
  observerPurpose: "correctness",
  eventName: requiredEnv("GITHUB_EVENT_NAME"),
});
