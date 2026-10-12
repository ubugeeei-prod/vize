import assert from "node:assert/strict";

// This contract applies only to the authenticated ESLint 10.4.1 serializer.
// No semantic diagnostic field, case identity or source hash may change.
export function validateProviderEnvelope(corpus, supplement) {
  assert.equal(supplement.schema, "vize.lint.public-vue-imports-provider-envelope-supplement");
  assert.equal(supplement.serializer.package, "eslint");
  assert.equal(supplement.serializer.version, "10.4.1");
  assert.equal(supplement.cases.length, corpus.cases.length);
  let removed = 0;
  for (let i = 0; i < corpus.cases.length; i++) {
    const initial = corpus.cases[i];
    const expected = initial.providerDiagnostics.map((finding) => {
      assert.equal(finding.nodeType, "Literal");
      const { nodeType: _nodeType, ...rest } = finding;
      removed++;
      return rest;
    });
    assert.deepEqual(supplement.cases[i], {
      id: initial.id,
      sourceSha256: initial.sourceSha256,
      diagnostics: expected,
    });
  }
  assert.deepEqual(supplement.changes, {
    cases: 19,
    fieldsRemoved: removed,
    removedField: "nodeType",
    removedValue: "Literal",
    semanticFieldsChanged: 0,
    nativeVectorsChanged: 0,
    checksDisabled: 0,
  });
  assert.equal(removed, 19);
}

export function validatePublicExportTable(names, pinned) {
  assert.equal(names.length, 361);
  assert.equal(new Set(names).size, 361);
  assert.deepEqual(
    names,
    [...pinned].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
  );
}
