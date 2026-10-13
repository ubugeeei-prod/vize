import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const [baseBin, headBin, output] = process.argv.slice(2);
assert(baseBin && headBin && output, "usage: doctor-filter-paired.mjs BASE_BIN HEAD_BIN OUTPUT");
for (const key of ["BASE_SHA", "HEAD_SHA"]) {
  assert.match(process.env[key] ?? "", /^[0-9a-f]{40}$/u);
}
const directory = `${output}.samples`;
mkdirSync(directory, { recursive: true });
const samples = { base: [], head: [] };

function run(side, pair) {
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
  const data = JSON.parse(readFileSync(file, "utf8"));
  assert.equal(data.sha, side === "base" ? process.env.BASE_SHA : process.env.HEAD_SHA);
  assert.equal(data.side, side);
  assert.equal(data.rows.length, 25);
  return data;
}

function assertParity(base, head) {
  const shape = (sample) =>
    sample.rows.map(({ elapsed_ns: _elapsed, compile_ns: _compile, ...row }) => row);
  assert.deepEqual(shape(head), shape(base), "Doctor verdicts or timed workload differ");
}

for (let warmup = 0; warmup < 2; warmup += 1) {
  assertParity(run("base", `warmup-${warmup}`), run("head", `warmup-${warmup}`));
}
for (let pair = 0; pair < 9; pair += 1) {
  const order = pair % 2 === 0 ? ["base", "head"] : ["head", "base"];
  const current = {};
  for (const side of order) current[side] = run(side, pair);
  assertParity(current.base, current.head);
  for (const side of ["base", "head"]) samples[side].push(current[side]);
}
function median(values) {
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
