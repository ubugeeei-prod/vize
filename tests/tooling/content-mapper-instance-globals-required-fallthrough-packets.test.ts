import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

// The component contract gates its required fallthrough props on whether every
// fallthrough member admits `{}` (#12457). Only that one line may change.
const VT = 'import("vize/virtual-ts")';
const C = "__Child_Component_0";
const BEFORE = Buffer.from(` & ${VT}.$RequiredFallthroughProps<${C}>;\n`);
const AFTER = Buffer.from(
  ` & (${VT}.$FallthroughProps<${C}> extends infer __F ? [__F extends unknown ? {} extends __F ? never : __F : never] extends [never] ? ${VT}.$RequiredFallthroughProps<${C}, __F, never> : ${VT}.$RequiredFallthroughProps<${C}, __F> : never);\n`,
);

type Row = number[];
type Directives = { directives: Row[] };

function directivesOf(result: Transform): Directives {
  const value = result.diagnosticDirectives;
  assert.ok(typeof value === "object" && value !== null && "directives" in value);
  assert.ok(Array.isArray(value.directives));
  return { ...value, directives: value.directives };
}

test("the required-fallthrough population changes only the gated contract line", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-0891e5274506.json" },
    files,
    cases,
  )!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-d48ea4e08e0e.json" },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.pullRequest, 12457);
  assert.equal(provenance.producer.sourceRevision, "0891e52745065d99aa4dfbdb03caf4af55a5caf0");
  assert.deepEqual(provenance.previousPopulation, {
    path: "source-packets-current-d48ea4e08e0e.json",
    bytes: Buffer.byteLength(files.get("source-packets-current-d48ea4e08e0e.json")!),
    sha256: sha(files.get("source-packets-current-d48ea4e08e0e.json")!),
  });
  assert.equal(golden.packets.length, 80);
  const changed: string[] = [];
  const growth = AFTER.length - BEFORE.length;
  for (const [index, packet] of golden.packets.entries()) {
    const before = previous.packets[index]!;
    assert.equal(packet.label, before.label);
    assert.equal(packet.sourceSha256, before.sourceSha256);
    assert.equal(packet.globalsSha256, before.globalsSha256);
    if (JSON.stringify(packet.result) === JSON.stringify(before.result)) {
      assert.equal(Buffer.from(before.result.text).indexOf(BEFORE), -1, packet.label);
      continue;
    }
    changed.push(packet.label);
    const old = before.result;
    const oldDirectives = directivesOf(old);
    const text = Buffer.from(old.text);
    const cut = text.indexOf(BEFORE);
    assert.notEqual(cut, -1);
    assert.equal(text.indexOf(BEFORE, cut + 1), -1);
    const end = cut + BEFORE.length;
    const shift = (at: number) => (at >= end ? at + growth : at);
    const mappings = old.mappings.map((row) => {
      assert.ok(row[0]! >= end || row[0]! + row[1]! <= cut, "no mapping covers the contract line");
      return [shift(row[0]!), ...row.slice(1)];
    });
    const directives = oldDirectives.directives.map(([a, b, start, stop, c, d]) => [
      a!,
      b!,
      shift(start!),
      shift(stop!),
      c!,
      d!,
    ]);
    assert.equal(
      JSON.stringify({
        ...old,
        text: Buffer.concat([text.subarray(0, cut), AFTER, text.subarray(end)]).toString(),
        mappings,
        diagnosticDirectives: { ...oldDirectives, directives },
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
