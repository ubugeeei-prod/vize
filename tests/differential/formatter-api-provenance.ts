import assert from "node:assert/strict";

// Schema 2 factors repeated source-file identities out of capture declarations.
// It preserves every authored expression range and original/current witness;
// normalization supplies declarations, never generated oracle output bytes.
export function normalizeFormatterHistoryManifest(manifest: any) {
  assert([1, 2].includes(manifest.version));
  if (manifest.version === 1) return manifest;
  assert(manifest.sources && typeof manifest.sources === "object");
  function witness(value: any) {
    if (!value?.sourceRef) return value;
    assert(Object.hasOwn(manifest.sources, value.sourceRef), "unknown source witness");
    const source = manifest.sources[value.sourceRef];
    assert.equal(typeof source.path, "string");
    assert.match(source.sourceSha256, /^[a-f0-9]{64}$/);
    if (source.commit) assert.match(source.commit, /^[a-f0-9]{40}$/);
    const { sourceRef: _, ...detail } = value;
    for (const key of Object.keys(source)) {
      assert(!Object.hasOwn(detail, key), "source identity override");
    }
    return { ...source, ...detail };
  }
  return {
    ...manifest,
    cases: manifest.cases.map((fixture: any) => ({
      ...fixture,
      witness: witness(fixture.witness),
      originalInputWitnesses: fixture.originalInputWitnesses.map(witness),
      additionalWitnesses: (fixture.additionalWitnesses ?? []).map(witness),
    })),
  };
}
