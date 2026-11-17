import assert from "node:assert/strict";
import { test } from "node:test";

import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { authoredComponentPacketChanges } from "./support/content-mapper-instance-globals-current-provenance.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

type Directives = { directives: number[][]; unusedExpectDirectiveDiagnostics: unknown[] };
type Result = Transform & { diagnosticDirectives: Directives };

const previousName = "source-packets-current-b04abc20ad7e.json";
const observedName = "source-packets-current-83b16c48aa52.json";
/** The two #13113 edits of the component export, in generated order. */
const edits = [
  [
    "$AuthoredComponent extends object ? {",
    "$AuthoredComponent extends infer A extends object ? {",
  ],
  ["} & $AuthoredComponent & $ComponentConstructor", "} & A & $ComponentConstructor"],
] as const;

/** Apply both edits to the predecessor packet, moving every UTF-8 generated offset after them. */
function bindByInference(before: Result, label: string): Result {
  let text = Buffer.from(before.text);
  const moves: { end: number; width: number }[] = [];
  for (const [from, to] of edits) {
    const needle = Buffer.from(from);
    const at = text.indexOf(needle);
    assert.ok(at >= 0, `${label} has ${from}`);
    assert.equal(text.indexOf(needle, at + 1), -1, `${label} has one ${from}`);
    const next = Buffer.from(to);
    text = Buffer.concat([text.subarray(0, at), next, text.subarray(at + needle.length)]);
    moves.push({ end: at + needle.length, width: next.length - needle.length });
  }
  // Each edit is measured after the previous one; no mapped or directive
  // offset starts inside an edited span.
  const shift = (offset: number) =>
    moves.reduce((value, { end, width }) => (value >= end ? value + width : value), offset);
  const expected = structuredClone(before);
  expected.text = text.toString();
  expected.mappings = before.mappings.map(([offset, ...rest]) => [shift(offset!), ...rest]);
  expected.diagnosticDirectives.directives = before.diagnosticDirectives.directives.map(
    ([a, b, from, to, ...rest]) => [a!, b!, shift(from!), shift(to!), ...rest],
  );
  return expected;
}

test("the authored-component population binds only the Options API export by inference", () => {
  const { authority, files, cases } = readGlobalsFixture();
  assert.equal(authority.currentSourcePackets, observedName);
  const observed = selectPacketGoldens(authority, files, cases)!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(observed.provenance as string)!);
  assert.equal(provenance.pullRequest, 13119);
  assert.deepEqual(provenance.producer, observed.producer);
  assert.equal(provenance.producer.sourceRevision, "83b16c48aa52764bf5fbb0889df44bad8361d12c");
  assert.equal(provenance.producer.sourceTree, "769b062474afeafbb950bb9c2ee16eb8f0c7a0e3");
  assert.equal(provenance.producer.pullRequestHead, "026787445f98a058d741022b1a0c60cfbd1270f1");
  assert.equal(
    provenance.sourceReceipt.path,
    "crates/vize_canon/src/virtual_ts/generator/component_export.rs",
  );
  for (const [key, name] of [
    ["previousPopulation", previousName],
    ["successorPopulation", observedName],
  ] as const)
    assert.deepEqual(provenance[key], {
      path: name,
      bytes: Buffer.byteLength(files.get(name)!),
      sha256: sha(files.get(name)!),
    });

  const changed: string[] = [];
  for (const [index, packet] of observed.packets.entries()) {
    const oldPacket = previous.packets[index]!;
    assert.equal(packet.label, oldPacket.label);
    assert.equal(packet.sourceSha256, oldPacket.sourceSha256);
    assert.equal(packet.globalsSha256, oldPacket.globalsSha256);
    if (JSON.stringify(packet.result) === JSON.stringify(oldPacket.result)) continue;
    changed.push(packet.label);
    const before = oldPacket.result as Result,
      after = packet.result as Result;
    assert.equal(
      Buffer.byteLength(after.text),
      Buffer.byteLength(before.text) - 1,
      `${packet.label} is one byte shorter`,
    );
    assert.equal(
      JSON.stringify(after),
      JSON.stringify(bindByInference(before, packet.label)),
      `${packet.label}: diagnostics, directives, mapping spans and all other fields stay exact`,
    );
  }
  assert.deepEqual(
    changed,
    authoredComponentPacketChanges(observed.packets.map(({ label }) => label)),
  );
  assert.deepEqual(changed, ["lf/options-own-method", "crlf-unicode/options-own-method"]);
});
