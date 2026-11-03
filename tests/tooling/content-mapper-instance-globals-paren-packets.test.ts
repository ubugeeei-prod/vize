import assert from "node:assert/strict";
import { test } from "node:test";
import { readGlobalsFixture } from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

type Row = [number, number, number, number, number, number];
type Directives = { directives: Row[]; unusedExpectDirectiveDiagnostics: unknown[] };
type Result = { diagnosticDirectives?: Directives; text: string; mappings: number[][] };

const isSpace = (byte: number) => [0x20, 0x09, 0x0a, 0x0c, 0x0d].includes(byte);

/** #11657: an unmapped row ending at a mapped span ends before the `(` run opening it. */
function trimmed(result: Result, rows: readonly Row[], explicit: readonly string[]): Row[] {
  const text = Buffer.from(result.text);
  const starts = new Set(result.mappings.filter((row) => row[1]! > 0).map((row) => row[0]));
  return rows.flatMap((row): Row[] => {
    if (explicit.includes(JSON.stringify(row))) return [row];
    let end = row[3];
    for (let at = row[3]; starts.has(row[3]) && at > row[2]; at -= 1) {
      if (text[at - 1] === 0x28) end = at - 1;
      else if (!isSpace(text[at - 1]!)) break;
    }
    return row[2] < end ? [[row[0], row[1], row[2], end, row[4], row[5]]] : [];
  });
}

test("the opening-parenthesis population changes only directive tables", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const select = (name: string) =>
    selectPacketGoldens({ ...authority, currentSourcePackets: name }, files, cases)!;
  const golden = select("source-packets-current-2d46a96fbefd.json");
  const previous = select("source-packets-current-949a579fb06d.json");
  const beforeIgnores = select("source-packets-current-d0f4e1da7022.json");
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  const shifts = JSON.parse(files.get(previous.provenance as string)!).wholeComparison
    .changedResults as Array<{ importEnd: number; generatedShiftBytes: number }>;
  assert.equal(provenance.source.sourceRevision, "2d46a96fbefdd4310e8631e96da3799d3663dd8d");
  assert.equal(provenance.sourceLaunch.receipt.sourceRevision, provenance.source.sourceRevision);
  assert.equal(provenance.sourceLaunch.receipt.binarySha256, provenance.source.binarySha256);
  assert.equal(provenance.actualQualification.status, "unqualified");
  assert.deepEqual(provenance.actualQualification.outcomes, provenance.actualOutcomes);
  const changed: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const result = packet.result as unknown as Result;
    const old = previous.packets[index]!.result as unknown as Result;
    const { diagnosticDirectives: current, ...rest } = result;
    const { diagnosticDirectives: before, ...oldRest } = old;
    // Texts, mappings, diagnostics and every other field keep their bytes and order.
    assert.equal(JSON.stringify(rest), JSON.stringify(oldRest), packet.label);
    // Explicit rows (before #11438, moved by #11627's import) are unchanged.
    const { importEnd, generatedShiftBytes } = shifts[index]!;
    const shift = (at: number) => (at >= importEnd ? at + generatedShiftBytes : at);
    const old0 = beforeIgnores.packets[index]!.result as unknown as Result;
    const explicit = (old0.diagnosticDirectives?.directives ?? []).map((row) =>
      JSON.stringify([row[0], row[1], shift(row[2]), shift(row[3]), row[4], row[5]]),
    );
    for (const row of explicit)
      assert.ok(
        current!.directives.some((kept) => JSON.stringify(kept) === row),
        packet.label,
      );
    const expected = trimmed(result, before!.directives, explicit);
    assert.deepEqual(current!.directives, expected, packet.label);
    assert.deepEqual(
      current!.unusedExpectDirectiveDiagnostics,
      before!.unusedExpectDirectiveDiagnostics,
    );
    if (JSON.stringify(current) === JSON.stringify(before)) continue;
    changed.push(packet.label);
    // Negative control: the #11438 table is not the current one.
    assert.notDeepEqual(before!.directives, current!.directives, packet.label);
    const proof = provenance.wholeComparison.changedResults[changed.length - 1];
    assert.deepEqual([proof.label, proof.previousDirectives], [packet.label, before]);
    assert.deepEqual(proof.currentDirectives, current);
    const text = Buffer.from(result.text);
    for (const [start, end] of proof.releasedOpeningParentheses as Array<[number, number]>) {
      assert.equal(text[start], 0x28, packet.label);
      assert.ok([...text.subarray(start, end)].every((byte) => byte === 0x28 || isSpace(byte)));
      assert.ok(result.mappings.some((mapping) => mapping[0] === end && mapping[1]! > 0));
    }
  }
  const outcomes = provenance.actualOutcomes as Array<{ id: string; outcome: string }>;
  assert.equal(outcomes.length, 398);
  assert.deepEqual(
    outcomes.filter((row) => row.outcome !== "pass").map((row) => row.id),
    changed.map((label) => `whole-packet/${label}`),
  );
  assert.equal(changed.length, 76);
  assert.equal(provenance.wholeComparison.unchangedWholePackets, 80 - 76);
});
