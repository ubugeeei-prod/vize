import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const [baseBin, headBin, output] = process.argv.slice(2);
assert(baseBin && headBin && output, "usage: type-snapshot-paired.mjs BASE_BIN HEAD_BIN OUTPUT");
for (const key of ["BASE_SHA", "HEAD_SHA"]) assert.match(process.env[key] ?? "", /^[0-9a-f]{40}$/u);
const directory = `${output}.samples`;
mkdirSync(directory, { recursive: true });
const samples = { base: [], head: [] };

function run(side, pair) {
  const file = resolve(directory, `${side}-${pair}.json`);
  const result = spawnSync(side === "base" ? baseBin : headBin, [file], {
    encoding: "utf8",
    timeout: 120_000,
    env: {
      ...process.env,
      TYPE_SNAPSHOT_FIXTURE_DIR: resolve(directory, "fixtures"),
      MEASURE_SIDE: side,
      MEASURE_SHA: side === "base" ? process.env.BASE_SHA : process.env.HEAD_SHA,
    },
  });
  assert.equal(result.error, undefined, `${side}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${side}: ${result.stderr}`);
  const data = JSON.parse(readFileSync(file, "utf8"));
  assert.equal(data.sha, side === "base" ? process.env.BASE_SHA : process.env.HEAD_SHA);
  assert.equal(data.side, side);
  assert.equal(data.rows.length, 12);
  for (const row of data.rows) {
    assert(Number.isFinite(row.elapsed_ns) && row.elapsed_ns > 0);
    assert.match(row.signature, /^[0-9a-f]{64}$/u);
  }
  return data;
}

function assertParity(base, head) {
  const shape = (sample) => sample.rows.map(({ elapsed_ns: _elapsed, ...row }) => row);
  assert.deepEqual(shape(head), shape(base), "complete type-world facts or workloads differ");
}
for (let pass = 0; pass < 2; pass++) assertParity(run("base", `warmup-${pass}`), run("head", `warmup-${pass}`));
for (let pair = 0; pair < 9; pair++) {
  const current = {};
  for (const side of pair % 2 === 0 ? ["base", "head"] : ["head", "base"]) current[side] = run(side, pair);
  assertParity(current.base, current.head);
  for (const side of ["base", "head"]) samples[side].push(current[side]);
}
function median(values) { return [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)]; }
const rows = samples.head[0].rows.map((row, index) => {
  const base = samples.base.map((sample) => sample.rows[index].elapsed_ns);
  const head = samples.head.map((sample) => sample.rows[index].elapsed_ns);
  return { ...row, elapsed_ns: undefined, base_median_ns: median(base), head_median_ns: median(head),
    ratio: median(head) / median(base), base_samples_ns: base, head_samples_ns: head };
});
writeFileSync(output, `${JSON.stringify({ schema: 1, base_sha: process.env.BASE_SHA,
  head_sha: process.env.HEAD_SHA, pairs: 9, warmups: 2, platform: process.platform, arch: process.arch, rows }, null, 2)}\n`);
for (const row of rows) console.log(`${row.id}: ${(1 / row.ratio).toFixed(2)}x`);
