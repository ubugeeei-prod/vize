import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

// A component usage names the shared `$UsageProps` and `$UsageRequiredFallthrough`
// helpers instead of declaring their inline conditionals (#12615). Only those two
// alias bodies may change; every later generated coordinate moves by their shrink.
const VT = 'import("vize/virtual-ts")';
const C = "__Child_Component_0";
const EDITS = [
  [
    `typeof Child extends { __vizeCheck: infer __F } ? (${VT}.IsAny<__F> extends true ? ${VT}.$ComponentRawProps<typeof Child> : Record<string, unknown>) : ${VT}.$ComponentRawProps<typeof Child>;\n`,
    `${VT}.$UsageProps<typeof Child>;\n`,
  ],
  [
    ` & (${VT}.$FallthroughProps<${C}> extends infer __F ? [__F extends unknown ? {} extends __F ? never : __F : never] extends [never] ? ${VT}.$RequiredFallthroughProps<${C}, __F, never> : ${VT}.$RequiredFallthroughProps<${C}, __F> : never);\n`,
    ` & ${VT}.$UsageRequiredFallthrough<${C}>;\n`,
  ],
].map(([before, after]) => [Buffer.from(before!), Buffer.from(after!)] as const);

type Row = number[];
type Directives = { directives: Row[] };

function directivesOf(result: Transform): Directives {
  const value = result.diagnosticDirectives;
  assert.ok(typeof value === "object" && value !== null && "directives" in value);
  assert.ok(Array.isArray(value.directives));
  return { ...value, directives: value.directives };
}

test("the usage-helper population changes only the two shared alias bodies", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const name = "source-packets-current-c6a6c571b090.json";
  const previousName = "source-packets-current-f29a8ee58999.json";
  // The comment-bytes population (#12655) succeeds this one.
  assert.equal(authority.currentSourcePackets, "source-packets-current-4faf0850f21f.json");
  const golden = selectPacketGoldens({ ...authority, currentSourcePackets: name }, files, cases)!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.pullRequest, 12615);
  assert.equal(provenance.producer.sourceRevision, "c6a6c571b090f86cf3c8bff563ed9cb7d1ea7034");
  assert.deepEqual(provenance.previousPopulation, {
    path: previousName,
    bytes: Buffer.byteLength(files.get(previousName)!),
    sha256: sha(files.get(previousName)!),
  });
  assert.equal(golden.packets.length, 80);
  const changed: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const before = previous.packets[index]!;
    assert.equal(packet.label, before.label);
    assert.equal(packet.sourceSha256, before.sourceSha256);
    assert.equal(packet.globalsSha256, before.globalsSha256);
    const old = before.result;
    const text = Buffer.from(old.text);
    if (JSON.stringify(packet.result) === JSON.stringify(old)) {
      for (const [from] of EDITS) assert.equal(text.indexOf(from), -1, packet.label);
      continue;
    }
    changed.push(packet.label);
    // Each edit's old byte range and shrink, in text order.
    const cuts = EDITS.map(([from, to]) => {
      const cut = text.indexOf(from);
      assert.notEqual(cut, -1, packet.label);
      assert.equal(text.indexOf(from, cut + 1), -1, packet.label);
      return { cut, end: cut + from.length, from, to };
    }).sort((left, right) => left.cut - right.cut);
    const shift = (at: number) => {
      let moved = at;
      for (const { cut, end, from, to } of cuts) {
        assert.ok(at <= cut || at >= end, "no coordinate falls inside an edited alias body");
        if (at >= end) moved -= from.length - to.length;
      }
      return moved;
    };
    const mappings = old.mappings.map((row) => {
      for (const { cut, end } of cuts)
        assert.ok(row[0]! >= end || row[0]! + row[1]! <= cut, "no mapping covers an alias body");
      return [shift(row[0]!), ...row.slice(1)];
    });
    const oldDirectives = directivesOf(old);
    const directives = oldDirectives.directives.map(([a, b, start, stop, c, d]) => [
      a!,
      b!,
      shift(start!),
      shift(stop!),
      c!,
      d!,
    ]);
    const parts: Buffer[] = [];
    let from = 0;
    for (const { cut, end, to } of cuts) {
      parts.push(text.subarray(from, cut), to);
      from = end;
    }
    parts.push(text.subarray(from));
    assert.equal(
      JSON.stringify({
        ...old,
        text: Buffer.concat(parts).toString(),
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
