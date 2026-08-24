// Loader for the flat davinci bench reports the harness exports (one
// `<bench_id>.json` per bench, shaped by davinci-bench.schema.json), used by
// tools/davinci/bench-compare.mjs for both the current and the baseline set.
//
// The loader re-validates the fields the gate reads instead of trusting the
// exporter: a truncated or hand-edited report must fail the gate loudly
// rather than compare as a suspiciously fast run. A missing directory is an
// empty set, which is how "no baseline recorded yet" reaches the gate.

import fs from "node:fs";
import path from "node:path";

import { BENCH_ID, fail } from "./bench-config.mjs";

function integerOrNull(value) {
  return value === null || (Number.isSafeInteger(value) && value >= 0);
}

export function loadReports(dir, label) {
  const reports = new Map();
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return reports; // a missing directory is an empty report set
  }
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith(".json")) continue;
    const file = path.join(dir, entry.name);
    let report;
    try {
      report = JSON.parse(fs.readFileSync(file, "utf8"));
    } catch {
      fail(`${label} report ${file} is not valid JSON`);
    }
    const stem = entry.name.slice(0, -".json".length);
    if (report == null || typeof report !== "object" || Array.isArray(report)) {
      fail(`${label} report ${file} is not an object`);
    }
    if (report.bench_id !== stem) {
      fail(
        `${label} report ${file} has bench_id ${JSON.stringify(report.bench_id)} (must match the file name)`,
      );
    }
    if (!BENCH_ID.test(stem)) fail(`${label} report ${file} has an invalid bench id`);
    const wall = report.wall_ns;
    if (
      wall == null ||
      typeof wall !== "object" ||
      !Number.isSafeInteger(wall.p50) ||
      wall.p50 < 0 ||
      !Number.isSafeInteger(wall.p95) ||
      wall.p95 < 0
    ) {
      fail(`${label} report ${file} has no integer wall_ns.p50/p95`);
    }
    if (!integerOrNull(report.allocs)) {
      fail(`${label} report ${file} has a non-integer, non-null allocs`);
    }
    if (!integerOrNull(report.alloc_bytes_peak)) {
      fail(`${label} report ${file} has a non-integer, non-null alloc_bytes_peak`);
    }
    if (!integerOrNull(report.rss_peak_bytes)) {
      fail(`${label} report ${file} has a non-integer, non-null rss_peak_bytes`);
    }
    reports.set(stem, { file, ...report });
  }
  return reports;
}
