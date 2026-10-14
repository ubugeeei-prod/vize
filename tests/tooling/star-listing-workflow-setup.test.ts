import assert from "node:assert/strict";
import { test } from "node:test";
import { stringify } from "yaml";
import { assertStarListingWorkflowSetup } from "../../tools/support/compat/github/empty-vue-star-listing-source.mjs";

test("current Star workflow setup retains commands, conditions, matrix and path triggers", () => {
  const file = ".github/workflows/typechecker-native-phases.yml";
  const original = {
    on: {
      pull_request: { paths: ["tools/benchmarks/scripts/typecheck-command.mjs", "crates/**"] },
    },
    jobs: {
      benchmark: {
        strategy: { matrix: { fixture: ["original", "sdk"] } },
        steps: [
          {
            uses: "dtolnay/rust-toolchain@7e38f4b43b4db5c8dd498af069a4f6196df1d067",
            with: { toolchain: "stable" },
          },
          {
            uses: "wild-linker/action@0bbbfa5df4380cab8e63cb8505a1ce65e1d10203",
            with: { "wild-version": "0.9.0" },
            if: "runner.os == 'Linux'",
          },
          { run: "cargo bench --locked", env: { RUSTFLAGS: "-C debuginfo=0" } },
        ],
      },
    },
  };
  const current = structuredClone(original);
  current.jobs.benchmark.steps[0].with = { toolchain: "1.99.0" };
  current.jobs.benchmark.steps[1].uses = "./.github/actions/setup-mold";
  delete current.jobs.benchmark.steps[1].with;
  current.on.pull_request.paths.splice(
    1,
    0,
    "tools/benchmarks/scripts/type-snapshot-parser-recovery.ts",
  );
  assertStarListingWorkflowSetup(file, stringify(original), stringify(original));
  assertStarListingWorkflowSetup(file, stringify(original), stringify(current));
  for (const [from, to] of [
    ["setup-mold", "setup-other"],
    ["1.99.0", "1.98.0"],
    ["cargo bench --locked", "cargo bench"],
    ["runner.os == 'Linux'", "true"],
    ["debuginfo=0", "debuginfo=1"],
    ["crates/**", "other/**"],
    ["sdk", "other"],
  ])
    assert.throws(
      () =>
        assertStarListingWorkflowSetup(
          file,
          stringify(original),
          stringify(current).replace(from, to),
        ),
      { code: "ERR_ASSERTION" },
    );
});
