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

test("changes select the rows whose crates they affect", () => {
  const rows = (paths: string[], packages?: string[]) =>
    selectRustFeatureRows(paths, packages).include.map(({ row }) => row);
  const every = allRustFeatureRows().include.map(({ row }) => row);
  assert.deepEqual(rows([]), every, "an unknown diff runs every row");
  for (const input of rustFeatureLaneInputs)
    assert.deepEqual(rows([input], []), every, `${input} runs every row`);
  assert.deepEqual(rows(["docs/guide.md"], undefined), []);
  assert.deepEqual(rows(["crates/vize_musea/src/lib.rs"], ["vize_musea"]), ["art-sources"]);
  assert.ok(rows(["davinci/vize_l0/src/lib.rs"], ["vize_l0"]).includes("profile-allocations"));
});

test("only the Windows row leaves the Linux runner", () => {
  const runners = new Map(allRustFeatureRows().include.map(({ row, runner }) => [row, runner]));
  for (const [row, runner] of runners)
    assert.equal(
      runner,
      row === "windows-project-paths"
        ? "blacksmith-32vcpu-windows-2025"
        : "blacksmith-32vcpu-ubuntu-2404",
    );
  assert.equal(rustFeatureRow("windows-project-paths").runtime, true);
  assert.throws(() => rustFeatureRow("missing"), /Unknown Rust feature row/u);
});
