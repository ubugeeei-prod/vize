// Loader for the flat davinci bench reports the harness exports (one
// `<bench_id>.json` per bench, shaped by davinci-bench.schema.json), used by
// tools/davinci/bench-compare.mjs for both the current and the baseline set.
//
// The loader re-validates the fields the gate reads instead of trusting the
// exporter: a truncated or hand-edited report must fail the gate loudly
// rather than compare as a suspiciously fast run. A missing directory is an
// empty set, which is how "no baseline recorded yet" reaches the gate; other
// directory read failures are configuration errors.

import fs from "node:fs";
import path from "node:path";

import { BENCH_ID, fail } from "./bench-config.mjs";

const PLATFORM = /^[a-z0-9_]+$/u;
const REPORT_FIELDS = [
  "alloc_bytes_peak",
  "allocs",
  "bench_id",
  "fixture",
  "harness_version",
  "platform",
  "rss_peak_bytes",
  "wall_ns",
];
const WALL_NS_FIELDS = ["p50", "p95"];

function integerOrNull(value) {
  return value === null || (Number.isSafeInteger(value) && value >= 0);
}

function errorCode(error) {
  return error != null && typeof error === "object" && "code" in error ? error.code : undefined;
}

function assertNoExtraFields(value, allowed, where) {
  const extras = Object.keys(value)
    .filter((key) => !allowed.includes(key))
    .sort();
  if (extras.length > 0) {
    fail(`${where} has unknown fields ${extras.join(", ")} (allowed: ${allowed.join(", ")})`);
  }
}

export function loadReports(dir, label) {
  const reports = new Map();
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch (error) {
    if (errorCode(error) === "ENOENT") return reports;
    const reason = error instanceof Error ? error.message : String(error);
    fail(`${label} reports directory ${dir} cannot be read: ${reason}`);
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
    assertNoExtraFields(report, REPORT_FIELDS, `${label} report ${file}`);
    if (report.bench_id !== stem) {
      fail(
        `${label} report ${file} has bench_id ${JSON.stringify(report.bench_id)} (must match the file name)`,
      );
    }
    if (!BENCH_ID.test(stem)) fail(`${label} report ${file} has an invalid bench id`);
    if (typeof report.fixture !== "string" || report.fixture.length === 0) {
      fail(`${label} report ${file} has no valid fixture`);
    }
    if (typeof report.platform !== "string" || !PLATFORM.test(report.platform)) {
      fail(`${label} report ${file} has no valid platform`);
    }
    if (typeof report.harness_version !== "string" || report.harness_version.length === 0) {
      fail(`${label} report ${file} has no valid harness_version`);
    }
    const wall = report.wall_ns;
    if (wall == null || typeof wall !== "object" || Array.isArray(wall)) {
      fail(`${label} report ${file} has no integer wall_ns.p50/p95`);
    }
    assertNoExtraFields(wall, WALL_NS_FIELDS, `${label} report ${file} wall_ns`);
    if (
      !Number.isSafeInteger(wall.p50) ||
      wall.p50 < 0 ||
      !Number.isSafeInteger(wall.p95) ||
      wall.p95 < 0
    ) {
      fail(`${label} report ${file} has no integer wall_ns.p50/p95`);
    }
    if (wall.p95 < wall.p50) {
      fail(`${label} report ${file} has wall_ns.p95 below wall_ns.p50`);
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
