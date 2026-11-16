import assert from "node:assert/strict";

interface ProfileNode {
  id: number;
  callFrame: {
    functionName: string;
    url: string;
    lineNumber: number;
    columnNumber: number;
  };
}

interface CpuProfile {
  nodes: ProfileNode[];
  samples: number[];
  timeDeltas: (number | null)[];
}

interface ProfileRow {
  function: string;
  file: string;
  line: number;
  column: number;
  samples: number;
  sampledIntervalsMicros: number | null;
}

/** Preserve sample order; signed SDK deltas never become invented durations. */
export function analyseProfile(profile: CpuProfile) {
  assert.ok(
    Array.isArray(profile.nodes) &&
      Array.isArray(profile.samples) &&
      Array.isArray(profile.timeDeltas),
  );
  assert.equal(profile.samples.length, profile.timeDeltas.length);
  const nodes = new Map(profile.nodes.map((node) => [node.id, node]));
  assert.equal(nodes.size, profile.nodes.length, "profile node IDs must be unique");
  const negativeDeltas: { index: number; nodeId: number; delta: number }[] = [];
  const validatedDeltas = profile.timeDeltas.map((delta, index) => {
    assert.equal(typeof delta, "number", "raw timing deltas must be signed integers");
    if (typeof delta !== "number") assert.fail("raw timing deltas must be signed integers");
    assert.ok(Number.isSafeInteger(delta), "raw timing deltas must be signed integers");
    if (delta < 0) negativeDeltas.push({ index, nodeId: profile.samples[index], delta });
    return delta;
  });
  const monotonic = negativeDeltas.length === 0;
  const rows = new Map<number, ProfileRow>();
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
    if (monotonic) {
      const intervals = row.sampledIntervalsMicros;
      assert.notEqual(intervals, null);
      if (intervals === null) assert.fail("monotonic profiles retain numeric intervals");
      row.sampledIntervalsMicros = intervals + validatedDeltas[index];
    }
    rows.set(node.id, row);
  }
  return {
    samples: profile.samples.length,
    monotonic,
    negativeDeltas,
    adapter: [...rows.values()].sort((left, right) => right.samples - left.samples),
  };
}
