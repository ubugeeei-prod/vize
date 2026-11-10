import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

// The component-props pass no longer replays a `v-slot` scope whose body
// checks no component (#12479).
const REPLAY = "\n  // Component props in v-slot scope: #default\n";
const CLOSE = "  };\n";

type Row = number[];
type Result = Transform & { diagnosticDirectives: { directives: Row[] } };

test("the slot-replay population drops only the unchecked replay from the slot packets", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-cd10508733d6.json" },
    files,
    cases,
  )!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-0891e5274506.json" },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.pullRequest, 12486);
  assert.equal(provenance.producer.sourceRevision, "cd10508733d6328e481bea39abad775855d7c7b7");
  assert.deepEqual(provenance.previousPopulation, {
    path: "source-packets-current-0891e5274506.json",
    bytes: Buffer.byteLength(files.get("source-packets-current-0891e5274506.json")!),
    sha256: sha(files.get("source-packets-current-0891e5274506.json")!),
  });
  assert.equal(golden.packets.length, 80);
  const changed: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const before = previous.packets[index]!;
    assert.equal(packet.label, before.label);
    assert.equal(packet.sourceSha256, before.sourceSha256);
    assert.equal(packet.globalsSha256, before.globalsSha256);
    if (JSON.stringify(packet.result) === JSON.stringify(before.result)) {
      assert.equal(before.result.text.includes(REPLAY), false, packet.label);
      continue;
    }
    changed.push(packet.label);
    const old = before.result as Result;
    const text = Buffer.from(old.text);
    const cut = text.indexOf(REPLAY);
    assert.notEqual(cut, -1);
    assert.equal(text.indexOf(REPLAY, cut + 1), -1);
    const end = text.indexOf(CLOSE, cut) + CLOSE.length;
    assert.equal(end - cut, 214);
    const shift = (at: number) => (at >= end ? at - (end - cut) : at);
    // The replay held one mapped span, its pattern binding; the unmapped-gap
    // ignore rows on either side of it become one.
    const dropped = old.mappings.filter((row) => row[0]! >= cut && row[0]! < end);
    assert.equal(dropped.length, 1);
    const [binding] = dropped;
    assert.equal(text.subarray(binding![0]!, binding![0]! + binding![1]!).toString(), "$ghost");
    const mappings = old.mappings
      .filter((row) => row !== binding)
      .map((row) => {
        assert.ok(row[0]! >= end || row[0]! + row[1]! <= cut, "no other mapping is inside");
        return [shift(row[0]!), ...row.slice(1)];
      });
    const rows = old.diagnosticDirectives.directives;
    const left = rows.findIndex((row) => row[3] === binding![0]);
    const right = rows.findIndex((row) => row[2] === binding![0]! + binding![1]!);
    assert.equal(right, left + 1);
    assert.ok(rows[left]![2]! < cut && rows[right]![3]! > end);
    const directives = rows.flatMap((row, at) => {
      if (at === left) return [[...row.slice(0, 3), shift(rows[right]![3]!), ...row.slice(4)]];
      if (at === right) return [];
      assert.ok(row[3]! <= cut || row[2]! >= end, "no other directive touches the replay");
      return [[row[0]!, row[1]!, shift(row[2]!), shift(row[3]!), row[4]!, row[5]!]];
    });
    assert.equal(
      JSON.stringify({
        ...old,
        text: Buffer.concat([text.subarray(0, cut), text.subarray(end)]).toString(),
        mappings,
        diagnosticDirectives: { ...old.diagnosticDirectives, directives },
      }),
      JSON.stringify(packet.result),
      "every diagnostic, mapping and opaque field is unchanged apart from the shifted coordinates",
    );
  }
  assert.deepEqual(changed, [
    "lf/slot-shadow",
    "lf/mixed-slot-scope",
    "crlf-unicode/slot-shadow",
    "crlf-unicode/mixed-slot-scope",
  ]);
});
