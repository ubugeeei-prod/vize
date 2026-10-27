/** Exact source-count/allocation laws, independent of timing and host speed. */
import assert from "node:assert/strict";
import {
  checkExactJson,
  integer,
  list,
  positive,
  record,
  render,
  string,
  type RecordValue,
} from "./profile-evidence-json.ts";

export function signature(row: RecordValue): unknown[] {
  return ["iterations", "sourceBytes", "sourceHash", "rows", "scenario", "checksum"].map(
    (key) => row[key],
  );
}
function count(row: RecordValue, name: string): [bigint, bigint] {
  const entries = list(row.micro)
    .map(record)
    .filter((entry) => entry.name === name);
  assert(entries.length <= 1, name);
  if (!entries.length) return [0n, 0n];
  const calls = integer(entries[0]!.calls),
    units = integer(entries[0]!.units);
  assert(calls >= 0n && units >= 0n);
  return [calls, units];
}
export function profilePair(left: RecordValue, right: RecordValue) {
  assert.deepEqual(signature(left), signature(right));
  assert.equal(left.instrumented, true);
  assert.equal(right.instrumented, true);
  const iterations = positive(left.iterations),
    bytes = positive(left.sourceBytes);
  assert(["irreparable", "valid", "repairable"].includes(string(left.scenario)));
  for (const row of [left, right]) {
    assert.deepEqual(count(row, "croquis.oxc.original.bytes"), [iterations, iterations * bytes]);
    assert.deepEqual(count(row, "croquis.setup.parse_requests.bytes"), [
      iterations,
      iterations * bytes,
    ]);
    const analyses = row.scenario === "irreparable" ? 0n : iterations;
    assert.deepEqual(count(row, "croquis.setup.analysis_requests.bytes"), [
      analyses,
      analyses * bytes,
    ]);
    const recoveries = row.scenario === "repairable" ? iterations : 0n;
    assert.deepEqual(count(row, "croquis.oxc.recovery.bytes"), [recoveries, recoveries * bytes]);
  }
  const semantic = (row: RecordValue) =>
    list(row.micro)
      .map(record)
      .filter((entry) => string(entry.name).startsWith("croquis."))
      .map((entry) => [string(entry.name), integer(entry.calls), integer(entry.units)] as const)
      .sort(
        (a, b) =>
          (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0) ||
          (a[1] < b[1] ? -1 : a[1] > b[1] ? 1 : a[2] < b[2] ? -1 : a[2] > b[2] ? 1 : 0),
      );
  assert.deepEqual(semantic(left), semantic(right));
  const base = record(left.allocation),
    head = record(right.allocation);
  const callsSaved = integer(base.calls) - integer(head.calls);
  const bytesSaved = integer(base.requestedBytes) - integer(head.requestedBytes);
  if (left.scenario === "irreparable") {
    assert.equal(callsSaved, iterations);
    assert(bytesSaved >= iterations * bytes);
    assert.equal(bytesSaved % iterations, 0n);
  } else {
    assert.equal(callsSaved, 0n);
    assert.equal(bytesSaved, 0n);
  }
  return {
    scope:
      "instrumented successful Rust allocation requests; cumulative layout traffic, not copied/live/RSS bytes",
    base,
    head,
    callsSaved,
    requestedBytesSaved: bytesSaved,
    savedLayoutBytesPerParse: bytesSaved / iterations,
    originalParsesPerRole: iterations,
    setupAnalysesPerRole: left.scenario === "irreparable" ? 0n : iterations,
    recoveryParsesPerRole: left.scenario === "repairable" ? iterations : 0n,
    croquisSourceCountsEqual: true,
  };
}
export function selfTest(): void {
  checkExactJson();
  const fixture = (scenario: string, calls: bigint, allocated: bigint): RecordValue => {
    const recovery = scenario === "repairable" ? 4n : 0n,
      analysis = scenario === "irreparable" ? 0n : 4n;
    return {
      iterations: 4n,
      sourceBytes: 100n,
      sourceHash: 17n,
      rows: 1n,
      scenario,
      checksum: 0n,
      instrumented: true,
      micro: [
        { name: "croquis.oxc.original.bytes", calls: 4n, units: 400n },
        { name: "croquis.oxc.recovery.bytes", calls: recovery, units: 100n * recovery },
        { name: "croquis.setup.parse_requests.bytes", calls: 4n, units: 400n },
        { name: "croquis.setup.analysis_requests.bytes", calls: analysis, units: analysis * 100n },
      ],
      allocation: { calls, requestedBytes: allocated },
    };
  };
  type Pair = [RecordValue, RecordValue];
  const irreparable: Pair = [fixture("irreparable", 10n, 1000n), fixture("irreparable", 6n, 600n)];
  assert.equal(profilePair(...irreparable).callsSaved, 4n);
  for (const scenario of ["valid", "repairable"]) {
    const row = fixture(scenario, 10n, 1000n);
    assert.equal(profilePair(row, structuredClone(row)).callsSaved, 0n);
  }
  const cases: [string, Pair][] = [];
  const add = (name: string, pair: Pair, mutate: (row: Pair) => void) => {
    const row = structuredClone(pair);
    mutate(row);
    cases.push([name, row]);
  };
  add("original-call-mismatch", irreparable, (row) => {
    record(list(row[1].micro)[0]).calls = 3n;
  });
  add(
    "repair-not-observed",
    [fixture("repairable", 10n, 1000n), fixture("repairable", 10n, 1000n)],
    (row) => {
      list(row[1].micro).splice(1, 1);
    },
  );
  add("no-removed-copy", irreparable, (row) => {
    record(row[1].allocation).calls = 10n;
  });
  add("two-removed-copies", irreparable, (row) => {
    record(row[1].allocation).calls = 2n;
  });
  add("too-few-removed-bytes", irreparable, (row) => {
    record(row[1].allocation).requestedBytes = 999n;
  });
  add(
    "valid-allocation-changed",
    [fixture("valid", 10n, 1000n), fixture("valid", 10n, 1000n)],
    (row) => {
      record(row[1].allocation).calls = 9n;
    },
  );
  add("source-mismatch", irreparable, (row) => {
    row[1].sourceHash = 18n;
  });
  add("normal-sample-as-profile", irreparable, (row) => {
    row[1].instrumented = false;
  });
  add("zero-iterations", irreparable, (row) => {
    row[0].iterations = row[1].iterations = 0n;
  });
  add("unsupported-scenario", irreparable, (row) => {
    row[0].scenario = row[1].scenario = "unknown";
  });
  for (const [name, pair] of cases) assert.throws(() => profilePair(...pair), Error, name);
  console.log(
    "clone-contract-controls=" + render({ positive: 3n, rejected: cases.map(([name]) => name) }),
  );
}
