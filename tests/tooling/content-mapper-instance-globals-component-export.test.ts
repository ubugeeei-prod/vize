import assert from "node:assert/strict";
import { test } from "node:test";

import { readGlobalsFixture } from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

type Row = [number, number, number, number, number, number];
type Result = {
  text: string;
  mappings: Row[];
  diagnostics: unknown;
  extension: string;
  diagnosticDirectives?: { directives: Row[] };
};

const DECLARED = "declare const __vize_component__: ";
const TYPE = "GlobalsComponent";
/** Definition, type definition, implementation, references and highlights. */
const NAVIGATION_TARGET = 248;

/** Unmapped gaps of the generated text, minus explicit directive intervals. */
function unmappedIgnores(result: Result, explicit: Row[]): Row[] {
  const mapped = result.mappings
    .filter((row) => row[1] > 0)
    .map((row) => [row[0], row[0] + row[1]] as const)
    .sort((left, right) => left[0] - right[0]);
  const gaps: Array<[number, number]> = [];
  let cursor = 0;
  for (const [start, end] of mapped) {
    if (cursor < start) gaps.push([cursor, start]);
    cursor = Math.max(cursor, end);
  }
  const length = Buffer.byteLength(result.text);
  if (cursor < length) gaps.push([cursor, length]);
  const covered = explicit
    .filter((row) => row[2] < row[3])
    .map((row) => [row[2], row[3]] as const)
    .sort((left, right) => left[0] - right[0]);
  const ignores: Row[] = [];
  for (const [gapStart, gapEnd] of gaps) {
    let start = gapStart;
    for (const [coverStart, coverEnd] of covered) {
      if (coverEnd <= start || coverStart >= gapEnd) continue;
      if (start < coverStart) ignores.push([0, 0, start, coverStart, 0, 0]);
      start = Math.max(start, coverEnd);
    }
    if (start < gapEnd) ignores.push([0, 0, start, gapEnd, 0, 0]);
  }
  return ignores;
}

const isIgnore = (row: Row) => row[0] === 0 && row[1] === 0 && row[4] === 0 && row[5] === 0;

test("the component-export population names the type and maps the export to the SFC only", () => {
  const { authority, files, cases } = readGlobalsFixture();
  assert.equal(authority.currentSourcePackets, "source-packets-current-c6a6c571b090.json");
  assert.ok(selectPacketGoldens(authority, files, cases));
  const golden = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-7f70c48c66d8.json" },
    files,
    cases,
  )!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-949a579fb06d.json" },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.source.sourceRevision, "7f70c48c66d8e4e36da9f1bcaa138e76051e3b32");
  assert.equal(provenance.sourceLaunch.receipt.sourceRevision, provenance.source.sourceRevision);
  assert.equal(provenance.sourceLaunch.receipt.binarySha256, provenance.source.binarySha256);
  assert.equal(provenance.actualQualification.status, "unqualified");
  assert.deepEqual(provenance.actualQualification.outcomes, provenance.actualOutcomes);
  const outcomes = provenance.actualOutcomes as Array<{ id: string; outcome: string }>;
  assert.equal(outcomes.length, 499);
  assert.deepEqual(
    outcomes.filter((row) => row.outcome !== "pass").map((row) => row.id),
    golden.packets.map((packet) => `whole-packet/${packet.label}`),
  );
  for (const [index, packet] of golden.packets.entries()) {
    const old = previous.packets[index]!.result as unknown as Result;
    const current = packet.result as unknown as Result;
    assert.equal(packet.label, previous.packets[index]!.label);
    assert.deepEqual(current.diagnostics, old.diagnostics, packet.label);
    assert.equal(current.extension, old.extension, packet.label);
    // Text: the component's intersection becomes a named type alias.
    const at = old.text.indexOf(DECLARED);
    const end = old.text.indexOf(";\n", at) + 2;
    assert.equal(
      current.text,
      `${old.text.slice(0, at)}type ${TYPE} = ${old.text.slice(at + DECLARED.length, end)}${DECLARED}${TYPE};\n${old.text.slice(end)}`,
      packet.label,
    );
    // Mappings: one navigation-target row maps the exported name to the SFC's start.
    const name = Buffer.from(current.text).indexOf(
      "declare const Globals: typeof __vize_component__;",
    );
    const added: Row = [
      name + "declare const ".length,
      "Globals".length,
      0,
      0,
      1,
      NAVIGATION_TARGET,
    ];
    assert.ok(
      old.mappings.every((row) => row[0] + row[1] <= Buffer.byteLength(old.text.slice(0, at))),
    );
    assert.deepEqual(current.mappings, [...old.mappings, added], packet.label);
    // Directives: explicit rows stay; ignores follow the one newly mapped name.
    const explicit = (current.diagnosticDirectives?.directives ?? []).filter(
      (row) => !isIgnore(row),
    );
    assert.deepEqual(
      explicit,
      (old.diagnosticDirectives?.directives ?? []).filter((row) => !isIgnore(row)),
      packet.label,
    );
    assert.deepEqual(
      (current.diagnosticDirectives?.directives ?? []).filter(isIgnore),
      unmappedIgnores(current, explicit),
      packet.label,
    );
  }
});
