import assert from "node:assert/strict";
import { test } from "node:test";
import {
  inferInstructionScope,
  retiredInstructions,
} from "../../tools/benchmarks/scripts/source-macos-rusage-scope.ts";

test("missing retired-instruction fields stay unavailable and unrelated numeric lines do not substitute", () => {
  assert.deepEqual(retiredInstructions("1234 maximum resident set size\n42 cycles elapsed\n"), {
    raw: null,
    count: null,
  });
  assert.deepEqual(retiredInstructions("  123456 instructions retired\n42 cycles elapsed\n"), {
    raw: "123456",
    count: 123456,
  });
  assert.deepEqual(retiredInstructions("9007199254740992 instructions retired\n"), {
    raw: "9007199254740992",
    count: null,
  });
});

test("worker inclusion and waited-child exclusion are separate controlled inferences", () => {
  const excluded = inferInstructionScope({
    baseline: 100,
    instructions: 1100,
    threads: 2100,
    child: 200,
  });
  assert.equal(excluded.workerThreads, "included");
  assert.equal(excluded.waitedRustChild, "excluded");
  const included = inferInstructionScope({
    baseline: 100,
    instructions: 1100,
    threads: 2100,
    child: 1100,
  });
  assert.equal(included.waitedRustChild, "included");
  const missing = inferInstructionScope({
    baseline: null,
    instructions: 1100,
    threads: 2100,
    child: 1100,
  });
  assert.equal(missing.workerThreads, "unavailable");
  assert.equal(missing.waitedRustChild, "unavailable");
  const ambiguous = inferInstructionScope({
    baseline: 100,
    instructions: 1100,
    threads: 1100,
    child: 700,
  });
  assert.equal(ambiguous.workerThreads, "inconclusive");
  assert.equal(ambiguous.waitedRustChild, "inconclusive");
});
