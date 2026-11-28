import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  allRustFeatureRows,
  rustFeatureCommands,
  rustFeatureLaneInputs,
  rustFeatureRow,
  rustFeatureRows,
  selectRustFeatureRows,
} from "../../tools/support/ci/rust-feature-tests.ts";
import {
  RUST_FEATURE_BATCH_SECONDS,
  rustFeatureBatches,
  rustFeatureJobRows,
  rustFeatureRowSeconds,
} from "../../tools/support/ci/rust-feature-batches.ts";

const root = new URL("../../", import.meta.url);
const workspaceMembers = /^members = \[([^\]]*)\]/mu
  .exec(readFileSync(new URL("Cargo.toml", root), "utf8"))![1]!
  .match(/"[^"]+"/gu)!
  .map((member) => {
    const manifest = readFileSync(new URL(`${member.slice(1, -1)}/Cargo.toml`, root), "utf8");
    return /^name = "([^"]+)"/mu.exec(manifest)![1]!;
  });

test("feature rows have unique names and run only tests, Clippy and checks", () => {
  const ids = rustFeatureRows.map((row) => row.id);
  assert.deepEqual(ids, [...new Set(ids)].sort());
  for (const row of rustFeatureRows) {
    assert.match(row.id, /^[a-z]+(?:-[a-z]+)*$/u);
    assert.ok(row.packages.length > 0, row.id);
    for (const command of row.commands) {
      assert.match(command, /^(?:test|clippy|check) /u, command);
      // Ignored tests read external corpora or measure; the lane never runs them.
      assert.doesNotMatch(command, /--(?:include-)?ignored|--nocapture/u, command);
    }
  }
});

test("row crates are workspace members", () => {
  for (const row of rustFeatureRows)
    for (const name of row.packages)
      assert.ok(workspaceMembers.includes(name), `${row.id}: ${name}`);
});

test("commands are locked, use the CI profile unless they name one, and deny Clippy warnings", () => {
  assert.deepEqual(
    rustFeatureCommands({
      id: "example",
      packages: ["vize_l0"],
      commands: [
        "clippy -p vize_l0 --features a --lib --no-deps",
        "test -p vize_l0 --features a --test t -- --test-threads=1",
        "test --profile ci-opt --manifest-path other/Cargo.toml",
      ],
    }),
    [
      [
        "clippy",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_l0",
        "--features",
        "a",
        "--lib",
        "--no-deps",
        "--",
        "-D",
        "warnings",
      ],
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_l0",
        "--features",
        "a",
        "--test",
        "t",
        "--",
        "--test-threads=1",
      ],
      ["test", "--locked", "--profile", "ci-opt", "--manifest-path", "other/Cargo.toml"],
    ],
  );
});

const matrixRows = (matrix: ReturnType<typeof allRustFeatureRows>) =>
  matrix.include.flatMap(({ rows }) => rustFeatureJobRows(rows));

test("changes select the rows whose crates they affect", () => {
  const rows = (paths: string[], packages?: string[]) =>
    matrixRows(selectRustFeatureRows(paths, packages)).sort();
  const every = rustFeatureRows.map((row) => row.id);
  assert.deepEqual(rows([]), every, "an unknown diff runs every row");
  for (const input of rustFeatureLaneInputs)
    assert.deepEqual(rows([input], []), every, `${input} runs every row`);
  assert.deepEqual(rows(["docs/guide.md"], undefined), []);
  assert.deepEqual(selectRustFeatureRows(["docs/guide.md"], undefined).include, []);
  assert.deepEqual(rows(["crates/vize_musea/src/lib.rs"], ["vize_musea"]), ["art-sources"]);
  assert.ok(rows(["davinci/vize_l0/src/lib.rs"], ["vize_l0"]).includes("profile-allocations"));
});

test("lane inputs name files that exist", () => {
  for (const input of rustFeatureLaneInputs)
    assert.doesNotThrow(() => readFileSync(new URL(input, root)), input);
});

test("only the Windows row leaves the Linux runner, in a job of its own", () => {
  const { include } = allRustFeatureRows();
  for (const { rows, runner } of include) {
    const ids = rustFeatureJobRows(rows);
    if (ids.includes("windows-project-paths")) {
      assert.deepEqual(ids, ["windows-project-paths"]);
      assert.equal(runner, "blacksmith-32vcpu-windows-2025");
    } else assert.equal(runner, "blacksmith-16vcpu-ubuntu-2404", rows);
  }
  assert.equal(rustFeatureRow("windows-project-paths").runtime, true);
  assert.throws(() => rustFeatureRow("missing"), /Unknown Rust feature row/u);
});

test("jobs partition the selected rows exactly and stay within the work budget", () => {
  const seconds = (id: string) => rustFeatureRowSeconds[id] ?? RUST_FEATURE_BATCH_SECONDS;
  const every = matrixRows(allRustFeatureRows());
  assert.deepEqual(
    [...every].sort(),
    rustFeatureRows.map((row) => row.id),
  );
  assert.equal(new Set(every).size, every.length);
  for (const { rows } of allRustFeatureRows().include) {
    const ids = rustFeatureJobRows(rows);
    assert.deepEqual(ids, [...ids].sort(), "a job lists its rows in declaration order");
    if (ids.length > 1)
      assert.ok(ids.reduce((sum, id) => sum + seconds(id), 0) <= RUST_FEATURE_BATCH_SECONDS, rows);
  }
  // Packing only shares setup: the full Linux set needs about its total work over the budget.
  const linux = rustFeatureRows.filter((row) => !row.windows);
  const total = linux.reduce((sum, row) => sum + seconds(row.id), 0);
  assert.ok(rustFeatureBatches(linux).length <= Math.ceil(total / RUST_FEATURE_BATCH_SECONDS) + 1);
  assert.deepEqual(rustFeatureBatches([]), []);
  const single = rustFeatureBatches([rustFeatureRow("scss-products")]);
  assert.deepEqual(
    single.map((job) => job.map((row) => row.id)),
    [["scss-products"]],
  );
});

test("every row estimate names a Linux row", () => {
  const linux = new Set(rustFeatureRows.filter((row) => !row.windows).map((row) => row.id));
  for (const [id, value] of Object.entries(rustFeatureRowSeconds)) {
    assert.ok(linux.has(id), id);
    assert.ok(Number.isInteger(value) && value > 0 && value <= RUST_FEATURE_BATCH_SECONDS, id);
  }
});
