import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  closedSetupReference,
  historicalSetupCode,
  type ExposurePacket,
} from "./support/script-setup-default-expose.ts";

const fixture = JSON.parse(
  readFileSync(
    new URL("./fixtures/script-setup-default-expose-packets.json", import.meta.url),
    "utf8",
  ),
) as {
  packets: { source: string; before: ExposurePacket; after: ExposurePacket }[];
  emptySetupModules: {
    packets: { source: string; before: ExposurePacket; after: ExposurePacket }[];
  };
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
        if (typeof packet.map !== "string")
          throw new TypeError("captured native map must be encoded");
        packet.map = packet.map.replace('"mappings":"', '"mappings":"A');
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

test("historical checksum receipts reverse only the exact initial exposure edit", () => {
  for (const row of fixture.packets) {
    assert.equal(historicalSetupCode(row.after.code, row.source), row.before.code);
    assert.throws(() => historicalSetupCode(row.before.code, row.source), {
      code: "ERR_ASSERTION",
    });
    const misplaced = row.after.code.replace("__expose();", "void 0;\n__expose();");
    assert.throws(() => historicalSetupCode(misplaced, row.source), { code: "ERR_ASSERTION" });
  }
});

test("Rust structured source maps retain the same complete coordinate contract", () => {
  for (const row of fixture.packets) {
    const before = { ...row.before, map: JSON.parse(row.before.map as string) };
    const after = { ...row.after, map: JSON.parse(row.after.map as string) };
    assert.deepEqual(closedSetupReference(before, row.source), after);
  }
});

test("empty setup returns preserve the complete three pinned SSR modules", () => {
  assert.equal(fixture.emptySetupModules.packets.length, 3);
  for (const row of fixture.emptySetupModules.packets) {
    assert.deepEqual(closedSetupReference(row.before, row.source), row.after);
    assert.equal(historicalSetupCode(row.after.code, row.source), row.before.code);
  }
});
