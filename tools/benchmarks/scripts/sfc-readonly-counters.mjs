import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { memberCases } from "./sfc-member-fixtures.mjs";
export function validateCounters(packet, optimized) {
  const vectors = memberCases();
  assert.deepEqual(
    packet.rows.map(({ id }) => id),
    vectors.map(({ id }) => id),
  );
  for (const [index, vector] of vectors.entries()) {
    const row = packet.rows[index];
    assert.equal(row.source, vector.source);
    const expected =
      vector.shape === "ascii" || (optimized && vector.shape === "readonly") ? 0 : vector.count;
    assert.equal(row.coldProfiles.length, 30);
    for (const profile of row.coldProfiles)
      assert.equal(profile.literalMemberParseCalls, expected, row.id);
  }
}
export function validateComparison(packets) {
  assert.deepEqual(
    packets.map(({ identity }) => identity.side),
    ["base", "head", "head", "base"],
  );
  for (const packet of packets) validateCounters(packet, packet.identity.side === "head");
  // This uses the actual retained baseline observations, never a synthetic packet.
  for (const packet of packets.filter(({ identity }) => identity.side === "base"))
    assert.throws(() => validateCounters(packet, true), /readonly-/);
  return {
    actualBaselineRejectedByCandidateContract: true,
    readonlySizes: [32, 128, 512],
    baselineCalls: [32, 128, 512],
    candidateCalls: [0, 0, 0],
  };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const directory = process.argv[2];
  const packets = ["0-base.json", "1-head.json", "2-head.json", "3-base.json"].map((name) =>
    JSON.parse(fs.readFileSync(path.join(directory, name), "utf8")),
  );
  const result = validateComparison(packets);
  fs.writeFileSync(
    path.join(directory, "readonly-counter-qualification.json"),
    JSON.stringify(result, null, 2) + "\n",
  );
  console.log(JSON.stringify(result));
}
