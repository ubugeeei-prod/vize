import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertWholeTransform,
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

const NAME = "source-packets-current-245bb0b6f1cb.json";
const BEFORE =
  "type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<$ComponentInstance, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, {}> & $ComponentInputGuard<Props, {}, $AuthoredProps>>;\n";
const OWN =
  "type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<{\n  $props: Props;\n  readonly __vizeRawProps?: Props;\n  $emit: $PublicEmit<Emits>;\n  $slots: $PublicSlots<$Slots>;\n}, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, {}> & $ComponentInputGuard<Props, {}, $AuthoredProps>> & ";
const AUTHORED = "Omit<$AuthoredInstance, '$props' | '$emit' | '$slots'> & ";

type Result = Transform & {
  diagnosticDirectives: { directives: number[][]; unusedExpectDirectiveDiagnostics: unknown[] };
};

// Review these frozen populations; the live comparator receives complete result
// objects unchanged. No candidate output passes through this finite comparison.
function reviewedResult(old: Result, authored: boolean): Result {
  assert.equal(old.text.split(BEFORE).length, 2, "one complete historical constructor");
  const after = `${OWN}${authored ? AUTHORED : ""}$ComponentPublicBase;\n`;
  const bytes = Buffer.from(old.text);
  const end = bytes.indexOf(BEFORE) + Buffer.byteLength(BEFORE);
  const delta = Buffer.byteLength(after) - Buffer.byteLength(BEFORE);
  const move = (offset: number) => (offset >= end ? offset + delta : offset);
  const atoms: number[][] = [];
  const mappings = old.mappings.flatMap((row, index) => {
    const [start, length, source, authoredLength] = row as [number, number, number, number];
    const stop = start + length;
    const previous = old.mappings[index - 1];
    const next = old.mappings[index + 1];
    const open = bytes[start - 1];
    const wrapped =
      ((open === 34 || open === 39) && bytes[stop] === open) || (open === 40 && bytes[stop] === 41);
    const available =
      wrapped &&
      (previous ? previous[0]! + previous[1]! : 0) <= start - 1 &&
      stop < (next?.[0] ?? bytes.length);
    const current = [move(start), ...row.slice(1)];
    if (!available) return [current];
    const left = [move(start - 1), 1, source, authoredLength, 1, 0];
    const right = [move(stop), 1, source, authoredLength, 1, 0];
    atoms.push(left, right);
    return [left, current, right];
  });
  const directives = old.diagnosticDirectives.directives.flatMap(
    ([a, b, start, stop, kind, extra]) => {
      let intervals = [[move(start!), move(stop!)]];
      if (a === 0 && b === 0 && kind === 0 && extra === 0)
        for (const [at] of atoms)
          intervals = intervals.flatMap(([s, t]) =>
            [
              [s!, Math.min(t!, at!)],
              [Math.max(s!, at! + 1), t!],
            ].filter(([x, y]) => x! < y!),
          );
      return intervals.map(([s, t]) => [a!, b!, s!, t!, kind!, extra!]);
    },
  );
  return {
    ...old,
    text: old.text.replace(BEFORE, after),
    mappings,
    diagnosticDirectives: { ...old.diagnosticDirectives, directives },
  };
}

function population() {
  const fixture = readGlobalsFixture();
  const select = (name: string) =>
    selectPacketGoldens(
      { ...fixture.authority, currentSourcePackets: name },
      fixture.files,
      fixture.cases,
    )!;
  const current = select(NAME);
  const previous = select("source-packets-current-2d46a96fbefd.json");
  return {
    ...fixture,
    current,
    previous,
    provenance: JSON.parse(fixture.files.get(current.provenance as string)!),
  };
}

test("the source-captured constructor population retains every whole packet and authored mapping", () => {
  const { files, cases, current, previous, provenance } = population();
  assert.equal(provenance.source.sourceRevision, "245bb0b6f1cb0dd618268aa48cbe9c1d2e01c3cc");
  assert.equal(provenance.source.sourceTree, "8d65578c46f76b805b672a3e188501963f242d1a");
  assert.equal(provenance.source.pullRequestHead, "45ee61bd932ca84657d90326f2dee4c1aa70fe55");
  assert.deepEqual(current.producer, provenance.source);
  assert.equal(provenance.sourceLaunch.receipt.sourceRevision, provenance.source.sourceRevision);
  assert.equal(provenance.sourceLaunch.receipt.binarySha256, provenance.source.binarySha256);
  assert.deepEqual(provenance.sourceCause.producers, [
    {
      path: "crates/vize_canon/src/virtual_ts/generator/component_constructors.rs",
      sha256: "3f5eec2c812921ff2bf555bdde2315fe3871f3a8af9fbba2ed488f54acb02138",
    },
    {
      path: "crates/vize_canon/src/batch/virtual_project/content_mapper.rs",
      sha256: "25abafce233364f0b11f71202d32f87290ef7d231f939063938f8649e3448b67",
    },
  ]);
  assert.equal(provenance.artifact.id, 13398636603);
  assert.equal(
    provenance.artifact.sha256,
    "75e7c6f2392cb1750b7846c0fa912871acd07eaa7aff9c1f985436a937b3ee8d",
  );
  assert.equal(
    provenance.previousPopulation.sha256,
    sha(files.get(provenance.previousPopulation.path)!),
  );
  assert.equal(current.packets.length, 80);
  assert.equal(provenance.actualInputs.length, 80);
  const groups = new Map<number, number>();
  for (const [index, packet] of current.packets.entries()) {
    const old = previous.packets[index]!;
    const source =
      index < 40
        ? cases[index]!.source
        : cases[index % 40]!.source.replace("{{ ", '{{ "日本語🍣" + ').replaceAll("\n", "\r\n");
    assert.deepEqual({ ...packet, result: old.result }, old);
    assert.equal(packet.sourceSha256, sha(source));
    assert.equal(packet.globalsSha256, sha(cases[index % 40]!.globals));
    const input = provenance.actualInputs[index];
    assert.equal(input.label, packet.label);
    assert.equal(input.bytes, Buffer.byteLength(source));
    assert.equal(input.sha256, packet.sourceSha256);
    assert.equal(input.globalsSha256, packet.globalsSha256);
    assert.equal(input.responseId, index + 3);
    assert.equal(input.resultSha256, sha(JSON.stringify(packet.result)));
    const expected = reviewedResult(
      old.result as Result,
      packet.label.endsWith("/options-own-method"),
    );
    assertWholeTransform(packet.result, expected);
    assert.notDeepEqual(packet.result, old.result);
    const added = packet.result.mappings.filter(
      (row) =>
        !old.result.mappings.some((before) => JSON.stringify(before) === JSON.stringify(row)),
    );
    groups.set(added.length, (groups.get(added.length) ?? 0) + 1);
    assert.deepEqual(provenance.wholeComparison.changedResults[index].addedDiagnosticAtoms, added);
  }
  assert.deepEqual(Object.fromEntries(groups), { 0: 48, 2: 28, 4: 4 });
  const outcomes = provenance.actualOutcomes as Array<{ id: string; outcome: string }>;
  assert.equal(outcomes.length, 398);
  assert.equal(outcomes.filter(({ outcome }) => outcome === "pass").length, 318);
  assert.deepEqual(
    outcomes.filter(({ outcome }) => outcome !== "pass").map(({ id }) => id),
    current.packets.map(({ label }) => `whole-packet/${label}`),
  );
  assert.equal(provenance.actualQualification.status, "unqualified");
  assert.deepEqual(provenance.limits, {
    fullPacketQualification: false,
    hostMatrixQualification: false,
    publicAcceptance: false,
    timingClaim: false,
    newNativeExecution: false,
  });
});

test("the complete constructor packet review refuses unrelated text, mapping, directive and opaque changes", () => {
  const { current, previous } = population();
  const actual = current.packets[0]!.result as Result;
  const expected = reviewedResult(previous.packets[0]!.result as Result, false);
  const mutations = [
    { ...actual, text: `${actual.text} ` },
    { ...actual, text: actual.text.replace("$ComponentPublicBase;", "unknown;") },
    { ...actual, mappings: actual.mappings.slice(1) },
    {
      ...actual,
      mappings: actual.mappings.map((row, i) => (i === 0 ? [row[0]! + 1, ...row.slice(1)] : row)),
    },
    {
      ...actual,
      diagnosticDirectives: {
        ...actual.diagnosticDirectives,
        directives: actual.diagnosticDirectives.directives.slice(1),
      },
    },
    { ...actual, diagnostics: [{ message: "unexpected" }] },
    { ...actual, foreignOpaqueField: true },
  ];
  for (const mutation of mutations) assert.throws(() => assertWholeTransform(mutation, expected));
  const atom = actual.mappings.findIndex((row) => row[5] === 0);
  assert.ok(atom >= 0);
  for (let field = 0; field < 6; field++) {
    const mutation = structuredClone(actual);
    mutation.mappings[atom]![field]! += 1;
    assert.throws(() => assertWholeTransform(mutation, expected));
  }
});
