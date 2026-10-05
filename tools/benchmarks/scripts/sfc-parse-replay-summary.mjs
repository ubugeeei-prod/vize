import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { BASE_SHA, CASES, HEAD_SHA, SETTINGS } from "./sfc-parse-replay-contract.mjs";

export function median(values) {
  assert.ok(values.length > 0 && values.every((value) => Number.isFinite(value) && value > 0));
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

export function pairedInterval(ratios, seed = 6189) {
  assert.equal(ratios.length, SETTINGS.pairs);
  let state = seed;
  const randomIndex = () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return Math.floor(((state >>> 0) / 0x1_0000_0000) * ratios.length);
  };
  const draws = Array.from({ length: 10_000 }, () =>
    median(Array.from({ length: ratios.length }, () => ratios[randomIndex()])),
  ).sort((left, right) => left - right);
  return { lower: draws[249], upper: draws[9749], confidence: 0.95, resamples: draws.length, seed };
}

export function summarizeRunner(report) {
  assert.equal(report.baseSha, BASE_SHA);
  assert.equal(report.headSha, HEAD_SHA);
  assert.deepEqual(report.settings, SETTINGS);
  assert.equal(report.observations.length, SETTINGS.pairs * CASES.length * 2);
  const rows = CASES.map((name) => {
    const pairs = Array.from({ length: SETTINGS.pairs }, (_, pair) => {
      const observations = report.observations.filter(
        (row) => row.name === name && row.pair === pair,
      );
      assert.equal(observations.length, 2, `missing or duplicate ${name} pair ${pair}`);
      const base = observations.find((row) => row.side === "base");
      const head = observations.find((row) => row.side === "head");
      assert.ok(base && head);
      const expected = pair % 2 === 0 ? ["base", "head"] : ["head", "base"];
      assert.deepEqual(
        observations.map((row) => row.side),
        expected,
        "pair order drift",
      );
      const ratio = head.estimates.median.point_estimate / base.estimates.median.point_estimate;
      assert.ok(Number.isFinite(ratio) && ratio > 0);
      return {
        pair,
        order: expected.join("/"),
        ratio,
        baseMedianNs: base.estimates.median.point_estimate,
        headMedianNs: head.estimates.median.point_estimate,
      };
    });
    const ratios = pairs.map((pair) => pair.ratio);
    return {
      name,
      pairs,
      medianPairedRatio: median(ratios),
      observedRange: [Math.min(...ratios), Math.max(...ratios)],
      conditionalPairedInterval: pairedInterval(ratios),
      baseFirstMedian: median(
        pairs.filter((pair) => pair.order === "base/head").map((pair) => pair.ratio),
      ),
      headFirstMedian: median(
        pairs.filter((pair) => pair.order === "head/base").map((pair) => pair.ratio),
      ),
    };
  });
  return {
    schema: 1,
    runner: report.runner,
    baseSha: BASE_SHA,
    headSha: HEAD_SHA,
    rows,
    limitation:
      "Percentile intervals resample six whole adjacent pairs, conditional on this runner and these binaries. They exclude build/layout and between-runner uncertainty. Criterion iterations are not independent A/B runs; observed ranges are not confidence intervals. This report does not establish a cause, a universal slowdown, or a typechecker speedup.",
  };
}

export function renderSummary(summary) {
  const lines = [
    `# Historical SFC parse replay: runner ${summary.runner}`,
    "",
    `Base: \`${summary.baseSha}\`; head: \`${summary.headSha}\`.`,
    "",
    "Six balanced adjacent AB/BA pairs per case; head/base below 1 is faster.",
    "",
    "| Case | Paired median | Conditional 95% interval | Observed range | Base first | Head first |",
    "| --- | ---: | --- | --- | ---: | ---: |",
  ];
  for (const row of summary.rows) {
    const ci = row.conditionalPairedInterval;
    const format = (value) => value.toFixed(4);
    lines.push(
      `| ${row.name} | ${format(row.medianPairedRatio)} | ${format(ci.lower)}–${format(ci.upper)} | ${row.observedRange.map(format).join("–")} | ${format(row.baseFirstMedian)} | ${format(row.headFirstMedian)} |`,
    );
  }
  return `${lines.join("\n")}\n\n${summary.limitation}\n`;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [input, jsonOutput, markdownOutput] = process.argv.slice(2);
  assert.ok(input && jsonOutput && markdownOutput, "usage: summary INPUT JSON MARKDOWN");
  const summary = summarizeRunner(JSON.parse(readFileSync(input, "utf8")));
  writeFileSync(jsonOutput, `${JSON.stringify(summary, null, 2)}\n`);
  writeFileSync(markdownOutput, renderSummary(summary));
}
