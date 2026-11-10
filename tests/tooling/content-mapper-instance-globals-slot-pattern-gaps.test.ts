import assert from "node:assert/strict";
import { test } from "node:test";

import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

type Row = [number, number, number, number, number, number];
type Result = Transform & {
  diagnosticDirectives: { directives: Row[]; unusedExpectDirectiveDiagnostics: unknown[] };
};

const FULL_FEATURES = 1048575;
const CURRENT = "source-packets-current-f29a8ee58999.json";
const PREVIOUS = "source-packets-current-b6b4bbb0a16a.json";
const CHANGED = [
  "lf/slot-shadow",
  "lf/mixed-slot-scope",
  "crlf-unicode/slot-shadow",
  "crlf-unicode/mixed-slot-scope",
];

test("CURRENT adds only the exact disjoint slot-pattern gap mappings", () => {
  const { authority, files, cases } = readGlobalsFixture();
  assert.equal(authority.currentSourcePackets, CURRENT);
  const current = selectPacketGoldens(authority, files, cases)!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: PREVIOUS },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(current.provenance as string)!);
  assert.equal(provenance.pullRequest, 12603);
  assert.equal(provenance.producer.sourceRevision, "f29a8ee589997f08d4c72addca1fd4fe1fd69f95");
  assert.deepEqual(provenance.previousPopulation, {
    path: PREVIOUS,
    bytes: Buffer.byteLength(files.get(PREVIOUS)!),
    sha256: sha(files.get(PREVIOUS)!),
  });
  assert.deepEqual(provenance.successorPopulation, {
    path: CURRENT,
    bytes: Buffer.byteLength(files.get(CURRENT)!),
    sha256: sha(files.get(CURRENT)!),
  });
  assert.deepEqual(
    provenance.capturedQualification.gapIds,
    CHANGED.map((label) => `whole-packet/${label}`),
  );

  const observed: string[] = [];
  for (const [index, packet] of current.packets.entries()) {
    const old = previous.packets[index]!;
    assert.equal(packet.label, old.label);
    assert.equal(packet.sourceSha256, old.sourceSha256);
    assert.equal(packet.globalsSha256, old.globalsSha256);
    if (!CHANGED.includes(packet.label)) {
      assert.deepEqual(packet.result, old.result, packet.label);
      continue;
    }
    observed.push(packet.label);
    const before = old.result as Result;
    const after = packet.result as Result;
    const bindingIndex = before.mappings.findIndex(([generated, length, , , kind, features]) => {
      const bytes = Buffer.from(before.text).subarray(generated - 2, generated + length + 2);
      return kind === 0 && features === FULL_FEATURES && bytes.toString() === "{ $ghost }";
    });
    assert.notEqual(bindingIndex, -1, `${packet.label} retained slot binding`);
    const binding = before.mappings[bindingIndex]! as Row;
    const [generated, length, source, sourceLength, kind, features] = binding;
    const opening: Row = [generated - 2, 2, source - 2, 2, kind, features];
    const closing: Row = [generated + length, 2, source + sourceLength, 2, kind, features];
    assert.equal(
      Buffer.from(after.text)
        .subarray(opening[0], opening[0] + 2)
        .toString(),
      "{ ",
    );
    assert.equal(
      Buffer.from(after.text)
        .subarray(closing[0], closing[0] + 2)
        .toString(),
      " }",
    );
    const expectedMappings = before.mappings.toSpliced(bindingIndex, 1, opening, binding, closing);
    const expectedDirectives = before.diagnosticDirectives.directives.map((row) => {
      if (row[3] === generated) return [row[0], row[1], row[2], generated - 3, row[4], row[5]];
      if (row[2] === generated + length)
        return [row[0], row[1], generated + length + 2, row[3], row[4], row[5]];
      return row;
    });
    assert.deepEqual(after, {
      ...before,
      mappings: expectedMappings,
      diagnosticDirectives: {
        ...before.diagnosticDirectives,
        directives: expectedDirectives,
      },
    });
  }
  assert.deepEqual(observed, CHANGED);
});
