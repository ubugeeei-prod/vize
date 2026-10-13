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
import type { compile as vaporCompile } from "@vue/compiler-vapor";

import { compareVaporNativePairs, parseCritcmpExport } from "./criterion-baselines.ts";
import type { CritcmpExport } from "./criterion-baselines.ts";
import { isRecord } from "./script-types.ts";
import type { CliArgs } from "./script-types.ts";

type VaporFixture = { name: string; source: string };
type CompileOptions = { mode: "module"; prefixIdentifiers: boolean };
/** The one `@vue/compiler-vapor` entry point timed here, as the package declares it. */
type VaporCompiler = { compile: typeof vaporCompile };
type ComparisonRow = {
  fixture: string;
  sourceSha256: string;
  s3Ns: number;
  retainedNs: number;
  officialNs: number;
  s3OverRetained: number;
  s3OverOfficial: number;
};
type ComparisonReport = {
  headSha: string;
  compilerVersion: string;
  runner: string;
  rows: ComparisonRow[];
};

const isFixtureList = (value: unknown): value is VaporFixture[] =>
  Array.isArray(value) &&
  value.every(
    (entry: unknown) =>
      isRecord(entry) && typeof entry.name === "string" && typeof entry.source === "string",
  );
const isVaporCompiler = (value: unknown): value is VaporCompiler =>
  isRecord(value) && typeof value.compile === "function";

const require = createRequire(import.meta.url);
const repo = resolve(import.meta.dirname, "../../..");
const FIXTURES: unknown = JSON.parse(
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

export function validateFixtures(fixtures: unknown = FIXTURES): VaporFixture[] {
  // Every entry must carry string name and source fields before their values are checked.
  assert.ok(isFixtureList(fixtures), "Vapor fixtures must be an array");
  const names = fixtures.map(({ name, source }) => {
    assert.match(name, /^[a-z_]+$/);
    assert.ok(typeof source === "string" && source.length > 0);
    return name;
  });
  assert.deepEqual(
    [...names].sort((left, right) => left.localeCompare(right)),
    EXPECTED,
    "Vapor fixture set must match Criterion",
  );
  return fixtures;
}

export function median(values: number[]): number {
  assert.ok(values.length > 0 && values.every((value) => Number.isFinite(value) && value > 0));
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}

export function compareThreeLanes(
  criterionExport: CritcmpExport,
  officialMeasurements: Record<string, number>,
  fixtures: unknown = FIXTURES,
): ComparisonRow[] {
  const validFixtures = validateFixtures(fixtures);
  const pairs = compareVaporNativePairs(criterionExport, criterionExport);
  assert.deepEqual(
    Object.keys(officialMeasurements).sort((left, right) => left.localeCompare(right)),
    EXPECTED,
    "Official measurements must cover the complete shared corpus",
  );
  return pairs.map((pair) => {
    const officialNs = officialMeasurements[pair.fixture];
    assert.ok(Number.isFinite(officialNs) && officialNs > 0);
    return {
      fixture: pair.fixture,
      sourceSha256: createHash("sha256")
        // compareVaporNativePairs() only yields the validated EXPECTED fixture names.
        .update(validFixtures.find(({ name }) => name === pair.fixture)!.source)
        .digest("hex"),
      s3Ns: pair.headNativeNs,
      retainedNs: pair.headRetainedNs,
      officialNs,
      s3OverRetained: pair.headNativeNs / pair.headRetainedNs,
      s3OverOfficial: pair.headNativeNs / officialNs,
    };
  });
}

export function renderComparison(result: ComparisonReport): string {
  const lines = [
    "## Davinci Vapor vs retained vs official compiler-vapor (report-only)",
    "",
    `Vize commit: \`${result.headSha}\`; official \`@vue/compiler-vapor@${result.compilerVersion}\`; runner: \`${result.runner}\`.`,
    "",
    "Identical source, module mode and prefix identifiers; Rust L3/retained medians are from this job's Criterion export, and the official JavaScript compiler is timed in this job after warmup. Timing excludes process startup and package loading. Different runtimes and sequential sampling make the cross-runtime ratio diagnostic, not a promotion gate.",
    "",
    "| Fixture | L3 | Retained | Official | L3 / retained | L3 / official |",
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

function formatNs(value: number): string {
  return `${(value / 1_000).toFixed(2)} µs`;
}

function parseArgs(argv: string[]): CliArgs {
  const args: CliArgs = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    assert.ok(key?.startsWith("--") && value, `Invalid argument: ${key ?? "<missing>"}`);
    args[key.slice(2)] = value;
  }
  return args;
}

function measureOfficial(compiler: VaporCompiler, fixtures: VaporFixture[]) {
  const options: CompileOptions = { mode: "module", prefixIdentifiers: true };
  const measurements: Record<string, number> = {};
  for (const { name, source } of fixtures) {
    for (let index = 0; index < 100; index++) compiler.compile(source, options);
    const samples: number[] = [];
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

function main(): void {
  const args = parseArgs(process.argv.slice(2));
  assert.match(args["head-sha"] ?? "", /^[0-9a-f]{40}$/);
  assert.ok(args.criterion && args.out && args.summary);
  const checkoutSha = execFileSync("git", ["-C", repo, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  assert.equal(checkoutSha, args["head-sha"], "benchmark checkout differs from requested SHA");
  const criterionExport = parseCritcmpExport(readFileSync(resolve(args.criterion), "utf8"), "head");
  const fixtures = validateFixtures();
  const compilerPackage: unknown = require("@vue/compiler-vapor/package.json");
  const compilerVersion = isRecord(compilerPackage) ? compilerPackage.version : undefined;
  assert.equal(compilerVersion, "3.6.0-beta.10", "official compiler pin changed");
  const vuePackage: unknown = require("vue/package.json");
  assert.equal(compilerVersion, isRecord(vuePackage) ? vuePackage.version : undefined);
  const compiler: unknown = require("@vue/compiler-vapor");
  assert.ok(isVaporCompiler(compiler), "official compiler-vapor has no compile()");
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
