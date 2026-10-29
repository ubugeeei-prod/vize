import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  clippyArgs,
  droppedRatchetCrates,
  parseRatchet,
  ratchetPath,
  touchedRatchetCrates,
} from "../../tools/support/compat/github/clippy-test-targets.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (file: string) => readFileSync(path.join(repoRoot, file), "utf8");

function workspacePackages(): Set<string> {
  const members = /^members = \[([\s\S]*?)\]/m.exec(read("Cargo.toml"))?.[1] ?? "";
  const names = new Set<string>();
  for (const [, member] of members.matchAll(/"([^"]+)"/g)) {
    const name = /^name = "([^"]+)"/m.exec(read(`${member}/Cargo.toml`))?.[1];
    assert.ok(name, `${member} names its package`);
    names.add(name);
  }
  return names;
}

void test("the checked-in ratchet lists sorted workspace crates", () => {
  const crates = parseRatchet(read(ratchetPath));
  assert.ok(crates.length > 0);
  const packages = workspacePackages();
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

void test("the ratchet only grows", () => {
  assert.deepEqual(droppedRatchetCrates(["a", "b"], ["a", "b", "c"]), []);
  assert.deepEqual(droppedRatchetCrates(["a", "b"], ["a"]), ["b"]);
});

void test("PR Clippy runs the ratchet and the plan job holds it", () => {
  const command = "node tools/support/compat/github/clippy-test-targets.ts";
  const prRust = read(".github/workflows/pr-rust-checks.yml");
  assert.equal(prRust.split(`${command} touched "$RUNNER_TEMP/rust-plan.json"`).length, 2);
  const prSource = read(".github/workflows/pr-source-checks.yml");
  assert.equal(prSource.split(`${command} hold "$BASE_SHA"`).length, 2);
});
