// Davinci P4-11a / P4-11b — TS-37 seeded HTML nesting classes.
//
// `seed-defects.rs --html-nesting` appends one snippet per checker class
// (and, where a `<script setup>` exists, the cross-component class) to every
// eligible template of the committed miniature set, lints the original and
// seeded trees with `vize lint --cross-file`, and asserts by identity: every
// injection found at its exact span, the one baseline finding moved through
// the insertions, nothing else. Corpus runs are a local concern (see
// davinci-road/plan/phase-4-records/p4-11a.md).

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const seedTool = path.join(root, "tools/commands/davinci/seed-defects.rs");
const fixtures = path.join(root, "tests/_fixtures/davinci-html-nesting");

function seed(env: NodeJS.ProcessEnv = process.env) {
  const out = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-html-nesting-"));
  process.on("exit", () => fs.rmSync(out, { recursive: true, force: true }));
  const args = [seedTool, "--html-nesting", "--fixtures", fixtures, "--out", out, "--assert"];
  const result = spawnSync("rust-script", args, { cwd: root, encoding: "utf8", env });
  if (result.error) throw result.error;
  return { result, out };
}

/// Every class id `ViolationClass::id` can return.
function checkerClasses(): string[] {
  const source = fs.readFileSync(
    path.join(root, "crates/vize_patina/src/html_content_model/class.rs"),
    "utf8",
  );
  const body = source.slice(source.indexOf("pub const fn id(self)"));
  const ids = body.slice(0, body.indexOf("\n    }\n")).matchAll(/=> "(?<id>[a-z-]+)"/gu);
  return [...ids].map((match) => match.groups!.id).sort();
}

const { result, out } = seed();

test("every nesting class and the cross-component class is detected by identity", () => {
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  const lines = result.stdout.trim().split("\n");
  assert.equal(
    lines[0],
    "scope-proof: files-scanned=4 html-eligible=2 composed-eligible=1 injections=45",
  );
  assert.equal(lines.at(-1), "assert: detected=45/45 baseline-mapped=1 unexpected=0 verdict=pass");
  const report = JSON.parse(fs.readFileSync(path.join(out, "html-report.json"), "utf8"));
  assert.deepEqual(report.scope.skipped, {
    "imported by another file": 1,
    "template has attributes": 1,
  });
  assert.deepEqual(report.misses, []);
  assert.deepEqual(report.unexpected, []);
  for (const recall of report.classes) {
    assert.equal(recall.detected, recall.expected, recall.class);
  }
});

test("the seeded classes are exactly the checker's classes plus the composed one", () => {
  const report = JSON.parse(fs.readFileSync(path.join(out, "html-report.json"), "utf8"));
  const seeded = report.classes.map((recall: { class: string }) => recall.class).sort();
  assert.deepEqual(seeded, [...checkerClasses(), "cross-component-nesting"].sort());
});

test("the assertion fails when the linter finds nothing (non-vacuous)", () => {
  const bin = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-html-nesting-bin-"));
  const fake = path.join(bin, "vize");
  fs.writeFileSync(fake, '#!/bin/sh\n[ "$1" = "--version" ] && echo vize && exit 0\necho "[]"\n');
  fs.chmodSync(fake, 0o755);
  try {
    const { result: silent, out: silentOut } = seed({ ...process.env, VIZE_BIN: fake });
    assert.equal(silent.status, 1, `${silent.stdout}\n${silent.stderr}`);
    const report = JSON.parse(fs.readFileSync(path.join(silentOut, "html-report.json"), "utf8"));
    assert.equal(report.misses.length, 45);
    assert.equal(report.verdict, "fail");
  } finally {
    fs.rmSync(bin, { recursive: true, force: true });
  }
});
