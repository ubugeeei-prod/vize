import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";
import { parenthesisDiagnosticMappings } from "./support/content-mapper-parenthesis-mappings-11675.ts";

function assertParenthesisTransition(before: Transform, current: Transform) {
  const atoms = parenthesisDiagnosticMappings(current);
  const keys = new Set(atoms.map((row) => JSON.stringify(row)));
  const remaining = current.mappings.filter((row) => !keys.has(JSON.stringify(row)));
  assert.equal(current.mappings.length - remaining.length, atoms.length, "one row per delimiter");
  assert.deepEqual(remaining, before.mappings, "all original mapping rows and features stay exact");
  const expected = structuredClone(before);
  const directives = expected.diagnosticDirectives as { directives: number[][] } | undefined;
  if (directives)
    directives.directives = directives.directives.flatMap((row) => {
      let ranges = [[row[2], row[3]]];
      for (const [at] of atoms)
        ranges = ranges.flatMap(([start, end]) =>
          at >= end || at + 1 <= start
            ? [[start, end]]
            : [
                [start, at],
                [at + 1, end],
              ].filter(([left, right]) => left < right),
        );
      return ranges.map(([start, end]) => [row[0], row[1], start, end, row[4], row[5]]);
    });
  const { mappings: _before, ...oldFields } = expected;
  const { mappings: _current, ...newFields } = current;
  assert.deepEqual(newFields, oldFields, "only newly mapped delimiter bytes leave ignore ranges");
  assert.deepEqual(
    current.mappings.filter((row) => row[5] === 0),
    [...before.mappings.filter((row) => row[5] === 0), ...atoms].sort((a, b) => a[0] - b[0]),
    "the complete diagnostic roster retains old receivers and exactly paired parentheses",
  );
  return atoms;
}

test("10617 current packets add only the independently observed 11675 diagnostic delimiters", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const previousName = "source-packets-current-2d46a96fbefd.json";
  const before = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  const parenthesisName = "source-packets-current-beca9eaf0681.json";
  const parenthesisRaw = readFileSync(
    new URL(
      `../fixtures/typechecker/undeclared-instance-globals-10617/${parenthesisName}`,
      import.meta.url,
    ),
    "utf8",
  );
  assert.equal(
    sha(parenthesisRaw),
    "8d388c0430adc0200efd324478c2f59e701a857e381b35988502249f746d4a08",
  );
  const current = JSON.parse(parenthesisRaw);
  assert.equal(authority.currentSourcePackets, "source-packets-current-f29a8ee58999.json");
  const proofRaw = readFileSync(
    new URL(
      "../fixtures/typechecker/undeclared-instance-globals-10617/source-packets-current-beca9eaf0681-provenance.json",
      import.meta.url,
    ),
    "utf8",
  );
  assert.equal(sha(proofRaw), "dcb060691f5181c76c3f8cf5d25e3d5417b8a2f6110e2bc344a062b0c642d319");
  const proof = JSON.parse(proofRaw);
  assert.equal(proof.previous.sha256, sha(files.get(previousName)!));
  assert.equal(proof.featureCommit, "46b752b95bd8e2b955148866655e01b58a166fe4");
  assert.equal(proof.producer.sourceRevision, "beca9eaf068137e6dffd818c4949b522c4919dd1");
  assert.deepEqual(current.producer, proof.producer);
  assert.equal(proof.sourceLaunch.receipt.binarySha256, proof.producer.binarySha256);
  assert.equal(proof.actualQualification.status, "unqualified");
  assert.equal(proof.actualQualification.outcomes.length, 398);
  const gaps = proof.actualQualification.outcomes.filter(
    (row: { outcome: string }) => row.outcome !== "pass",
  );
  assert.equal(gaps.length, 66, "the original capture remains failed, with no acceptance credit");
  let changed = 0;
  let pairs = 0;
  for (const [index, packet] of current.packets.entries()) {
    const old = before.packets[index]!;
    assert.equal(packet.label, old.label);
    assert.equal(packet.sourceSha256, old.sourceSha256);
    assert.equal(packet.globalsSha256, old.globalsSha256);
    const atoms = assertParenthesisTransition(old.result, packet.result);
    assert.equal(proof.proofs[index].previousResultSha256, sha(JSON.stringify(old.result)));
    assert.equal(proof.proofs[index].currentResultSha256, sha(JSON.stringify(packet.result)));
    if (atoms.length) changed += 1;
    pairs += atoms.length / 2;
  }
  assert.equal(changed, 32);
  assert.equal(pairs, 36);
});

test("11675 current packet comparisons reject missing, widened, foreign and extra delimiter identities", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const old = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-2d46a96fbefd.json" },
    files,
    cases,
  )!.packets.find((row) => row.label === "lf/setup-shadow")!.result;
  // The population that added the delimiters; later successors relocate them.
  const current = JSON.parse(
    readFileSync(
      new URL(
        "../fixtures/typechecker/undeclared-instance-globals-10617/source-packets-current-beca9eaf0681.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ).packets.find((row: { label: string }) => row.label === "lf/setup-shadow")!.result as Transform;
  assert.deepEqual(parenthesisDiagnosticMappings(current), [
    [1745, 1, 94, 20, 1, 0],
    [1766, 1, 94, 20, 1, 0],
  ]);
  const index = current.mappings.findIndex((row) => row[0] === 1745);
  for (let field = 0; field < 6; field++) {
    const changed = structuredClone(current);
    changed.mappings[index][field] += 1;
    assert.throws(() => assertParenthesisTransition(old, changed), `delimiter field ${field}`);
  }
  const mutations: Array<(copy: Transform) => void> = [
    (copy) => {
      copy.mappings.splice(index, 1);
    },
    (copy) => {
      copy.mappings.push([...copy.mappings[index]]);
    },
    (copy) => {
      copy.mappings[index + 1][5] = 0;
    },
    (copy) => {
      copy.text = copy.text.slice(0, 1745) + "[" + copy.text.slice(1746);
    },
    (copy) => {
      copy.diagnostics.push({ message: "unexpected diagnostic" });
    },
    (copy) => {
      copy.extra = "unknown producer field";
    },
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(current);
    mutate(changed);
    assert.notDeepEqual(changed, current);
    assert.throws(() => assertParenthesisTransition(old, changed));
  }
});
