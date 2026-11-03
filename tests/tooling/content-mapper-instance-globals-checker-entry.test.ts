import assert from "node:assert/strict";
import { test } from "node:test";
import { readGlobalsFixture, sha } from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

const BEFORE = 'import type {} from "vize/virtual-ts";';
const AFTER = 'import type {} from "vize/virtual-ts-checker";';
const SHIFT = Buffer.byteLength(AFTER) - Buffer.byteLength(BEFORE);

type Result = {
  text: string;
  mappings: number[][];
  diagnosticDirectives: { directives: number[][] } & Record<string, unknown>;
  semanticLinks?: Array<{ sourceStart: number; targetStart: number } & Record<string, unknown>>;
} & Record<string, unknown>;

// Undo exactly the #11591 import rename: one line, and an 8-byte move of every
// generated coordinate after it. Nothing else may differ.
function beforeCheckerEntry(result: Result): Result {
  const end = Buffer.from(result.text).indexOf(AFTER) + Buffer.byteLength(AFTER);
  assert.equal(result.text.split(AFTER).length - 1, 1);
  assert.equal(result.text.split(BEFORE).length - 1, 0);
  const back = (offset: number) => (offset >= end ? offset - SHIFT : offset);
  return {
    ...result,
    text: result.text
      .replace(AFTER, BEFORE)
      .replace(
        /@vize-map: (\d+):(\d+)/gu,
        (_, start, stop) => `@vize-map: ${back(+start)}:${back(+stop)}`,
      ),
    mappings: result.mappings.map(([start, ...rest]) => [back(start!), ...rest]),
    diagnosticDirectives: {
      ...result.diagnosticDirectives,
      directives: result.diagnosticDirectives.directives.map(([a, b, start, stop, ...rest]) => [
        a!,
        b!,
        back(start!),
        back(stop!),
        ...rest,
      ]),
    },
    ...(result.semanticLinks
      ? {
          semanticLinks: result.semanticLinks.map((link) => ({
            ...link,
            sourceStart: back(link.sourceStart),
            targetStart: back(link.targetStart),
          })),
        }
      : {}),
  };
}

test("the checker-entry population moves only the import and the coordinates after it", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(authority, files, cases)!;
  assert.equal(authority.currentSourcePackets, "source-packets-current-949a579fb06d.json");
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-03836b7ea580.json" },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.source.sourceRevision, "949a579fb06d3f11fb57e21e82278cb15b42bc88");
  assert.equal(provenance.sourceLaunch.receipt.sourceRevision, provenance.source.sourceRevision);
  assert.equal(provenance.sourceLaunch.receipt.binarySha256, provenance.source.binarySha256);
  assert.equal(
    provenance.previousPopulation.sha256,
    sha(files.get(provenance.previousPopulation.path)!),
  );
  assert.equal(provenance.actualQualification.status, "unqualified");
  assert.equal(provenance.actualQualification.fullPacketQualification, false);
  assert.deepEqual(provenance.actualQualification.outcomes, provenance.actualOutcomes);
  const outcomes = provenance.actualOutcomes as Array<{ id: string; outcome: string }>;
  assert.equal(outcomes.length, 398);
  assert.deepEqual(
    outcomes.filter((row) => row.outcome !== "pass").map((row) => row.id),
    golden.packets.map((packet) => `whole-packet/${packet.label}`),
  );
  assert.equal(provenance.wholeComparison.unchangedWholePackets, 0);
  for (const [index, packet] of golden.packets.entries()) {
    const old = previous.packets[index]!;
    assert.equal(packet.label, old.label);
    assert.equal(packet.sourceSha256, old.sourceSha256);
    assert.notDeepEqual(packet.result, old.result, packet.label);
    assert.equal(
      JSON.stringify(beforeCheckerEntry(packet.result as unknown as Result)),
      JSON.stringify(old.result),
      packet.label,
    );
    const end = Buffer.from(packet.result.text as string).indexOf(AFTER) + Buffer.byteLength(AFTER);
    assert.deepEqual(provenance.wholeComparison.changedResults[index], {
      label: packet.label,
      sourceSha256: packet.sourceSha256,
      importEnd: end,
      generatedShiftBytes: SHIFT,
    });
  }
});

test("the checker-entry inverse refuses any other change", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const packet = selectPacketGoldens(authority, files, cases)!.packets[0]!;
  const result = packet.result as unknown as Result;
  for (const mutated of [
    { ...result, text: result.text.replace(AFTER, BEFORE) },
    { ...result, text: `${result.text}${AFTER}` },
  ])
    assert.throws(() => beforeCheckerEntry(mutated));
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-03836b7ea580.json" },
    files,
    cases,
  )!.packets[0]!.result;
  const shifted = {
    ...result,
    mappings: result.mappings.map(([start, ...rest]) => [start! + 1, ...rest]),
  };
  assert.notEqual(JSON.stringify(beforeCheckerEntry(shifted)), JSON.stringify(previous));
});
