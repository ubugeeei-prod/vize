import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

// The template context of a component whose only template global is a resolved
// CSS module no longer declares the three `__Global` helpers (#12460).
const HELPER = Buffer.from(
  "    type $GlobalContextIsAny<T> = 0 extends (1 & T) ? true : false;\n    type $GlobalContextProperty<T, F> = $GlobalContextIsAny<T> extends true ? T : unknown extends T ? F : T;\n    type __Global<K extends string, F = unknown> = K extends keyof __Ctx ? $GlobalContextProperty<__Ctx[K], F> : F;\n",
);

type Row = number[];
type Result = Transform & { diagnosticDirectives: { directives: Row[] } };

test("the template-global population drops only the unused helper from the CSS-module packets", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-d48ea4e08e0e.json" },
    files,
    cases,
  )!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-517b8abf5651.json" },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.pullRequest, 12460);
  assert.equal(provenance.producer.sourceRevision, "d48ea4e08e0ee0b2773d4ba3c696bc1c9235bce1");
  assert.deepEqual(provenance.previousPopulation, {
    path: "source-packets-current-517b8abf5651.json",
    bytes: Buffer.byteLength(files.get("source-packets-current-517b8abf5651.json")!),
    sha256: sha(files.get("source-packets-current-517b8abf5651.json")!),
  });
  assert.equal(golden.packets.length, 80);
  const changed: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const before = previous.packets[index]!;
    assert.equal(packet.label, before.label);
    assert.equal(packet.sourceSha256, before.sourceSha256);
    assert.equal(packet.globalsSha256, before.globalsSha256);
    if (JSON.stringify(packet.result) === JSON.stringify(before.result)) {
      assert.equal(Buffer.from(before.result.text).indexOf(HELPER), -1, packet.label);
      continue;
    }
    changed.push(packet.label);
    const old = before.result as Result;
    const text = Buffer.from(old.text);
    const cut = text.indexOf(HELPER);
    assert.notEqual(cut, -1);
    assert.equal(text.indexOf(HELPER, cut + 1), -1);
    const end = cut + HELPER.length;
    const shift = (at: number) => (at >= end ? at - HELPER.length : at);
    const mappings = old.mappings.map((row) => {
      assert.ok(row[0]! >= end || row[0]! + row[1]! <= cut, "no mapping covers the helper");
      return [shift(row[0]!), ...row.slice(1)];
    });
    const directives = old.diagnosticDirectives.directives.map(([a, b, start, stop, c, d]) => [
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
        text: Buffer.concat([text.subarray(0, cut), text.subarray(end)]).toString(),
        mappings,
        diagnosticDirectives: { ...old.diagnosticDirectives, directives },
      }),
      JSON.stringify(packet.result),
      "every diagnostic, mapping and opaque field is unchanged apart from the shifted coordinates",
    );
  }
  assert.deepEqual(changed, ["lf/css-module", "crlf-unicode/css-module"]);
});
