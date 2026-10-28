import assert from "node:assert/strict";
import { test } from "node:test";
import { verifyPartitionBenchLayout } from "../../tools/benchmarks/scripts/partition-weight-layout.ts";

void test("ordinary partition benchmark layout preserves bodies and rejects source drift", () => {
  const before = {
    main: '#[path = "partition_weight/baseline.rs"]\nmod baseline;\n#[path = "partition_weight/candidate.rs"]\nmod candidate;\nfn main() {}\n',
    children: ["baseline kernel", "candidate kernel", "source checker"],
  };
  const after = {
    main: "mod baseline;\nmod candidate;\nfn main() {}\n",
    children: [...before.children],
  };
  const manifest =
    '[[bench]]\nname = "partition_weight"\npath = "benches/partition_weight/main.rs"\nharness = false\n';
  assert.equal(verifyPartitionBenchLayout(before, after, manifest).bodyPreserved, true);
  for (const value of [
    { ...after, main: after.main + " " },
    { ...after, children: ["changed", ...after.children.slice(1)] },
    { ...after, children: after.children.slice(1) },
  ])
    assert.throws(() => verifyPartitionBenchLayout(before, value, manifest));
  assert.throws(() =>
    verifyPartitionBenchLayout(before, after, manifest.replace("/main.rs", "/other.rs")),
  );
  assert.throws(() =>
    verifyPartitionBenchLayout(
      {
        ...before,
        main: before.main.replace("mod baseline;", '#[path = "extra.rs"]\nmod baseline;'),
      },
      after,
      manifest,
    ),
  );
});
