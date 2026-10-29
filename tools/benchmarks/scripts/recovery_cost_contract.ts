/** Only the two pinned incomplete-member retry classes are admissible. */
import assert from "node:assert/strict";
import {
  checkExactJson,
  integer,
  isMain,
  list,
  positive,
  record,
  render,
  type RecordValue,
} from "./profile-evidence-json.ts";

function sourceCounter(sample: RecordValue, name: string): [bigint, bigint] {
  const entries = list(sample.micro)
    .map(record)
    .filter((row) => row.name === name);
  assert.equal(entries.length, 1, "missing or duplicate source counter: " + name);
  return [positive(entries[0]!.calls), positive(entries[0]!.units)];
}
export function validate(value: unknown) {
  const profiles = record(value);
  assert.deepEqual(Object.keys(profiles).sort(), ["base", "head"]);
  const base = record(profiles.base),
    head = record(profiles.head);
  for (const key of ["iterations", "sourceBytes", "sourceHash", "checksum", "rows", "scenario"])
    assert.deepEqual(base[key], head[key], key);
  assert.equal(base.scenario, "incomplete-member");
  integer(base.sourceHash);
  const iterations = positive(base.iterations),
    bytes = positive(base.sourceBytes);
  const counts: Record<string, { original: [bigint, bigint]; recovery: [bigint, bigint] }> = {};
  for (const role of ["base", "head"]) {
    const sample = record(profiles[role]);
    assert.equal(sample.instrumented, true);
    positive(sample.iterations);
    positive(sample.sourceBytes);
    const original = sourceCounter(sample, "croquis.oxc.original.bytes");
    const recovery = sourceCounter(sample, "croquis.oxc.recovery.bytes");
    assert.deepEqual(original, [iterations, iterations * bytes]);
    assert.equal(recovery[1], recovery[0] * bytes);
    if (role === "head") assert.equal(recovery[0], iterations);
    for (const name of ["calls", "requestedBytes"]) positive(record(sample.allocation)[name]);
    counts[role] = { original, recovery };
  }
  const baseCount = counts.base!,
    headCount = counts.head!;
  const baseAllocation = record(base.allocation),
    headAllocation = record(head.allocation);
  let comparison: string;
  if (baseCount.recovery[0] === 2n * iterations) {
    for (const name of ["calls", "requestedBytes"])
      assert(positive(headAllocation[name]) < positive(baseAllocation[name]));
    comparison = "historical-two-to-one-strict-allocation-reduction";
  } else if (baseCount.recovery[0] === iterations) {
    assert.deepEqual(baseCount, headCount);
    assert.deepEqual(baseAllocation, headAllocation);
    comparison = "already-one-to-one-exact-unchanged-control";
  } else {
    throw new Error("unsupported baseline parse count");
  }
  return {
    class: comparison,
    sourceCounts: counts,
    allocation: { base: baseAllocation, head: headAllocation },
  };
}
function fixture(multiplier: bigint, calls: bigint, allocated: bigint): RecordValue {
  const iterations = 10n,
    bytes = 100n;
  return {
    iterations,
    sourceBytes: bytes,
    sourceHash: 71n,
    checksum: 2570n,
    rows: 256n,
    scenario: "incomplete-member",
    instrumented: true,
    micro: [
      { name: "croquis.oxc.original.bytes", calls: iterations, units: iterations * bytes },
      {
        name: "croquis.oxc.recovery.bytes",
        calls: iterations * multiplier,
        units: iterations * multiplier * bytes,
      },
    ],
    allocation: { calls, requestedBytes: allocated },
  };
}
export function selfTest(): void {
  checkExactJson();
  const head = fixture(1n, 70n, 8000n);
  const two = { base: fixture(2n, 100n, 10000n), head };
  const one = { base: structuredClone(head), head };
  assert(validate(two).class.startsWith("historical-two-to-one"));
  assert(validate(one).class.startsWith("already-one-to-one"));
  type Pair = typeof one;
  const cases: [string, Pair][] = [];
  const add = (name: string, source: Pair, mutate: (row: Pair) => void) => {
    const value = structuredClone(source);
    mutate(value);
    cases.push([name, value]);
  };
  add("missing-source-counter", one, (value) => {
    list(value.head.micro).pop();
  });
  add("duplicate-source-counter", one, (value) => {
    const rows = list(value.head.micro);
    rows.push(structuredClone(rows.at(-1)));
  });
  add("zero-iterations", one, (value) => {
    value.base.iterations = value.head.iterations = 0n;
  });
  add("unsupported-base-count", two, (value) => {
    value.base = fixture(3n, 100n, 10000n);
  });
  add("head-two-repairs", two, (value) => {
    value.head = fixture(2n, 70n, 8000n);
  });
  add("recovery-byte-mismatch", one, (value) => {
    record(list(value.head.micro).at(-1)).units = 999n;
  });
  add("original-call-mismatch", one, (value) => {
    record(list(value.head.micro)[0]).calls = 9n;
  });
  add("same-class-call-increase", one, (value) => {
    record(value.head.allocation).calls = 71n;
  });
  add("same-class-byte-increase", one, (value) => {
    record(value.head.allocation).requestedBytes = 8001n;
  });
  add("same-class-unrelated-decrease", one, (value) => {
    record(value.head.allocation).requestedBytes = 7999n;
  });
  add("two-to-one-no-byte-win", two, (value) => {
    record(value.head.allocation).requestedBytes = 10000n;
  });
  add("two-to-one-no-call-win", two, (value) => {
    record(value.head.allocation).calls = 100n;
  });
  add("source-identity-mismatch", one, (value) => {
    value.head.sourceHash = 72n;
  });
  add("unowned-scenario", one, (value) => {
    value.base.scenario = value.head.scenario = "valid";
  });
  add("non-instrumented-sample", one, (value) => {
    value.head.instrumented = false;
  });
  add("boolean-iterations", one, (value) => {
    value.head.iterations = true;
  });
  for (const [name, value] of cases) assert.throws(() => validate(value), Error, name);
  assert.throws(() => validate({ ...one, extra: head }));
  console.log(
    "recovery-class-controls=" +
      render({ positive: 2n, rejected: [...cases.map(([name]) => name), "unexpected-role"] }),
  );
}
if (isMain(import.meta.url)) selfTest();
