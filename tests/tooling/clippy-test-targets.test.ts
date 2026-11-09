import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  clippyArgs,
  droppedRatchetCrates,
  workspacePackages,
  parseRatchet,
  ratchetPath,
  touchedRatchetCrates,
} from "../../tools/support/compat/github/clippy-test-targets.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (file: string) => readFileSync(path.join(repoRoot, file), "utf8");

void test("the checked-in ratchet lists sorted workspace crates", () => {
  const crates = parseRatchet(read(ratchetPath));
  assert.ok(crates.length > 0);
  const packages = workspacePackages(repoRoot);
  assert.deepEqual(
    crates.filter((name) => !packages.has(name)),
    [],
  );
});

void test("the ratchet rejects unsorted, duplicate and malformed entries", () => {
  assert.deepEqual(parseRatchet('{"crates":["a","b"]}'), ["a", "b"]);
  for (const text of [
    '{"crates":["b","a"]}',
    '{"crates":["a","a"]}',
    '{"crates":["a b"]}',
    '{"crates":"a"}',
    "[]",
  ])
    assert.throws(() => parseRatchet(text), Error, text);
});

void test("PR lanes lint only the listed crates that own a changed file", () => {
  const ratchet = ["vize_carton", "vize_croquis", "vize_relief"];
  assert.deepEqual(
    touchedRatchetCrates({ changedPackages: ["vize", "vize_relief", "vize_carton"] }, ratchet),
    ["vize_carton", "vize_relief"],
  );
  assert.deepEqual(touchedRatchetCrates({ changedPackages: ["vize"] }, ratchet), []);
  assert.throws(() => touchedRatchetCrates({}, ratchet), /changedPackages/);
  assert.deepEqual(clippyArgs(["vize_carton", "vize_relief"]), [
    "clippy",
    "--package",
    "vize_carton",
    "--package",
    "vize_relief",
    "--all-targets",
    "--profile",
    "ci",
    "--",
    "-D",
    "warnings",
  ]);
});

void test("the ratchet only grows while its crates exist", () => {
  const workspace = new Set(["a", "b", "c"]);
  assert.deepEqual(droppedRatchetCrates(["a", "b"], ["a", "b", "c"], workspace), []);
  assert.deepEqual(droppedRatchetCrates(["a", "b"], ["a"], workspace), ["b"]);
  // A renamed or deleted crate leaves the list with nothing to lint.
  assert.deepEqual(droppedRatchetCrates(["a", "b"], ["a", "c"], new Set(["a", "c"])), []);
  const packages = workspacePackages(repoRoot);
  assert.ok(packages.has("vize_carton") && packages.has("poc-support"));
});

void test("PR Clippy runs the ratchet and the plan job holds it", () => {
  const command = "node tools/support/compat/github/clippy-test-targets.ts";
  const prRust = read(".github/workflows/pr-rust-checks.yml");
  assert.equal(prRust.split(`${command} touched "$RUNNER_TEMP/rust-plan.json"`).length, 2);
  const prSource = read(".github/workflows/pr-source-checks.yml");
  assert.equal(prSource.split(`${command} hold "$BASE_SHA"`).length, 2);
});
