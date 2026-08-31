import assert from "node:assert/strict";
import { test } from "node:test";

import { selectTypecheckPerformanceProjects } from "../../legacy-tools/fixtures/typecheck-performance-shard.mjs";

const registry = {
  projects: [
    { id: "one", typecheckPerformance: { enabled: true } },
    { id: "two", typecheckPerformance: { enabled: false } },
    { id: "three", typecheckPerformance: { enabled: true } },
  ],
};

test("typecheck performance shard selector rejects invalid shard arguments", () => {
  for (const [args, message] of [
    [{ shardIndex: 0, shardCount: undefined }, /shard count must be a positive integer/],
    [{ shardIndex: 0, shardCount: 0 }, /shard count must be a positive integer/],
    [{ shardIndex: 0, shardCount: 1.5 }, /shard count must be a positive integer/],
    [{ shardIndex: undefined, shardCount: 2 }, /shard index must be in \[0, 2\)/],
    [{ shardIndex: -1, shardCount: 2 }, /shard index must be in \[0, 2\)/],
    [{ shardIndex: 2, shardCount: 2 }, /shard index must be in \[0, 2\)/],
  ] as const) {
    assert.throws(() => selectTypecheckPerformanceProjects(registry, args), message);
  }
});

test("typecheck performance shard selector allows valid empty selections", () => {
  assert.deepEqual(
    selectTypecheckPerformanceProjects(registry, { shardIndex: 1, shardCount: 2 }),
    [],
  );
});
