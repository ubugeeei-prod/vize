/** Exact admission for software source counters; missing hardware fields stay null. */
import assert from "node:assert/strict";

interface Counter {
  key: string;
  name: string;
  calls: number;
  units: number;
}
interface Span {
  key: string;
  requestedAllocationCalls: number;
  requestedAllocationBytes: number;
}
interface Window {
  group: string;
  iterations: number;
  cases: number;
  successes: number;
  registeredDuringWarmup: Counter[];
  sourceCounters: Counter[];
  sourceSpans: Span[];
}
export interface Role {
  role: string;
  fixtureRootPhysical: string;
  compiledSourceRevision: string;
  windows: Window[];
  normalPerformanceQualified: boolean;
  coldQualified: boolean;
}

const count = (w: Window, name: string) => {
  assert.ok(
    w.registeredDuringWarmup.some((site) => site.name === name),
    "site must be proven registered before inferring software zero",
  );
  return w.sourceCounters
    .filter((site) => site.name === name)
    .reduce((sum, site) => {
      assert.ok(Number.isSafeInteger(site.units));
      return sum + site.units;
    }, 0);
};
const historyAlloc = (w: Window) => {
  const keys = new Set(
    w.registeredDuringWarmup
      .filter(
        (site) =>
          site.name === "cli.imports.probe.history.initial_allocation" ||
          site.name === "cli.imports.probe.history.push_allocation",
      )
      .map((site) => site.key),
  );
  return w.sourceSpans
    .filter((span) => keys.has(span.key))
    .reduce(
      (sum, span) => ({
        calls: sum.calls + span.requestedAllocationCalls,
        bytes: sum.bytes + span.requestedAllocationBytes,
      }),
      { calls: 0, bytes: 0 },
    );
};

export function validateProbeMicroRoles(roles: Record<string, Role>) {
  const groups = [
    { name: "relative", cases: 7, positives: 6, probes: 7 },
    { name: "absolute", cases: 2, positives: 1, probes: 2 },
    { name: "alias", cases: 4, positives: 3, probes: 7 },
  ];
  assert.equal(roles.legacy.windows.length, groups.length);
  assert.equal(roles.candidate.windows.length, groups.length);
  return roles.legacy.windows.map((before, index) => {
    const expected = groups[index];
    assert.equal(before.group, expected.name);
    assert.equal(before.iterations, 1000);
    assert.equal(before.cases, expected.cases);
    assert.equal(before.successes, expected.positives * before.iterations);
    assert.equal(
      count(before, "cli.imports.probe.total.calls"),
      expected.probes * before.iterations,
    );
    assert.ok(count(before, "cli.imports.probe.is_file.calls") > 0);
    const after = roles.candidate.windows[index];
    assert.equal(after.group, before.group);
    assert.equal(after.iterations, before.iterations);
    assert.equal(after.cases, before.cases);
    assert.equal(after.successes, before.successes);
    for (const label of ["cli.imports.probe.total.calls", "cli.imports.probe.is_file.calls"])
      assert.equal(count(after, label), count(before, label));
    const label = "cli.imports.probe.history.path_entries",
      bytes = "cli.imports.probe.history.path_encoded_bytes";
    if (before.group === "alias") {
      assert.ok(
        count(before, label) > 0,
        "retained alias proof inputs must actually be constructed",
      );
      assert.equal(count(after, label), count(before, label));
      assert.equal(count(after, bytes), count(before, bytes));
    } else {
      assert.ok(count(before, label) > 0);
      assert.equal(count(after, label), 0);
      assert.equal(count(after, bytes), 0);
      assert.ok(
        historyAlloc(before).calls > 0 && historyAlloc(before).bytes > 0,
        "allocator must actually be active",
      );
      assert.equal(historyAlloc(after).calls, 0);
      assert.equal(historyAlloc(after).bytes, 0);
    }
    return {
      group: before.group,
      probeCalls: count(before, "cli.imports.probe.total.calls"),
      isFileCalls: count(before, "cli.imports.probe.is_file.calls"),
      historyPaths: { legacy: count(before, label), candidate: count(after, label) },
      historyEncodedBytes: { legacy: count(before, bytes), candidate: count(after, bytes) },
      historyRequestedAllocations: { legacy: historyAlloc(before), candidate: historyAlloc(after) },
      sourceCounters: { legacy: before.sourceCounters, candidate: after.sourceCounters },
      sourceSpans: { legacy: before.sourceSpans, candidate: after.sourceSpans },
    };
  });
}
