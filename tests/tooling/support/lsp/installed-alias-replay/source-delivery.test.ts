import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { digest } from "./custody.ts";
import { loadSignedSourceDelivery } from "./source-delivery.ts";

// Receipt refusal controls only; these synthetic records never authorize a provider.
const repository = "ubugeeei-prod/vize";
const merge = "a".repeat(40);
const receipt = {
  schema: "vize-signed-source-delivery-v1",
  repository,
  pullRequest: {
    number: 8262,
    html_url: `https://github.com/${repository}/pull/8262`,
    state: "closed",
    merged: true,
    merged_at: "2026-10-08T10:00:00Z",
    base: { ref: "main", repo: { full_name: repository } },
    head: { sha: "b".repeat(40) },
    merge_commit_sha: merge,
  },
  commit: {
    sha: merge,
    html_url: `https://github.com/${repository}/commit/${merge}`,
    commit: { verification: { verified: true, reason: "valid" } },
    parents: [{ sha: "c".repeat(40) }],
  },
};

test("source delivery binds both complete GitHub records to the delivered repository", () => {
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "source-delivery-control-")),
  );
  const file = path.join(directory, "receipt.json");
  const load = (record: typeof receipt) => {
    const bytes = JSON.stringify(record);
    fs.writeFileSync(file, bytes);
    return loadSignedSourceDelivery(file, digest(bytes));
  };
  try {
    assert.equal(load(receipt), merge);
    const foreignBase = structuredClone(receipt);
    foreignBase.pullRequest.base.repo.full_name = "foreign/vize";
    const foreignPull = structuredClone(receipt);
    foreignPull.pullRequest.html_url = "https://github.com/foreign/vize/pull/8262";
    const foreignCommit = structuredClone(receipt);
    foreignCommit.commit.html_url = `https://github.com/foreign/vize/commit/${merge}`;
    for (const record of [foreignBase, foreignPull, foreignCommit]) {
      assert.throws(() => load(record), /delivered repository/);
    }
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});
