import assert from "node:assert/strict";
import { instanceReceiverReads, type Transform } from "./content-mapper-instance-globals-10617.ts";

// Public protocol geometry: ordinary property tokens retain all capabilities;
// synthetic receiver prefixes have atom affinity and diagnostics-only identity.
const allFeatures = 1048575;
export function assertDiagnosticPrefixGeometry(
  source: string,
  result: Transform,
  referencePackage: string,
) {
  const bytes = Buffer.from(result.text);
  const authored = Buffer.from(source);
  const expected = [];
  for (const read of instanceReceiverReads(result, referencePackage)) {
    if (read.receiver !== "__ctx") continue;
    const property = result.mappings.filter(
      (span) => span[0] === read.start && span[1] === read.length && span[5] !== 0,
    );
    // Unmapped runtime-context prelude declarations have no authored identity.
    if (!property.length) continue;
    assert.equal(property.length, 1, "one exact source owner per generated property token");
    const span = property[0];
    assert.equal(span.length, 6);
    assert.equal(span[3], read.length);
    assert.equal(span[4], 0, "the property remains an ordinary navigable token");
    assert.equal(
      span[5],
      allFeatures,
      "the original property navigation capabilities are retained",
    );
    assert.equal(
      authored.subarray(span[2], span[2] + span[3]).toString(),
      read.name,
      "whole own dollar token",
    );
    assert.equal(read.prefixLength, 6);
    assert.equal(
      bytes.subarray(read.prefixStart, read.start).toString(),
      "__ctx.",
      "AST-authorized synthetic receiver region",
    );
    expected.push([read.prefixStart, 6, span[2], span[3], 1, 0]);
  }
  expected.sort((a, b) => a[0] - b[0]);
  assert.deepEqual(
    result.mappings.filter((span) => span[5] === 0),
    expected,
    "the complete diagnostic-only roster must contain exactly the mapped AST receiver prefixes",
  );
  return expected;
}

export function assertPrefixGeometryMutations(
  source: string,
  result: Transform,
  referencePackage: string,
) {
  assertDiagnosticPrefixGeometry(source, result, referencePackage);
  const index = result.mappings.findIndex((span) => span[5] === 0);
  assert.notEqual(index, -1, "the mutation control needs a real source prefix");
  const prefix = result.mappings[index];
  const propertyIndex = result.mappings.findIndex(
    (span) => span[0] === prefix[0] + prefix[1] && span[2] === prefix[2] && span[5] === allFeatures,
  );
  assert.notEqual(propertyIndex, -1);
  const mutations: Array<(copy: Transform) => void> = [
    (copy) => {
      copy.mappings.splice(index, 1);
    },
    (copy) => {
      copy.mappings[index][5] = allFeatures;
    },
    (copy) => {
      copy.mappings[propertyIndex][5] = 0;
    },
    (copy) => {
      copy.mappings[propertyIndex][4] = 1;
    },
    (copy) => {
      copy.mappings.push([...copy.mappings[index]]);
    },
    (copy) => {
      copy.text = copy.text.slice(0, prefix[0]) + "__bad." + copy.text.slice(prefix[0] + 6);
    },
  ];
  for (const field of [0, 1, 2, 3, 4])
    for (const delta of [-1, 1])
      mutations.push((copy) => {
        copy.mappings[index][field] += delta;
      });
  for (const mutate of mutations) {
    const copy = structuredClone(result);
    mutate(copy);
    assert.notEqual(JSON.stringify(copy), JSON.stringify(result));
    assert.throws(() => assertDiagnosticPrefixGeometry(source, copy, referencePackage));
  }
}
