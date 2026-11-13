import assert from "node:assert/strict";
import { test } from "node:test";

import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { currentPacketChanges } from "./support/content-mapper-instance-globals-current-provenance.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

type Directives = { directives: number[][]; unusedExpectDirectiveDiagnostics: unknown[] };
type Result = Transform & { diagnosticDirectives: Directives };

const previousName = "source-packets-current-a0fe19e9a57e.json";
const observedName = "source-packets-current-b04abc20ad7e.json";
const anchor = Buffer.from(" readonly __vizeHasFallthroughProps: true;");
const member =
  /^ readonly __vizeFallthroughSource\?: (?:Partial<\$NativeElement<"div">>|Record<string, unknown>);$/u;
const whitespaceDirectiveLabels = ["lf/string-comment-role", "crlf-unicode/string-comment-role"];

/** Rebuild the predecessor packet with one inserted member, in UTF-8 generated offsets. */
function insertMember(before: Result, after: Result, label: string) {
  const old = Buffer.from(before.text),
    next = Buffer.from(after.text);
  let at = 0;
  while (at < old.length && old[at] === next[at]) at += 1;
  const width = next.length - old.length;
  assert.ok(width > 0, `${label} grows by the inserted member`);
  assert.equal(next.subarray(at + width).equals(old.subarray(at)), true, `${label} single insert`);
  // A shared leading space can shift the first difference; anchor on the member itself.
  const start = next.lastIndexOf(" readonly __vizeFallthroughSource?: ", at + width);
  assert.ok(start >= 0 && start <= at, `${label} inserted member start`);
  assert.equal(
    Buffer.concat([next.subarray(0, start), next.subarray(start + width)]).equals(old),
    true,
    `${label} removing the member restores the predecessor text`,
  );
  const inserted = next.subarray(start, start + width).toString();
  assert.match(inserted, member, `${label} inserts only the raw fallthrough source`);
  assert.equal(
    next.subarray(start + width, start + width + anchor.length).equals(anchor),
    true,
    `${label} member precedes __vizeHasFallthroughProps`,
  );
  const shift = (offset: number) => (offset >= start ? offset + width : offset);
  const expected = structuredClone(before);
  expected.text = after.text;
  expected.mappings = before.mappings.map(([offset, ...rest]) => [shift(offset!), ...rest]);
  expected.diagnosticDirectives.directives = before.diagnosticDirectives.directives.map(
    ([a, b, from, to, ...rest]) => [a!, b!, shift(from!), shift(to!), ...rest],
  );
  return { expected, old };
}

test("the fallthrough-source population inserts one member and drops only whitespace ignores", () => {
  const { authority, files, cases } = readGlobalsFixture();
  assert.equal(authority.currentSourcePackets, observedName);
  const observed = selectPacketGoldens(authority, files, cases)!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(observed.provenance as string)!);
  assert.equal(provenance.pullRequest, 12951);
  assert.deepEqual(provenance.sourceChanges, [
    { pullRequest: 12848, commit: "da8fa24faf2ca6bfdbc7157a339df8b1e33a383e" },
    { pullRequest: 12863, commit: "99bf1bc7a48092f42844c7cac3b6540bfa16d494" },
  ]);
  assert.deepEqual(provenance.producer, observed.producer);
  assert.equal(provenance.producer.sourceRevision, "b04abc20ad7e6944e9951417d492f786b7782ebd");
  assert.equal(provenance.producer.sourceTree, "820816a945ff11eddebedd32db63f737b508c43c");
  assert.equal(provenance.producer.pullRequestHead, "558eaa1dcc10886035238dba76ee13f9d677ff78");
  for (const [key, name] of [
    ["previousPopulation", previousName],
    ["successorPopulation", observedName],
  ] as const)
    assert.deepEqual(provenance[key], {
      path: name,
      bytes: Buffer.byteLength(files.get(name)!),
      sha256: sha(files.get(name)!),
    });

  const changed: string[] = [],
    droppedWhitespace: string[] = [];
  for (const [index, packet] of observed.packets.entries()) {
    const oldPacket = previous.packets[index]!;
    assert.equal(packet.label, oldPacket.label);
    assert.equal(packet.sourceSha256, oldPacket.sourceSha256);
    assert.equal(packet.globalsSha256, oldPacket.globalsSha256);
    if (JSON.stringify(packet.result) === JSON.stringify(oldPacket.result)) continue;
    changed.push(packet.label);
    const before = oldPacket.result as Result,
      after = packet.result as Result;
    const { expected, old } = insertMember(before, after, packet.label);
    const kept = new Set(after.diagnosticDirectives.directives.map((row) => JSON.stringify(row)));
    const dropped = expected.diagnosticDirectives.directives.filter(
      (row) => !kept.has(JSON.stringify(row)),
    );
    if (dropped.length) {
      assert.equal(dropped.length, 1, `${packet.label} drops one directive`);
      const [, , from, to] =
        before.diagnosticDirectives.directives[
          expected.diagnosticDirectives.directives.indexOf(dropped[0]!)
        ]!;
      assert.match(
        old.subarray(from!, to!).toString(),
        /^\s+$/u,
        `${packet.label} drops an ignore over whitespace only`,
      );
      expected.diagnosticDirectives.directives = expected.diagnosticDirectives.directives.filter(
        (row) => row !== dropped[0],
      );
      droppedWhitespace.push(packet.label);
    }
    assert.equal(
      JSON.stringify(after),
      JSON.stringify(expected),
      `${packet.label}: diagnostics, mapping spans and all other fields stay exact`,
    );
  }
  assert.deepEqual(changed, currentPacketChanges(observed.packets.map(({ label }) => label)));
  assert.equal(changed.length, 64);
  assert.deepEqual(droppedWhitespace, whitespaceDirectiveLabels);
});
