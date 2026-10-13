import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

import { isRecord } from "./script-types.ts";

type Side = "base" | "head";
/** One filter row from the measured binary; timings are integer nanoseconds. */
type DoctorRow = Record<string, unknown> & { id: string; elapsed_ns: number; compile_ns: number };
type DoctorSample = { sha: string; side: string; rows: DoctorRow[] };

function isDoctorSample(value: unknown): value is DoctorSample {
  return (
    isRecord(value) &&
    typeof value.sha === "string" &&
    typeof value.side === "string" &&
    Array.isArray(value.rows) &&
    value.rows.every(
      (row: unknown) =>
        isRecord(row) &&
        typeof row.id === "string" &&
        typeof row.elapsed_ns === "number" &&
        typeof row.compile_ns === "number",
    )
  );
}

const [baseBin, headBin, output] = process.argv.slice(2);
assert(baseBin && headBin && output, "usage: doctor-filter-paired.ts BASE_BIN HEAD_BIN OUTPUT");
for (const key of ["BASE_SHA", "HEAD_SHA"]) {
  assert.match(process.env[key] ?? "", /^[0-9a-f]{40}$/u);
}
const directory = `${output}.samples`;
mkdirSync(directory, { recursive: true });
const samples: Record<Side, DoctorSample[]> = { base: [], head: [] };

function run(side: Side, pair: number | string): DoctorSample {
  const file = path.resolve(directory, `${side}-${pair}.json`);
  const result = spawnSync(side === "base" ? baseBin : headBin, [file], {
    encoding: "utf8",
    timeout: 120_000,
    env: {
      ...process.env,
      MEASURE_SIDE: side,
      MEASURE_SHA: side === "base" ? process.env.BASE_SHA : process.env.HEAD_SHA,
    },
  });
  assert.equal(result.error, undefined, `${side}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${side}: ${result.stderr}`);
  const data: unknown = JSON.parse(readFileSync(file, "utf8"));
  assert.ok(isDoctorSample(data), `${side}: malformed Doctor filter sample ${file}`);
  assert.equal(data.sha, side === "base" ? process.env.BASE_SHA : process.env.HEAD_SHA);
  assert.equal(data.side, side);
  assert.equal(data.rows.length, 25);
  return data;
}

function assertParity(base: DoctorSample, head: DoctorSample): void {
  const shape = (sample: DoctorSample) =>
    sample.rows.map(({ elapsed_ns: _elapsed, compile_ns: _compile, ...row }) => row);
  assert.deepEqual(shape(head), shape(base), "Doctor verdicts or timed workload differ");
}

for (let warmup = 0; warmup < 2; warmup += 1) {
  assertParity(run("base", `warmup-${warmup}`), run("head", `warmup-${warmup}`));
}
for (let pair = 0; pair < 9; pair += 1) {
  const order: Side[] = pair % 2 === 0 ? ["base", "head"] : ["head", "base"];
  const current: Partial<Record<Side, DoctorSample>> = {};
  for (const side of order) current[side] = run(side, pair);
  assertParity(current.base!, current.head!);
  for (const side of ["base", "head"] as const) samples[side].push(current[side]!);
}
function median(values: number[]): number {
  return [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
}
const rows = samples.head[0].rows.map((row, index) => {
  const baseNs = samples.base.map((sample) => sample.rows[index].elapsed_ns);
  const headNs = samples.head.map((sample) => sample.rows[index].elapsed_ns);
  const baseCompile = samples.base.map((sample) => sample.rows[index].compile_ns);
  const headCompile = samples.head.map((sample) => sample.rows[index].compile_ns);
  return {
    id: row.id,
    patterns: row.patterns,
    candidates: row.candidates,
    iterations: row.iterations,
    matched: row.matched,
    base_median_ns: median(baseNs),
    head_median_ns: median(headNs),
    ratio: median(headNs) / median(baseNs),
    base_samples_ns: baseNs,
    head_samples_ns: headNs,
    base_compile_median_ns: median(baseCompile),
    head_compile_median_ns: median(headCompile),
  };
});
const report = {
  schema: 1,
  base_sha: process.env.BASE_SHA,
  head_sha: process.env.HEAD_SHA,
  pairs: 9,
  warmups: 2,
  platform: process.platform,
  arch: process.arch,
  rows,
};
writeFileSync(output, `${JSON.stringify(report, null, 2)}\n`);
for (const row of rows) console.log(`${row.id}: ${row.ratio.toFixed(4)}`);
