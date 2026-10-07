import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
const read = (root, relative) => fs.readFileSync(path.join(root, relative));

const fixture = "tests/_fixtures/differential/compiler/n8n-if-key-regression";

export function validateDiagnosticContract(output, source) {
  const contract = JSON.parse(read(output, "diagnostic-contract.json"));
  const coordinates = JSON.parse(read(source, `${fixture}/diagnostics.json`));
  const template = read(source, `${fixture}/official/template.txt`).toString("utf8");
  assert.deepEqual(contract.coordinates, coordinates);
  assert.equal(coordinates.length, 3);
  assert.equal(contract.defaultProduction, "refused");
  assert.equal(contract.prefixedProduction, "returned");
  assert.deepEqual(
    contract.legacyErrors,
    coordinates.map((item) => ({
      code: item.code,
      message: item.message,
      location: { span: item.span },
    })),
  );
  assert.equal(contract.nativeDiagnostics.length, 3);
  for (const [index, item] of coordinates.entries()) {
    assert.equal(item.officialCode, 29);
    for (const side of ["start", "end"]) {
      const prefix = template.slice(0, item.officialLocation[side].offset);
      assert.equal(Buffer.byteLength(prefix), item.span[side]);
      assert.equal(prefix.split("\n").length, item.officialLocation[side].line);
      assert.equal(prefix.split("\n").at(-1).length + 1, item.officialLocation[side].column);
    }
    assert.equal(
      Buffer.from(template).subarray(item.span.start, item.span.end).toString("utf8"),
      item.officialLocation.source,
    );
    const diagnostic = contract.nativeDiagnostics[index];
    assert.deepEqual(Object.keys(diagnostic).sort(), [
      "code",
      "completeDebug",
      "message",
      "parts",
      "severity",
      "span",
      "stage",
      "witness",
    ]);
    assert.equal(diagnostic.code, item.code);
    assert.equal(diagnostic.message, item.message);
    assert.deepEqual(diagnostic.span, item.span);
    assert.equal(diagnostic.stage, "Semantic");
    assert.equal(diagnostic.severity, "Error");
    assert.ok(Array.isArray(diagnostic.parts));
    assert.equal(typeof diagnostic.witness, "string");
    assert.equal(typeof diagnostic.completeDebug, "string");
  }
  return contract;
}

export function validateInstanceMode(row, contract) {
  assert.deepEqual(
    row.native.loweredDiagnostics,
    contract.nativeDiagnostics,
    "complete ordered mode-neutral native diagnostics including metadata",
  );
  assert.deepEqual(
    row.legacy.errors,
    row.prefixIdentifiers ? [] : contract.legacyErrors,
    "complete ordered legacy diagnostic output at matching mode",
  );
  assert.deepEqual(
    row.native.effectiveDiagnostics,
    row.prefixIdentifiers ? [] : contract.nativeDiagnostics,
  );
  assert.equal(
    row.native.production.status,
    row.prefixIdentifiers ? contract.prefixedProduction : contract.defaultProduction,
  );
  assert.equal(row.native.diagnosedEmission.status, "diagnosed-output");
  assert.equal(
    row.native.diagnosedEmission.assembled,
    row.legacy.assembled,
    "complete original diagnosed code at matching mode",
  );
  if (row.prefixIdentifiers) {
    assert.equal(row.native.assembled, row.legacy.assembled);
    assert.equal(row.native.production.assembled, row.legacy.assembled);
    assert.equal(row.native.error, null);
  } else {
    assert.equal(row.native.assembled, null);
    assert.equal(typeof row.native.production.error, "string");
    assert.equal(row.native.error, row.native.production.error);
  }
}

export function validateBaselineDiagnostics(before, after, contract) {
  const filename = "InstanceAiConfirmationPanel.vue.json";
  const baseline = JSON.parse(read(before, filename));
  const current = JSON.parse(read(after, filename));
  assert.equal(baseline.originalSource, current.originalSource);
  const original = baseline.rows.find((row) => row.kind === "template" && !row.prefixIdentifiers);
  assert.equal(
    original.native.loweredDiagnostics.length,
    2,
    "immutable original source has exactly two complete raw diagnostics",
  );
  assert.deepEqual(
    original.native.loweredDiagnostics.map((item) => item.span),
    contract.coordinates.slice(0, 2).map((item) => item.span),
  );
  const expanded = [
    original.native.loweredDiagnostics[0],
    original.native.loweredDiagnostics[1],
    original.native.loweredDiagnostics[1],
  ];
  assert.equal(expanded.length, 3);
  assert.deepEqual(
    contract.nativeDiagnostics,
    expanded,
    "whole native contract must preserve immutable raw metadata under official per-prior-key law",
  );
  const historicalLegacy = [0, 1].map(() => ({
    code: "VIfSameKey",
    message: "v-if/v-else-if branches must use unique keys.",
    location: null,
  }));
  assert.deepEqual(
    original.legacy.errors,
    historicalLegacy,
    "complete historical legacy errors preserve their original missing locations",
  );
  for (const row of current.rows.filter((row) => row.kind === "template"))
    assert.deepEqual(
      row.native.loweredDiagnostics,
      expanded,
      "whole current native metadata must equal authenticated baseline expansion",
    );
  return {
    law: "One diagnostic for each matching prior branch key; complete raw metadata retained.",
    beforeNative: original.native.loweredDiagnostics,
    expectedNative: expanded,
    beforeLegacy: original.legacy.errors,
    expectedCurrentLegacy: contract.legacyErrors,
  };
}
