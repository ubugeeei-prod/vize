import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";

const fixtures = [
  { rows: 0, declaration: "disk" },
  { rows: 1, declaration: "disk" },
  { rows: 32, declaration: "disk" },
  { rows: 128, declaration: "disk" },
  { rows: 0, declaration: "open" },
] as const;
const metrics = [
  "warm_ns",
  "warm_allocations",
  "warm_peak",
  "cold_ns",
  "cold_allocations",
  "cold_peak",
  "retained_with_response",
  "retained_after_drop",
] as const;
type Metric = (typeof metrics)[number];
type Sample = Record<Metric, number> & { rows: number; declaration: "disk" | "open" };
type Pair = { base: Sample[]; head: Sample[] };

const [base, head, output] = process.argv.slice(2);
assert(base && head && output, "Expected base and head test executables and a JSON output path");

function measure(binary: string): Sample[] {
  const log = execFileSync(
    binary,
    [
      "repeated_native_reads_preserve_every_mapped_diagnostic",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ],
    { encoding: "utf8" },
  );
  assert.match(log, /test result: ok\. 1 passed; 0 failed;/);
  const samples = Array.from(
    log.matchAll(/ambient_sample (\{[^\n]+\})/g),
    ([, json]) => JSON.parse(json) as Sample,
  );
  assert.deepEqual(
    samples.map(({ rows, declaration }) => ({ rows, declaration })),
    fixtures,
  );
  for (const sample of samples) {
    for (const key of metrics) {
      assert(
        Number.isSafeInteger(sample[key]) && sample[key] >= 0,
        `Invalid ${key} for ${sample.rows} rows`,
      );
    }
    assert(sample.warm_ns > 0 && sample.warm_allocations > 0 && sample.cold_peak > 0);
  }
  return samples;
}

function collect(reverse: boolean): Pair[] {
  return Array.from({ length: 31 }, (_, index) => {
    if ((index % 2 === 0) !== reverse) return { base: measure(base), head: measure(head) };
    const candidate = measure(head);
    return { base: measure(base), head: candidate };
  });
}

function median(values: number[]): number {
  return values.toSorted((a, b) => a - b)[Math.floor(values.length / 2)];
}

function summarize(pairs: Pair[]) {
  return fixtures.flatMap(({ rows, declaration }, index) =>
    metrics.map((metric) => ({
      rows,
      declaration,
      metric,
      base: median(pairs.map((pair) => pair.base[index][metric])),
      head: median(pairs.map((pair) => pair.head[index][metric])),
      higher: pairs.filter((pair) => pair.head[index][metric] > pair.base[index][metric]).length,
    })),
  );
}

// Use the existing paired sampler's 23-of-31 sign rule. A second series starts
// on the opposite side and confirms the same metric at the same cardinality.
const first = collect(false);
const summary = summarize(first);
const suspects = summary.filter((metric) => metric.higher >= 23);
const confirmation = suspects.length ? collect(true) : [];
const confirmed = confirmation.length
  ? summarize(confirmation).filter(
      (metric) =>
        metric.higher >= 23 &&
        suspects.some(
          (first) =>
            first.rows === metric.rows &&
            first.declaration === metric.declaration &&
            first.metric === metric.metric,
        ),
    )
  : [];
writeFileSync(output, JSON.stringify({ summary, first, confirmation, confirmed }) + "\n");
console.log("| Diagnostics | Declaration | Metric | Base median | Head median | Higher pairs |");
console.log("| ---: | --- | --- | ---: | ---: | ---: |");
for (const row of summary) {
  console.log(
    `| ${row.rows} | ${row.declaration} | ${row.metric} | ${row.base} | ${row.head} | ${row.higher}/31 |`,
  );
}
assert.deepEqual(confirmed, [], "Reproducible increase in native diagnostic cost");
