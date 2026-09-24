#!/usr/bin/env node
/**
 * Optional P3-6 reference-runner comparison at one exact Vize commit.
 * Rust medians come from this run's Criterion export; the official compiler
 * is measured in this same job over the identical checked-in sources.
 */

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { compareVaporNativePairs, parseCritcmpExport } from "./criterion-baselines.mjs";

const require = createRequire(import.meta.url);
const repo = resolve(import.meta.dirname, "../../..");
const FIXTURES = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "vapor-compiler-fixtures.json"), "utf8"),
);
const EXPECTED = [
  "components",
  "control_flow",
  "events",
  "expressions",
  "spreads",
  "templates",
  "text_runs",
];

export function validateFixtures(fixtures = FIXTURES) {
  assert.ok(Array.isArray(fixtures), "Vapor fixtures must be an array");
  const names = fixtures.map(({ name, source }) => {
    assert.match(name, /^[a-z_]+$/);
    assert.ok(typeof source === "string" && source.length > 0);
    return name;
  });
  assert.deepEqual([...names].sort(), EXPECTED, "Vapor fixture set must match Criterion");
  return fixtures;
}

export function median(values) {
  assert.ok(values.length > 0 && values.every((value) => Number.isFinite(value) && value > 0));
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}

export function compareThreeLanes(criterionExport, officialMeasurements, fixtures = FIXTURES) {
  validateFixtures(fixtures);
  const pairs = compareVaporNativePairs(criterionExport, criterionExport);
  assert.deepEqual(
    Object.keys(officialMeasurements).sort(),
    EXPECTED,
    "Official measurements must cover the complete shared corpus",
  );
  return pairs.map((pair) => {
    const officialNs = officialMeasurements[pair.fixture];
    assert.ok(Number.isFinite(officialNs) && officialNs > 0);
    return {
      fixture: pair.fixture,
      sourceSha256: createHash("sha256")
        .update(fixtures.find(({ name }) => name === pair.fixture).source)
        .digest("hex"),
      s3Ns: pair.headNativeNs,
      retainedNs: pair.headRetainedNs,
      officialNs,
      s3OverRetained: pair.headNativeNs / pair.headRetainedNs,
      s3OverOfficial: pair.headNativeNs / officialNs,
    };
  });
}

export function renderComparison(result) {
  const lines = [
    "## Davinci Vapor vs retained vs official compiler-vapor (report-only)",
    "",
    `Vize commit: \`${result.headSha}\`; official \`@vue/compiler-vapor@${result.compilerVersion}\`; runner: \`${result.runner}\`.`,
    "",
    "Identical source, module mode and prefix identifiers; Rust S3/retained medians are from this job's Criterion export, and the official JavaScript compiler is timed in this job after warmup. Timing excludes process startup and package loading. Different runtimes and sequential sampling make the cross-runtime ratio diagnostic, not a promotion gate.",
    "",
    "| Fixture | S3 | Retained | Official | S3 / retained | S3 / official |",
    "| --- | ---: | ---: | ---: | ---: | ---: |",
  ];
  for (const row of result.rows) {
    lines.push(
      `| ${row.fixture} | ${formatNs(row.s3Ns)} | ${formatNs(row.retainedNs)} | ${formatNs(row.officialNs)} | ${row.s3OverRetained.toFixed(3)}x | ${row.s3OverOfficial.toFixed(3)}x |`,
    );
  }
  lines.push(
    "",
    "The `spreads` source emits different native and retained programs; its ratios cannot establish equal-output speed. The other rows still require a repeatable runner baseline and reviewed numeric budget before P3-6 promotion.",
    "",
  );
  return lines.join("\n");
}

function formatNs(value) {
  return `${(value / 1_000).toFixed(2)} µs`;
}

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    assert.ok(key?.startsWith("--") && value, `Invalid argument: ${key ?? "<missing>"}`);
    args[key.slice(2)] = value;
  }
  return args;
}

function measureOfficial(compiler, fixtures) {
  const options = { mode: "module", prefixIdentifiers: true };
  const measurements = {};
  for (const { name, source } of fixtures) {
    for (let index = 0; index < 100; index++) compiler.compile(source, options);
    const samples = [];
    for (let sample = 0; sample < 9; sample++) {
      let codeBytes = 0;
      const start = process.hrtime.bigint();
      for (let index = 0; index < 100; index++) {
        const result = compiler.compile(source, options);
        codeBytes += result.code.length;
      }
      samples.push(Number(process.hrtime.bigint() - start) / 100);
      assert.ok(codeBytes > 0, `Official compiler produced no code for ${name}`);
    }
    measurements[name] = median(samples);
  }
  return measurements;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  assert.match(args["head-sha"] ?? "", /^[0-9a-f]{40}$/);
  assert.ok(args.criterion && args.out && args.summary);
  const checkoutSha = execFileSync("git", ["-C", repo, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  assert.equal(checkoutSha, args["head-sha"], "benchmark checkout differs from requested SHA");
  const criterionExport = parseCritcmpExport(readFileSync(resolve(args.criterion), "utf8"), "head");
  const fixtures = validateFixtures();
  const compilerVersion = require("@vue/compiler-vapor/package.json").version;
  assert.equal(compilerVersion, "3.6.0-beta.10", "official compiler pin changed");
  assert.equal(compilerVersion, require("vue/package.json").version);
  const compiler = require("@vue/compiler-vapor");
  const rows = compareThreeLanes(criterionExport, measureOfficial(compiler, fixtures), fixtures);
  const result = {
    schemaVersion: 1,
    headSha: checkoutSha,
    compilerVersion,
    runner: process.env.RUNNER_NAME ?? "local",
    methodology:
      "same-run Criterion Rust medians and 9x100 official JS compile batches, 100 warmups",
    rows,
  };
  const summary = renderComparison(result);
  writeFileSync(resolve(args.out), `${JSON.stringify(result, null, 2)}\n`);
  writeFileSync(resolve(args.summary), summary);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
  else process.stdout.write(summary);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    main();
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
}
