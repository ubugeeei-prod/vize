import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  assertTransformEnvelope,
  assertWholeTransform,
  readGlobalsFixture,
  sha,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";
import {
  assertDiagnosticMappingRoster,
  parenthesisDiagnosticMappings,
  withParenthesisDiagnosticMappings,
} from "./support/content-mapper-parenthesis-mappings-11675.ts";

const observedPath = new URL(
  "../_fixtures/tooling/instance-globals-parenthesis-main-11675/observed-packets.json.txt",
  import.meta.url,
);
const observedBytes = fs.readFileSync(observedPath);
assert.equal(
  sha(observedBytes),
  "3cabf02a09908eb7e3f35141df012eef5170d0ad6837abdff38134d0b98ef543",
);
const observed = JSON.parse(observedBytes.toString());
const { authority, files, cases } = readGlobalsFixture();
// This recorded transition owns the historical population as CURRENT advances.
const golden = selectPacketGoldens(
  { ...authority, currentSourcePackets: "source-packets-current-2d46a96fbefd.json" },
  files,
  cases,
)!;

void test("main's genuine LF and CRLF Unicode packets keep complete historical identity plus known atoms", () => {
  for (const row of observed.packets) {
    const old = golden.packets.find((packet) => packet.label === row.label)!;
    assert.equal(row.sourceSha256, old.sourceSha256);
    assert.equal(row.globalsSha256, old.globalsSha256);
    assertTransformEnvelope(row.response, row.response.id);
    assertWholeTransform(row.response.result, withParenthesisDiagnosticMappings(old.result));
  }
});

void test("the expected-only transition has an exact inverse over all eighty historical packets", () => {
  let changed = 0;
  let pairs = 0;
  for (const packet of golden.packets) {
    const old = packet.result;
    const identity = JSON.stringify(old);
    const atoms = parenthesisDiagnosticMappings(old);
    const current = withParenthesisDiagnosticMappings(old);
    const keys = new Set(atoms.map((row) => JSON.stringify(row)));
    assert.deepEqual(
      current.mappings.filter((row) => !keys.has(JSON.stringify(row))),
      old.mappings,
    );
    const { mappings: _oldMap, diagnosticDirectives: oldTable, ...oldOpaque } = old;
    const { mappings: _newMap, diagnosticDirectives: newTable, ...newOpaque } = current;
    assert.deepEqual(newOpaque, oldOpaque, "text, diagnostics and all opaque fields stay exact");
    const before = oldTable as {
      directives: number[][];
      unusedExpectDirectiveDiagnostics: unknown;
    };
    const after = newTable as typeof before;
    assert.deepEqual(
      after.unusedExpectDirectiveDiagnostics,
      before.unusedExpectDirectiveDiagnostics,
    );
    const authored = (rows: number[][]) => rows.filter((row) => row[0] !== 0 || row[1] !== 0);
    assert.deepEqual(authored(after.directives), authored(before.directives));
    const gaps = before.directives.filter((row) => row[0] === 0 && row[1] === 0);
    const restored = [
      ...after.directives
        .filter((row) => row[0] === 0 && row[1] === 0)
        .map((row) => [row[2], row[3]]),
      ...atoms
        .filter(([at]) => gaps.some((row) => row[2] <= at && at < row[3]))
        .map(([at]) => [at, at + 1]),
    ].sort((left, right) => left[0] - right[0]);
    const merged: number[][] = [];
    for (const range of restored) {
      const previous = merged.at(-1);
      if (previous && range[0] <= previous[1]) previous[1] = Math.max(previous[1], range[1]);
      else merged.push([...range]);
    }
    assert.deepEqual(
      merged,
      gaps.map((row) => [row[2], row[3]]),
      "only the new mapped bytes leave old ignore gaps",
    );
    assert.equal(JSON.stringify(old), identity, "historical packet is never mutated");
    if (atoms.length) changed++;
    pairs += atoms.length / 2;
  }
  assert.equal(changed, 32);
  assert.equal(pairs, 36);
});

void test("the shared late roster entry accepts genuine prior AST facts without substituting a compiler", () => {
  for (const row of observed.packets) {
    const old = golden.packets.find((packet) => packet.label === row.label)!;
    assert.equal(row.response.result.text, old.result.text);
    assertDiagnosticMappingRoster(
      row.response.result,
      old.result.mappings.filter((span) => span[5] === 0),
    );
  }
  assertDiagnosticMappingRoster(observed.symbolGeometry.response.result, [
    observed.symbolGeometry.expectedAstPrefixFromOriginalFailure,
  ]);
});

void test("all actual atom fields, old mapping fields and opaque payload mutations reach the real comparator", () => {
  for (const row of observed.packets) {
    const old = golden.packets.find((packet) => packet.label === row.label)!;
    const current = row.response.result;
    const expected = withParenthesisDiagnosticMappings(old.result);
    assertWholeTransform(current, expected);
    for (let index = 0; index < current.mappings.length; index++) {
      for (let field = 0; field < 6; field++) {
        const copy = structuredClone(current);
        copy.mappings[index][field]++;
        assert.throws(() => assertWholeTransform(copy, expected), /complete generated text/u);
      }
    }
    const mutations = [
      (copy: typeof current) => copy.mappings.pop(),
      (copy: typeof current) => copy.mappings.reverse(),
      (copy: typeof current) => copy.mappings.push([...copy.mappings[0]]),
      (copy: typeof current) => {
        copy.unknownProducerField = true;
      },
      (copy: typeof current) => {
        copy.diagnosticDirectives.unknownField = true;
      },
      (copy: typeof current) => {
        copy.diagnosticDirectives.directives[0][2]++;
      },
      (copy: typeof current) => {
        copy.diagnostics.push({ message: "unproved" });
      },
      (copy: typeof current) => {
        copy.text += "\n";
      },
    ];
    for (const mutate of mutations) {
      const copy = structuredClone(current);
      mutate(copy);
      assert.throws(() => assertWholeTransform(copy, expected), /complete generated text/u);
    }
  }
});

void test("the real late roster rejects missing, duplicated and mutated delimiter identities", () => {
  const row = observed.packets[0];
  const old = golden.packets.find((packet) => packet.label === row.label)!;
  const prefixes = old.result.mappings.filter((span) => span[5] === 0);
  const atoms = parenthesisDiagnosticMappings(old.result);
  const index = row.response.result.mappings.findIndex((span: number[]) => span[0] === atoms[0][0]);
  for (let field = 0; field < 6; field++) {
    const copy = structuredClone(row.response.result);
    copy.mappings[index][field]++;
    assert.throws(() => assertDiagnosticMappingRoster(copy, prefixes), /diagnostic-only roster/u);
  }
  for (const mutate of [
    (copy: typeof row.response.result) => copy.mappings.splice(index, 1),
    (copy: typeof row.response.result) => copy.mappings.push([...copy.mappings[index]]),
  ]) {
    const copy = structuredClone(row.response.result);
    mutate(copy);
    assert.throws(() => assertDiagnosticMappingRoster(copy, prefixes), /diagnostic-only roster/u);
  }
});
