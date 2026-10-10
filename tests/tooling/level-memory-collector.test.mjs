import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  collectFreshReports,
  validatePair,
  MEMORY_ENVIRONMENT,
} from "../../tools/benchmarks/scripts/level-memory-lib.mjs";
import { run } from "../../tools/benchmarks/scripts/level-memory-runner.mjs";
import * as memory from "../../tools/benchmarks/scripts/level-memory-lib.mjs";

const report = (id) => ({
  bench_id: id,
  fixture: `synthetic:${id}`,
  platform: "linux",
  wall_ns: { p50: 1, p95: 2 },
  allocs: 0,
  alloc_bytes_peak: 0,
  rss_peak_bytes: 0,
  harness_version: "1.0.0",
});
function fixture(t) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "level-memory-law-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const dir = path.join(root, "tools/benchmarks/results/davinci");
  fs.mkdirSync(dir, { recursive: true });
  const identities = new Map(
    ["one", "two"].map((id) => [
      id,
      {
        bench_id: id,
        fixture: `synthetic:${id}`,
        fixture_sha256: "a".repeat(64),
        window: "routine-return",
      },
    ]),
  );
  for (const id of identities.keys())
    fs.writeFileSync(path.join(dir, `${id}.json`), JSON.stringify(report(id)));
  const markers = [...identities.keys()]
    .map((id) => `davinci-harness: wrote ${path.join(dir, `${id}.json`)}`)
    .join("\n");
  return { root, dir, identities, markers };
}

test("fresh complete reports retain whole packets and original identities", (t) => {
  const f = fixture(t);
  const rows = collectFreshReports(f.root, f.markers, f.identities);
  assert.equal(rows.size, 2);
  assert.deepEqual(rows.get("one").report, report("one"));
  assert.equal(rows.get("one").identity.window, "routine-return");
  assert.match(rows.get("one").raw_sha256, /^[a-f0-9]{64}$/);
  validatePair(rows, rows);
  assert.ok(!Object.hasOwn(MEMORY_ENVIRONMENT, "VIZE_INSTRUCTION_COUNTS"));
});

test("tracked or cached reports without actual emitted writes are rejected", (t) => {
  const f = fixture(t);
  assert.throws(() => collectFreshReports(f.root, "", f.identities), /emitted|missing/);
  assert.throws(
    () => collectFreshReports(f.root, f.markers.split("\n")[0], f.identities),
    /missing/,
  );
});

test("duplicate, extra and foreign emitted reports cannot qualify", (t) => {
  const f = fixture(t);
  assert.throws(
    () => collectFreshReports(f.root, `${f.markers}\n${f.markers}`, f.identities),
    /duplicate/,
  );
  fs.writeFileSync(path.join(f.dir, "three.json"), JSON.stringify(report("three")));
  assert.throws(
    () =>
      collectFreshReports(
        f.root,
        `${f.markers}\ndavinci-harness: wrote ${path.join(f.dir, "three.json")}`,
        f.identities,
      ),
    /unregistered|extra/,
  );
  assert.throws(
    () =>
      collectFreshReports(
        f.root,
        `${f.markers}\ndavinci-harness: wrote /tmp/foreign.json`,
        f.identities,
      ),
    /outside|foreign/,
  );
});

test("null metrics, forged fields and wrong fixture identities remain red", (t) => {
  const f = fixture(t);
  for (const change of [
    { allocs: null },
    { alloc_bytes_peak: null },
    { rss_peak_bytes: null },
    { fixture: "synthetic:other" },
    { bench_id: "two" },
    { platform: "darwin" },
    { accepted: true },
    { wall_ns: { p50: 3, p95: 2 } },
  ]) {
    fs.writeFileSync(path.join(f.dir, "one.json"), JSON.stringify({ ...report("one"), ...change }));
    assert.throws(() => collectFreshReports(f.root, f.markers, f.identities));
  }
});

test("symlink output and changed same-byte source/window populations are rejected", (t) => {
  const f = fixture(t);
  const rows = collectFreshReports(f.root, f.markers, f.identities);
  const changed = structuredClone(rows);
  changed.get("one").identity.window = "stage-return";
  assert.throws(() => validatePair(rows, changed), /identity|window/);
  changed.get("one").identity.window = "routine-return";
  changed.get("one").identity.fixture_sha256 = "b".repeat(64);
  assert.throws(() => validatePair(rows, changed), /identity|fixture/);
  changed.delete("two");
  assert.throws(() => validatePair(rows, changed), /missing/);
  fs.unlinkSync(path.join(f.dir, "one.json"));
  fs.symlinkSync(path.join(f.dir, "two.json"), path.join(f.dir, "one.json"));
  assert.throws(() => collectFreshReports(f.root, f.markers, f.identities), /symlink/);
});

test("a failing process preserves complete raw streams before refusing admission", (t) => {
  const f = fixture(t);
  const directory = path.join(f.root, "failed-process");
  assert.throws(
    () =>
      run(
        process.execPath,
        [
          "-e",
          "process.stdout.write(Buffer.from([0,255,10]));process.stderr.write('whole stderr\\n');process.exit(7)",
        ],
        f.root,
        directory,
        { ...process.env, MEMORY_TEST_SECRET: "must-not-escape" },
      ),
    /exit 7/,
  );
  assert.deepEqual(fs.readFileSync(path.join(directory, "stdout.bin")), Buffer.from([0, 255, 10]));
  assert.equal(fs.readFileSync(path.join(directory, "stderr.bin"), "utf8"), "whole stderr\n");
  const receipt = JSON.parse(fs.readFileSync(path.join(directory, "process.json")));
  assert.equal(receipt.status, 7);
  assert.equal(receipt.signal, null);
  assert.deepEqual(receipt.imageBefore, receipt.imageAfter);
  assert.ok(!JSON.stringify(receipt).includes("must-not-escape"));
});

test("launch failures retain unknown status and actual resolution absence", (t) => {
  const f = fixture(t);
  const directory = path.join(f.root, "missing-process");
  assert.throws(
    () => run("missing-level-memory-image", [], f.root, directory, { PATH: f.root }),
    /spawn/,
  );
  const receipt = JSON.parse(fs.readFileSync(path.join(directory, "process.json")));
  assert.equal(receipt.status, null);
  assert.equal(receipt.imageBefore, null);
  assert.equal(receipt.error.code, "ENOENT");
  assert.equal(fs.readFileSync(path.join(directory, "stdout.bin")).length, 0);
});

test("normal measurements select Criterion benchmark sampling explicitly", () => {
  assert.deepEqual(memory.NORMAL_ARGUMENTS, ["--bench", "--quick"]);
  assert.ok(!Object.hasOwn(MEMORY_ENVIRONMENT, "VIZE_INSTRUCTION_COUNTS"));
});
