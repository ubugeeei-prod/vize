import assert from "node:assert/strict";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  assertTwin,
  collectInstructionSuite,
  foldRuns,
  measurePair,
  movedRows,
  renderReport,
  validateSha,
  type Side,
} from "./release-measure.ts";

const methodology = { profile: "release-profile-test" };
const row = (instructions: number) => ({
  instructions,
  fixture_sha256: "a".repeat(64),
  window: "routine-return",
  methodology,
});
const side = (rows: Record<string, number>, cli = 100, napi = 50, bench = "a"): Side => ({
  rows: Object.fromEntries(
    Object.entries(rows).map(([id, instructions]) => [id, row(instructions)]),
  ),
  sizes: { "`vize` CLI (stripped)": cli, "`vize_vitrine` N-API library": napi },
  binaries: { "vize_armature/davinci": bench.repeat(64) },
});

void test("foldRuns keeps one count per row and rejects disagreeing executions", () => {
  const identified = (instructions: number) => ({
    instructions,
    fixture_sha256: "a".repeat(64),
    window: "routine-return",
  });
  const run = { a: identified(5), b: identified(7) };
  assert.deepEqual(foldRuns({ methodology, runs: [run, run, run] }), {
    a: row(5),
    b: row(7),
  });
  assert.throws(
    () => foldRuns({ methodology, runs: [run, { a: identified(6), b: identified(7) }] }),
    /disagree/,
  );
});

void test("movedRows lists rows that differ or exist on one side only", () => {
  assert.deepEqual(movedRows(side({ a: 1, b: 2 }), side({ a: 1, b: 3, c: 4 })), ["b", "c"]);
});

void test("assertTwin requires reproducible counts and binary sizes", () => {
  const primary = side({ a: 1, b: 2 });
  assert.doesNotThrow(() => assertTwin("base", primary, side({ a: 1, b: 2 })));
  assert.throws(() => assertTwin("base", primary, side({ a: 1, b: 3 })), /instruction counts/);
  assert.throws(() => assertTwin("base", primary, side({ a: 1, b: 2 }, 101)), /binary sizes/);
});

void test("assertTwin requires byte-identical benchmark executables", () => {
  const primary = side({ a: 1 });
  assert.throws(
    () => assertTwin("head", primary, side({ a: 1 }, 100, 50, "b")),
    /head benchmark executables are not byte-identical/,
  );
});

void test("instruction comparison rejects a changed workload identity", () => {
  const changed = side({ a: 1 });
  changed.rows.a!.fixture_sha256 = "b".repeat(64);
  assert.throws(() => movedRows(side({ a: 1 }), changed), /workload identity changed/);
});

void test("validateSha accepts only exact lowercase commit IDs", () => {
  assert.doesNotThrow(() => validateSha("BASE_SHA", "a".repeat(40)));
  assert.throws(() => validateSha("BASE_SHA", "a".repeat(39)), /exact commit SHA/);
  assert.throws(() => validateSha("BASE_SHA", "A".repeat(40)), /exact commit SHA/);
});

void test("renderReport tables every moved row, the identical rest, A/A and sizes", () => {
  const base = side({ a: 1_000, b: 2_000, c: 3_000 }, 1_000, 500);
  const head = side({ a: 990, b: 2_000, c: 3_003 }, 996, 500);
  const markdown = renderReport({
    baseSha: "a".repeat(40),
    headSha: "b".repeat(40),
    base,
    head,
    baseTwin: base,
    headTwin: head,
  });
  assert.match(markdown, /\| `a` \| 1,000 \| 990 \| −10 \(−1\.000%\) \| 1,000 \| 990 \|/);
  assert.match(markdown, /\| `c` \| 3,000 \| 3,003 \| \+3 \(\+0\.100%\) \|/);
  assert.match(markdown, /\| other 1 rows \| \| \| 0 \(identical\) \| \| \|/);
  assert.match(markdown, /\*\*2 moved, 1 up, 1 identical\*\* of 3 rows\./);
  assert.match(
    markdown,
    /A\/A: base vs base twin 0 moved \(byte-identical bench executables\); head vs head twin 0 moved \(byte-identical bench executables\); twin pair repeats every Δ above\./,
  );
  assert.match(markdown, /\| `vize` CLI \(stripped\) \| 1,000 \| 996 \| −4 \(−0\.400%\) \|/);
  assert.match(markdown, /\| `vize_vitrine` N-API library \| 500 \| 500 \| 0 \|/);
});

void test("renderReport works without twins", () => {
  const markdown = renderReport({
    baseSha: "a".repeat(40),
    headSha: "a".repeat(40),
    base: side({ a: 1 }),
    head: side({ a: 1 }),
  });
  assert.match(markdown, /\*\*0 moved, 0 up, 1 identical\*\* of 1 rows\./);
  assert.doesNotMatch(markdown, /A\/A:/);
});

void test("renderReport compares reproducible added and removed rows with twins", () => {
  const base = side({ kept: 1, removed: 2 });
  const head = side({ kept: 1, added: 3 });
  const markdown = renderReport({
    baseSha: "a".repeat(40),
    headSha: "b".repeat(40),
    base,
    head,
    baseTwin: side({ kept: 1, removed: 2 }),
    headTwin: side({ kept: 1, added: 3 }),
  });
  assert.match(markdown, /\| `added` \| — \| 3 \| added\/removed \| — \| 3 \|/);
  assert.match(markdown, /\| `removed` \| 2 \| — \| added\/removed \| 2 \| — \|/);
  assert.match(markdown, /\*\*2 moved, 1 up, 1 identical\*\* of 3 rows\./);
  assert.match(markdown, /twin pair repeats every Δ above/);
});

void test("measurePair never overwrites an existing pair", () => {
  const root = mkdtempSync(join(tmpdir(), "release-measure-"));
  const out = join(root, "out");
  const measured = side({ a: 1 });
  const fakeMeasure = (_source: string, target: string, label: string) => {
    const directory = join(target, label);
    mkdirSync(directory, { recursive: true });
    writeFileSync(join(directory, "side.json"), `${JSON.stringify(measured)}\n`);
    return measured;
  };
  try {
    measurePair(
      "base",
      "head",
      out,
      root,
      false,
      fakeMeasure,
      (dir) => (dir === "base" ? "a".repeat(40) : "b".repeat(40)),
      () => "1700000000",
    );
    const first = readFileSync(join(out, "pair.json"), "utf8");
    assert.throws(
      () =>
        measurePair(
          "new-base",
          "new-head",
          out,
          root,
          false,
          fakeMeasure,
          () => "c".repeat(40),
          () => "1700000000",
        ),
      /refusing to reuse measurement output/,
    );
    assert.equal(readFileSync(join(out, "pair.json"), "utf8"), first);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

void test("measurePair builds base, head and twins once each with the base commit clock", () => {
  const root = mkdtempSync(join(tmpdir(), "release-measure-"));
  const out = join(root, "out");
  const builds: string[] = [];
  const fakeMeasure = (
    source: string,
    target: string,
    label: string,
    _scratch: string,
    epoch: string,
  ) => {
    builds.push(`${label}:${source}:${epoch}`);
    const measured = side({ a: 1 });
    const directory = join(target, label);
    mkdirSync(directory, { recursive: true });
    writeFileSync(join(directory, "side.json"), `${JSON.stringify(measured)}\n`);
    return measured;
  };
  try {
    measurePair(
      "base",
      "head",
      out,
      root,
      true,
      fakeMeasure,
      (dir) => (dir === "base" ? "a".repeat(40) : "b".repeat(40)),
      (dir) => (dir === "base" ? "1700000000" : "1800000000"),
    );
    assert.deepEqual(builds, [
      "base:base:1700000000",
      "head:head:1700000000",
      "base-twin:base:1700000000",
      "head-twin:head:1700000000",
    ]);
    assert.equal(
      JSON.parse(readFileSync(join(out, "pair.json"), "utf8")).sourceDateEpoch,
      "1700000000",
    );
    assert.match(
      readFileSync(join(out, "summary.md"), "utf8"),
      /Every build uses `SOURCE_DATE_EPOCH=1700000000` \(the base commit time\)\./,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

void test("instruction captures share one execution path and retain each original side", () => {
  const root = mkdtempSync(join(tmpdir(), "release-capture-"));
  try {
    const paths: string[] = [];
    for (const label of ["base", "head", "base-twin", "head-twin"]) {
      const destination = join(root, "out", label, "formatter", "measurement");
      const dir = collectInstructionSuite(
        "/source",
        join(root, "scratch"),
        destination,
        ["--formatter"],
        {},
        (executable, args) => {
          assert.equal(executable, "node");
          assert.equal(args[1], "--formatter");
          const capture = args.at(-1)!;
          paths.push(capture);
          mkdirSync(capture, { recursive: true });
          writeFileSync(join(capture, "measurement.json"), JSON.stringify({ label }));
        },
      );
      assert.equal(dir, destination);
    }
    assert.equal(new Set(paths).size, 1);
    for (const label of ["base", "head", "base-twin", "head-twin"]) {
      assert.deepEqual(
        JSON.parse(
          readFileSync(
            join(root, "out", label, "formatter", "measurement", "measurement.json"),
            "utf8",
          ),
        ),
        { label },
      );
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

void test("failed collection retains partial evidence, propagates failure and empties the capture", () => {
  const root = mkdtempSync(join(tmpdir(), "release-capture-failure-"));
  const scratch = join(root, "scratch");
  const failure = new Error("instruction ceiling exceeded");
  try {
    for (const partial of [false, true]) {
      const destination = join(root, "out", String(partial));
      assert.throws(
        () =>
          collectInstructionSuite("/source", scratch, destination, [], {}, (_executable, args) => {
            if (partial) {
              const dir = args.at(-1)!;
              mkdirSync(dir, { recursive: true });
              writeFileSync(join(dir, "measurement.json"), "partial evidence");
            }
            throw failure;
          }),
        (error) => error === failure,
      );
      assert.equal(existsSync(join(scratch, "instruction-capture")), false);
      assert.equal(existsSync(destination), partial);
      if (partial)
        assert.equal(
          readFileSync(join(destination, "measurement.json"), "utf8"),
          "partial evidence",
        );
    }
    const destination = join(root, "occupied");
    mkdirSync(destination);
    writeFileSync(join(destination, "measurement.json"), "original evidence");
    assert.throws(
      () =>
        collectInstructionSuite("/source", scratch, destination, [], {}, (_executable, args) => {
          const dir = args.at(-1)!;
          mkdirSync(dir, { recursive: true });
          writeFileSync(join(dir, "measurement.json"), "partial evidence");
          throw failure;
        }),
      (error) =>
        error instanceof AggregateError && error.errors.length === 2 && error.errors[0] === failure,
    );
    assert.equal(readFileSync(join(destination, "measurement.json"), "utf8"), "original evidence");
    assert.equal(existsSync(join(scratch, "instruction-capture")), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
