import assert from "node:assert/strict";
import { apiRoot, repository } from "./custody.mjs";

export const reservedFixes = [
  { pr: 8668, merge: "19e8fa700d3c79631166bcaed4e3cfe65dda1903" },
  { pr: 8677, merge: "0de4eb3969cf230c2464e0d1edf1734b0dc388b8" },
  { pr: 8684, merge: "38af95bc04d6a8872cc5f045d60412bcaf19f70f" },
];

/** Every actual product merge must be present in the freshly authenticated H. */
export async function authenticateReservedFixes(head, json) {
  assert.match(head ?? "", /^[0-9a-f]{40}$/);
  const authorities = [];
  for (const { pr, merge } of reservedFixes) {
    const fix = await json(`${apiRoot}/pulls/${pr}`);
    assert(Number.isSafeInteger(fix.id) && fix.id > 0);
    assert.equal(fix.number, pr);
    assert.equal(fix.merged, true);
    assert.equal(fix.state, "closed");
    assert.match(fix.merged_at ?? "", /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);
    assert.equal(fix.merge_commit_sha, merge);
    assert.equal(fix.base.ref, "main");
    assert.equal(fix.base.repo.full_name, repository);
    assert.equal(fix.head.repo.full_name, repository);
    const ancestry = await json(`${apiRoot}/compare/${merge}...${head}`);
    assert.equal(ancestry.base_commit.sha, merge);
    assert.equal(ancestry.merge_base_commit.sha, merge);
    assert(["ahead", "identical"].includes(ancestry.status));
    authorities.push({ fix, ancestry });
  }
  assert.equal(authorities.length, 3);
  return authorities;
}
