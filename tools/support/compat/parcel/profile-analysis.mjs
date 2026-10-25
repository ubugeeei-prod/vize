import assert from "node:assert/strict";

/** Preserve sample order; signed SDK deltas never become invented durations. */
export function analyseProfile(profile) {
  assert.ok(
    Array.isArray(profile.nodes) &&
      Array.isArray(profile.samples) &&
      Array.isArray(profile.timeDeltas),
  );
  assert.equal(profile.samples.length, profile.timeDeltas.length);
  const nodes = new Map(profile.nodes.map((node) => [node.id, node]));
  assert.equal(nodes.size, profile.nodes.length, "profile node IDs must be unique");
  const negativeDeltas = [];
  for (let index = 0; index < profile.timeDeltas.length; index++) {
    const delta = profile.timeDeltas[index];
    assert.ok(Number.isSafeInteger(delta), "raw timing deltas must be signed integers");
    if (delta < 0) negativeDeltas.push({ index, nodeId: profile.samples[index], delta });
  }
  const monotonic = negativeDeltas.length === 0;
  const rows = new Map();
  for (let index = 0; index < profile.samples.length; index++) {
    const node = nodes.get(profile.samples[index]);
    assert.ok(node, "every sample must name an existing node");
    assert.equal(typeof node.callFrame.url, "string");
    if (!node.callFrame.url.includes("/@vizejs/parcel-transformer/")) continue;
    const row = rows.get(node.id) ?? {
      function: node.callFrame.functionName,
      file: node.callFrame.url,
      line: node.callFrame.lineNumber + 1,
      column: node.callFrame.columnNumber + 1,
      samples: 0,
      sampledIntervalsMicros: monotonic ? 0 : null,
    };
    row.samples++;
    if (monotonic) row.sampledIntervalsMicros += profile.timeDeltas[index];
    rows.set(node.id, row);
  }
  return {
    samples: profile.samples.length,
    monotonic,
    negativeDeltas,
    adapter: [...rows.values()].sort((left, right) => right.samples - left.samples),
  };
}
