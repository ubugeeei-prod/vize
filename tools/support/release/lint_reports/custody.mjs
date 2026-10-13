import assert from "node:assert/strict";
import { apiRoot, authenticate, optionsFrom } from "../typed_defaults/custody.mjs";
export const fixes = [
  { number: 8765, merge: "d5a25a89540198f9cfb50cb9e2e6e74a24d6d14e" },
  { number: 8761, merge: "5c8f3825fd2eb698c27d3b97d0a47b0671b33e01" },
];
export const extraOverrides = [
  "CORSA_BIN",
  "VIZE_TEST_DISABLE_TSGO",
  "VIZE_TEST_REQUIRE_TSGO",
  "CARGO_BIN_EXE_vize",
  "SOURCE_SHA",
  "VIZE_CORSA_DEV_PATHS",
];
export function reportOptions(env) {
  for (const key of extraOverrides) assert(!env[key], `runtime override must be absent: ${key}`);
  return optionsFrom(env);
}
export async function authenticateReports(options, json) {
  // Reuse the full existing official release validator and its original typed
  // correction requirement. No weaker duplicate C/H/tag/PR/R validator.
  const official = await authenticate(options, json);
  const reports = [];
  for (const expected of fixes) {
    const fix = await json(`${apiRoot}/pulls/${expected.number}`);
    assert.equal(fix.number, expected.number);
    assert.equal(fix.merged, true);
    assert.equal(fix.merge_commit_sha, expected.merge);
    assert.equal(fix.base.repo.full_name, "ubugeeei-prod/vize");
    assert.equal(fix.head.repo.full_name, "ubugeeei-prod/vize");
    const ancestry = await json(`${apiRoot}/compare/${expected.merge}...${options.head}`);
    assert.equal(ancestry.base_commit.sha, expected.merge);
    assert.equal(ancestry.merge_base_commit.sha, expected.merge);
    assert(["ahead", "identical"].includes(ancestry.status));
    reports.push({ fix, ancestry });
  }
  return { official, reports };
}
