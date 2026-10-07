import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function authenticatePackets(custody, before, after, productSource) {
  assert.equal(custody.schema, "vize.n8n.compiler-before-after");
  assert.equal(custody.version, 1);
  assert.equal(custody.productSource, productSource);
  assert.equal(custody.productionBytesIdentical, true);
  assert.equal(custody.baselineAncestorOfRepairAnchor, true);
  assert.equal(custody.before.sourceRevision, custody.baselineSource);
  assert.match(custody.repairAnchor, /^[a-f0-9]{40}$/);
  for (const [phase, bytes] of [
    ["before", before],
    ["after", after],
  ]) {
    const identity = custody[phase];
    assert.match(identity.sourceRevision, /^[a-f0-9]{40}$/);
    assert.match(identity.sourceTree, /^[a-f0-9]{40}$/);
    const packets = identity.authoredPackets.filter(
      (packet) => packet.packet === "n8n-default-slot-loop.json",
    );
    assert.equal(packets.length, 1, `${phase}: exactly one authenticated authored packet`);
    assert.equal(
      createHash("sha256").update(bytes).digest("hex"),
      packets[0].sha256,
      `${phase}: complete captured packet bytes`,
    );
  }
}
