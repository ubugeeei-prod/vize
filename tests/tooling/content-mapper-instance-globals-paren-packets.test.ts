import assert from "node:assert/strict";
import { test } from "node:test";
import { readGlobalsFixture } from "./support/content-mapper-instance-globals-10617.ts";
import {
  releasedRanges,
  selectPacketGoldens,
  unmappedIgnoreRows,
  type DirectiveRow as Row,
} from "./support/content-mapper-instance-globals-packets-10617.ts";

test("the opening-parenthesis population changes only directive tables", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const select = (name: string) =>
    selectPacketGoldens({ ...authority, currentSourcePackets: name }, files, cases)!;
  const golden = select("source-packets-current-8844827414c2.json");
  const previous = select("source-packets-current-03836b7ea580.json");
  const beforeIgnores = select("source-packets-current-d0f4e1da7022.json");
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.source.sourceRevision, "8844827414c2773fcd2a7505a226621178bea755");
  assert.equal(provenance.sourceLaunch.receipt.sourceRevision, provenance.source.sourceRevision);
  assert.equal(provenance.sourceLaunch.receipt.binarySha256, provenance.source.binarySha256);
  assert.equal(provenance.actualQualification.status, "unqualified");
  assert.deepEqual(provenance.actualQualification.outcomes, provenance.actualOutcomes);
  type Directives = { directives: Row[]; unusedExpectDirectiveDiagnostics: unknown[] };
  const changed: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const split = (result: unknown) =>
      result as { diagnosticDirectives: Directives; text: string; mappings: number[][] };
    const { diagnosticDirectives: current, ...rest } = split(packet.result);
    const { diagnosticDirectives: before, ...oldRest } = split(previous.packets[index]!.result);
    // Texts, mappings, diagnostics and every other field keep their bytes and order.
    assert.equal(JSON.stringify(rest), JSON.stringify(oldRest), packet.label);
    const result = split(packet.result);
    const explicit =
      split(beforeIgnores.packets[index]!.result).diagnosticDirectives?.directives ?? [];
    const ignores = unmappedIgnoreRows(result, explicit, true);
    const rows = [...explicit, ...ignores].sort((a, b) => a[2] - b[2]);
    assert.deepEqual(current.directives, rows, packet.label);
    assert.deepEqual(
      current.unusedExpectDirectiveDiagnostics,
      before.unusedExpectDirectiveDiagnostics,
    );
    if (JSON.stringify(current) === JSON.stringify(before)) continue;
    changed.push(packet.label);
    // Negative control: the #11438 gap rule does not produce this table.
    assert.notDeepEqual(unmappedIgnoreRows(result, explicit, false), ignores, packet.label);
    // Nothing is newly ignored; every released run is the `(` opening a mapped span.
    assert.deepEqual(releasedRanges(current.directives, before.directives), [], packet.label);
    const released = releasedRanges(before.directives, current.directives);
    const text = Buffer.from(result.text);
    const starts = new Set(result.mappings.map((mapping) => mapping[0]));
    for (const [start, end] of released) {
      assert.equal(text[start], 0x28, packet.label);
      assert.match(text.subarray(start, end).toString(), /^[(\s]+$/u, packet.label);
      assert.ok(starts.has(end), packet.label);
    }
    const proof = provenance.wholeComparison.changedResults[changed.length - 1];
    assert.deepEqual([proof.label, proof.releasedOpeningParentheses], [packet.label, released]);
    assert.deepEqual([proof.previousDirectives, proof.currentDirectives], [before, current]);
  }
  const outcomes = provenance.actualOutcomes as Array<{ id: string; outcome: string }>;
  assert.equal(outcomes.length, 398);
  assert.deepEqual(
    outcomes.filter((row) => row.outcome !== "pass").map((row) => row.id),
    changed.map((label) => `whole-packet/${label}`),
  );
  assert.equal(changed.length, 76);
  assert.equal(provenance.wholeComparison.unchangedWholePackets, 4);
});
