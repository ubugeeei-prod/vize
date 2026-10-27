import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { producerCandidateRecord } from "../../../tools/support/release/release/producer_identity.ts";
import {
  CRATE_INVENTORY_PATH,
  releaseCrateInventory,
  decodeCrateInventoryFile,
} from "../../../tools/support/release/release/quota/inventory.ts";
import { readProducedCandidate } from "../../../tools/support/release/release/quota/producer.ts";
import { readOwnedCrateSource } from "../../../tools/support/release/release/quota/crate_source.ts";
import { mockProduced, mockProducerResponse, mockCrate } from "./quota-fixtures.ts";
const owner = {
  version: "0.602.0",
  runId: "42",
  runAttempt: 1,
  workflowHead: "b".repeat(40),
  candidateSha: "c".repeat(40),
  producerSource: "d".repeat(40),
  nativeManifestGitBlob: "e".repeat(40),
  inventoryGitBlob: "f".repeat(40),
};
test("actual producer capture requires physical candidate, single parent and committed version", () => {
  const record = mockProduced(owner).proof.record,
    commit = "tree " + "a".repeat(40) + "\nparent " + owner.producerSource + "\n\nrelease\n";
  const manifest = JSON.stringify({ name: "@vizejs/native", version: owner.version });
  assert.deepEqual(producerCandidateRecord(record, owner.candidateSha, commit, manifest), record);
  assert.throws(() => producerCandidateRecord(record, "9".repeat(40), commit, manifest));
  assert.throws(() =>
    producerCandidateRecord(
      record,
      owner.candidateSha,
      commit.replace("\n\n", "\nparent " + "8".repeat(40) + "\n\n"),
      manifest,
    ),
  );
  assert.throws(() =>
    producerCandidateRecord(
      record,
      owner.candidateSha,
      commit,
      manifest.replace("0.602.0", "0.601.0"),
    ),
  );
});
test("source-pinned cohort requires canonical literal, unique names and authentic Git blob", () => {
  const source = 'export const publishedCrates: readonly string[] = ["alpha", "beta"];';
  assert.deepEqual(releaseCrateInventory(source), ["alpha", "beta"]);
  assert.throws(() => releaseCrateInventory(source.replace('"beta"', '"alpha"')));
  assert.throws(() => releaseCrateInventory("export const publishedCrates = compute();"));
  assert.throws(() => releaseCrateInventory(source.replace('"beta"', "...dynamic")));
  assert.throws(() => releaseCrateInventory(source.replace(', "beta"', ' "beta"')));
  const raw = Buffer.from(source),
    sha = createHash("sha1")
      .update("blob " + raw.length + "\0")
      .update(raw)
      .digest("hex");
  const value = {
    path: CRATE_INVENTORY_PATH,
    type: "file",
    encoding: "base64",
    content: raw.toString("base64"),
    sha,
  };
  assert.equal(decodeCrateInventoryFile(value).gitBlob, sha);
  assert.throws(() => decodeCrateInventoryFile({ ...value, sha: "0".repeat(40) }));
  assert.throws(() => decodeCrateInventoryFile({ ...value, path: "other.ts" }));
});
test("tampered ZIP digest and foreign artifact workflow cannot grant ownership", async () => {
  for (const kind of ["digest", "workflow"]) {
    const fetchImpl = (async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(input instanceof Request ? input.url : input),
        response = mockProducerResponse(owner, input, init);
      assert.ok(response);
      if (url.pathname.endsWith("/artifacts/7")) {
        const metadata = mockProduced(owner).metadata;
        if (kind === "digest") metadata.digest = "sha256:" + "0".repeat(64);
        else metadata.workflow_run.head_sha = "8".repeat(40);
        return new Response(JSON.stringify(metadata));
      }
      return response;
    }) as typeof fetch;
    await assert.rejects(
      readProducedCandidate(owner, fetchImpl),
      kind === "digest" ? /digest\/size/ : /workflow binding/,
    );
  }
});
test("public crate full bytes and Cargo VCS must both identify the intended C", async () => {
  const fixture = mockCrate("alpha", owner.version, owner.candidateSha);
  const fetchImpl = (async () => new Response(new Uint8Array(fixture.archive))) as typeof fetch;
  assert.equal(
    (
      await readOwnedCrateSource(
        "alpha",
        owner.version,
        fixture.checksum,
        owner.candidateSha,
        fetchImpl,
      )
    ).candidateSha,
    owner.candidateSha,
  );
  await assert.rejects(
    readOwnedCrateSource("alpha", owner.version, "0".repeat(64), owner.candidateSha, fetchImpl),
    /checksum/,
  );
  await assert.rejects(
    readOwnedCrateSource("alpha", owner.version, fixture.checksum, "9".repeat(40), fetchImpl),
    /VCS/,
  );
});
