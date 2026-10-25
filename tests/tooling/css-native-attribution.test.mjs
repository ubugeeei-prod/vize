import assert from "node:assert/strict";
import { test } from "node:test";
import {
  campaign,
  validateArtifact,
  validateReplayImage,
  attributionOrder,
} from "../../tools/benchmarks/scripts/css-native-attribution.mjs";

await test("attribution admits only the completed original archive without substituting another source or artifact", () => {
  const value = {
    id: campaign.artifact,
    workflow_run: { id: campaign.run, head_sha: campaign.head },
    digest: `sha256:${campaign.digest}`,
    size_in_bytes: campaign.bytes,
    expired: false,
  };
  validateArtifact(value);
  for (const mutate of [
    (row) => {
      row.id++;
    },
    (row) => {
      row.workflow_run.id++;
    },
    (row) => {
      row.workflow_run.head_sha = "a".repeat(40);
    },
    (row) => {
      row.digest = "sha256:" + "b".repeat(64);
    },
    (row) => {
      row.size_in_bytes--;
    },
    (row) => {
      row.expired = true;
    },
  ]) {
    const changed = structuredClone(value);
    mutate(changed);
    assert.throws(() => validateArtifact(changed));
  }
});
await test("replay keeps actual producer source identities and rejects wrong image, token, role or retained-target claims", () => {
  for (const side of ["base", "head"]) {
    const expected = campaign.images[side],
      producer = {
        side,
        source: expected.source,
        binarySha256: expected.sha256,
        token: expected.token,
        targetInitiallyEmpty: true,
      };
    validateReplayImage(side, producer, expected.sha256, expected.token);
    for (const changed of [
      { side: side === "base" ? "head" : "base" },
      { source: "a".repeat(40) },
      { binarySha256: "b".repeat(64) },
      { token: "c".repeat(32) },
      { targetInitiallyEmpty: false },
    ])
      assert.throws(() =>
        validateReplayImage(side, { ...producer, ...changed }, expected.sha256, expected.token),
      );
    assert.throws(() => validateReplayImage(side, producer, "d".repeat(64), expected.token));
    assert.throws(() => validateReplayImage(side, producer, expected.sha256, "e".repeat(32)));
  }
});
await test("each native operation remains isolated and independently follows complete ABBA source order", () => {
  const order = attributionOrder();
  assert.equal(order.length, 8);
  for (const operation of ["authored", "processed"])
    assert.deepEqual(
      order.filter((item) => item.operation === operation).map((item) => item.side),
      ["base", "head", "head", "base"],
    );
  for (let index = 0; index < 4; index++)
    assert.deepEqual(
      order.filter((item) => item.index === index).map((item) => item.operation),
      index % 2 === 0 ? ["authored", "processed"] : ["processed", "authored"],
    );
});
