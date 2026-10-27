import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  closedSetupReference,
  type ExposurePacket,
} from "./support/script-setup-default-expose.ts";

const fixture = JSON.parse(
  readFileSync(
    new URL("./fixtures/script-setup-default-expose-packets.json", import.meta.url),
    "utf8",
  ),
) as {
  packets: { source: string; before: ExposurePacket; after: ExposurePacket }[];
};

test("the reviewed exposure edit preserves complete captured code, maps and descriptors", () => {
  assert.equal(fixture.packets.length, 3);
  for (const row of fixture.packets) {
    const untouched = structuredClone(row.before);
    assert.deepEqual(closedSetupReference(row.before, row.source), row.after);
    assert.deepEqual(row.before, untouched, "historical packets remain immutable");
  }
});

test("the full reference rejects unrelated code, map, descriptor and diagnostic mutations", () => {
  for (const row of fixture.packets) {
    const reference = closedSetupReference(row.before, row.source);
    for (const change of [
      (packet: ExposurePacket) => {
        packet.code += "\nvoid 1;";
      },
      (packet: ExposurePacket) => {
        packet.map = String(packet.map).replace('"mappings":"', '"mappings":"A');
      },
      (packet: ExposurePacket) => {
        packet.moduleShape!.defaultExportEnd += 1;
      },
      (packet: ExposurePacket) => {
        packet.errors = [{ code: "unexpected" }];
      },
      (packet: ExposurePacket) => {
        packet.macroArtifacts = [{ kind: "forged" }];
      },
    ]) {
      const changed = structuredClone(row.after);
      change(changed);
      assert.throws(() => assert.deepEqual(changed, reference), { code: "ERR_ASSERTION" });
    }
  }
});

test("ordinary sources and explicit exposure are not advanced again", () => {
  const before = fixture.packets[1].before;
  assert.deepEqual(closedSetupReference(before, "<script>export default {}</script>"), before);
  const after = fixture.packets[1].after;
  assert.deepEqual(closedSetupReference(after, fixture.packets[1].source), after);
});
