import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

function lineMatching(source: string, pattern: RegExp): string {
  const line = source.split("\n").find((line) => pattern.test(line));
  assert.ok(line, `missing line matching ${pattern}`);
  return line;
}

function taskSection(source: string, id: string): string {
  const start = new RegExp(`^## ${id} —`, "mu").exec(source)?.index;
  assert.notEqual(start, undefined, `missing ${id} task section`);
  const tail = source.slice(start);
  const next = /^## P2-/mu.exec(tail.slice(1))?.index;
  return next === undefined ? tail : tail.slice(0, next + 1);
}

test("P2-10 docs describe vue.css-bind spans as file-absolute", () => {
  const phase = readRepoFile("davinci-road", "plan", "phase-2.md");
  const records = readRepoFile("davinci-road", "plan", "phase-2-records.md");
  const tasks = readRepoFile("davinci-road", "plan", "phase-2-tasks.md");
  const p2_10 = readRepoFile("davinci-road", "plan", "phase-2-records", "p2-10.md");
  const folioFormat = readRepoFile("davinci-road", "plan", "folio-format.md");
  const entries = [
    lineMatching(phase, /P2-10.*vue\.css-bind/u),
    lineMatching(records, /P2-10.*vue\.css-bind/u),
    taskSection(tasks, "P2-10"),
    p2_10,
    lineMatching(folioFormat, /vue\.css-bind value=<expr>.*style-block content start/u),
  ];

  for (const entry of entries) {
    assert.match(entry, /file-absolute/u);
    if (entry.includes("vue.css-bind value=<expr>")) {
      assert.match(entry, /style-block content start/u);
    }
    assert.doesNotMatch(entry, /block-relative|to_block_relative/u);
  }
});

test("P2-10 implementation pins shifted file-absolute spans", () => {
  const regression = readRepoFile("crates", "vize_s1_to_s2", "tests", "css_bind_lowering.rs");

  assert.match(regression, /fn block_start_produces_file_absolute_spans\(\)/u);
  assert.match(regression, /ui\.element style @90:120/u);
  assert.match(regression, /vue\.css-bind value=js\(\\"color\\" @111:116\) @104:117/u);
});
