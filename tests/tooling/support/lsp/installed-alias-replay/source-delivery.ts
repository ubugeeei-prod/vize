import assert from "node:assert/strict";
import fs from "node:fs";
import { exactFile } from "./custody.ts";

/** Reviewed raw GitHub PR/commit receipt; local branch state cannot substitute delivery. */
export function loadSignedSourceDelivery(receiptPath: string, receiptSha256: string): string {
  exactFile(receiptPath, receiptSha256);
  const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  assert.equal(receipt.schema, "vize-signed-source-delivery-v1");
  assert.equal(receipt.repository, "ubugeeei-prod/vize");
  const pr = receipt.pullRequest;
  assert.equal(pr.number, 8262, "the actual original defaults-key producer must be delivered");
  assert.equal(
    pr.base.repo.full_name,
    receipt.repository,
    "PR must target the delivered repository",
  );
  assert.equal(
    pr.html_url,
    `https://github.com/${receipt.repository}/pull/${pr.number}`,
    "PR record must belong to the delivered repository",
  );
  assert.equal(pr.state, "closed");
  assert.equal(pr.merged, true);
  assert.equal(pr.base.ref, "main");
  assert.ok(typeof pr.merged_at === "string" && Number.isFinite(Date.parse(pr.merged_at)));
  assert.match(pr.merge_commit_sha, /^[a-f0-9]{40}$/u);
  assert.equal(receipt.commit.sha, pr.merge_commit_sha);
  assert.equal(
    receipt.commit.html_url,
    `https://github.com/${receipt.repository}/commit/${pr.merge_commit_sha}`,
    "merge commit record must belong to the delivered repository",
  );
  assert.equal(receipt.commit.commit.verification.verified, true);
  assert.equal(receipt.commit.commit.verification.reason, "valid");
  assert.ok(Array.isArray(receipt.commit.parents) && receipt.commit.parents.length > 0);
  return pr.merge_commit_sha;
}
