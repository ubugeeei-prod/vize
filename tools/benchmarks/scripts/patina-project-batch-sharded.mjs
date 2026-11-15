import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { availableParallelism } from "node:os";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import {
  CLI_ARGS,
  assertProcess,
  cleanEnvironment,
  inventory,
  median,
  parseRss,
  profileCounts,
  readAuthority,
  sha256,
} from "./patina-project-batch-lib.mjs";

const [baseInput, headInput, baseSha, headSha, outputInput, productRootInput] =
  process.argv.slice(2);
assert.ok(
  baseInput && headInput && baseSha && headSha && outputInput && productRootInput,
  "usage: patina-project-batch-sharded.mjs BASE_BIN HEAD_BIN BASE_SHA HEAD_SHA OUTPUT_DIRECTORY PRODUCT_ROOT",
);
for (const sha of [baseSha, headSha]) assert.match(sha, /^[a-f0-9]{40}$/u);
assert.notEqual(baseSha, headSha);
assert.equal(
  process.platform,
  "linux",
  "this paired lane requires the Linux GNU time observer",
);

const harness = fileURLToPath(new URL("../../../", import.meta.url));
const productRoot = fs.realpathSync(productRootInput);
const fixture = path.join(harness, "tools/benchmarks/fixtures/patina-project-batch");
const output = path.resolve(outputInput);
assert.ok(!fs.existsSync(output), "each benchmark must own a fresh evidence directory");
fs.mkdirSync(output, { recursive: true });
const consumer = path.join(output, "consumer");
const frozen = readAuthority(fixture);
const cohortRows = JSON.parse(fs.readFileSync(path.join(fixture, "cohort.stdout"), "utf8"));
const cohortStdout = Buffer.from(JSON.stringify(cohortRows, null, 2));

fs.mkdirSync(consumer, { recursive: true });
for (const name of ["package.json", "package-lock.json"])
  fs.copyFileSync(
    path.join(fixture, name === "package-lock.json" ? name : `${name}.txt`),
    path.join(consumer, name),
  );
const shardDirectories = [];
for (let cohort = 0; cohort < frozen.authority.cohorts; cohort++) {
  const name = `cohort-${String(cohort).padStart(2, "0")}`;
  const directory = path.join(consumer, "shards", name);
  shardDirectories.push({ cohort, name, directory });
  fs.mkdirSync(directory, { recursive: true });
  fs.copyFileSync(path.join(fixture, "tsconfig.json.txt"), path.join(directory, "tsconfig.json"));
  for (const [relative, text] of frozen.seed) {
    const file = path.join(directory, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, text);
  }
}

const binaries = { base: fs.realpathSync(baseInput), head: fs.realpathSync(headInput) };
const corsa = fs.realpathSync(
  path.join(
    productRoot,
    "node_modules/@typescript",
    `typescript-${process.platform}-${process.arch}`,
    "lib/tsc",
  ),
);
const runtimePackage = JSON.parse(
  fs.readFileSync(path.join(path.dirname(corsa), "../package.json"), "utf8"),
);
assert.equal(
  runtimePackage.version,
  "7.0.2",
  "frozen request vectors require the captured checker runtime",
);
const env = cleanEnvironment(process.env, corsa);
const dependencyInstall = spawnSync(
  "npm",
  ["ci", "--ignore-scripts", "--no-audit", "--no-fund"],
  { cwd: consumer, env, encoding: "utf8", timeout: 120_000 },
);
fs.writeFileSync(
  path.join(output, "dependency-install.log"),
  `${dependencyInstall.stdout ?? ""}${dependencyInstall.stderr ?? ""}`,
);
assert.equal(dependencyInstall.error, undefined);
assert.equal(dependencyInstall.status, 0, dependencyInstall.stderr);
assert.equal(
  JSON.parse(fs.readFileSync(path.join(consumer, "node_modules/vue/package.json"), "utf8"))
    .version,
  "3.5.44",
);

const inputPrefixes = [
  "node_modules",
  "package.json",
  "package-lock.json",
  ...shardDirectories.flatMap(({ name }) => [
    path.join("shards", name, "src"),
    path.join("shards", name, "tsconfig.json"),
  ]),
];
const inputs = inventory(consumer, inputPrefixes);
const binaryHashes = Object.fromEntries(
  Object.entries(binaries).map(([side, binary]) => [side, sha256(fs.readFileSync(binary))]),
);
const runtimeHash = sha256(fs.readFileSync(corsa));
const authorityHash = sha256(fs.readFileSync(path.join(fixture, "authority.json")));
fs.writeFileSync(
  path.join(output, "provenance.json"),
  `${JSON.stringify(
    {
      schema: "vize.patina-project-batch-sharded-provenance",
      version: 1,
      baseSha,
      headSha,
      harnessSha: process.env.GITHUB_SHA ?? null,
      profile: "ci-opt-release-thin-lto-cgu16",
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      availableParallelism: availableParallelism(),
      rayonThreads: "default unpinned",
      shards: shardDirectories.length,
      binaries,
      binaryHashes,
      checker: { path: corsa, version: runtimePackage.version, sha256: runtimeHash },
      authoritySha256: authorityHash,
      inputInventorySha256: sha256(JSON.stringify(inputs)),
    },
    null,
    2,
  )}\n`,
);
fs.writeFileSync(path.join(output, "complete-expected.stdout"), frozen.stdout);
fs.writeFileSync(path.join(output, "inputs-before.json"), `${JSON.stringify(inputs, null, 2)}\n`);
fs.copyFileSync(path.join(fixture, "authority.json"), path.join(output, "authority.json"));

function parseGnuTime(text) {
  const value = (label) => {
    const match = text.match(new RegExp(`^\\s*${label}: ([0-9.]+)\\s*$`, "mu"));
    assert.ok(match, `GNU time must report ${label}`);
    const number = Number(match[1]);
    assert.ok(Number.isFinite(number) && number >= 0);
    return number;
  };
  return {
    userSeconds: value("User time \\(seconds\\)"),
    systemSeconds: value("System time \\(seconds\\)"),
    maximumRssKiB: parseRss(text),
  };
}

function aggregateProfiles(profiles) {
  const aggregate = new Map();
  for (const profile of profiles) {
    for (const row of profileCounts(profile)) {
      const identity = JSON.stringify([row.key, row.attribution]);
      const current = aggregate.get(identity) ?? { ...row, count: 0 };
      current.count += row.count;
      aggregate.set(identity, current);
    }
  }
  return [...aggregate.values()].sort((a, b) =>
    JSON.stringify([a.key, a.attribution]).localeCompare(
      JSON.stringify([b.key, b.attribution]),
    ),
  );
}

const samples = [];
function run(side, label, { profile = false, record = true } = {}) {
  const started = performance.now();
  const rows = [];
  const resources = [];
  const profiles = [];
  for (const { cohort, name, directory } of shardDirectories) {
    const shardLabel = `${label}-${name}`;
    fs.appendFileSync(
      path.join(output, "progress.jsonl"),
      `${JSON.stringify({ side, label, shard: name, phase: "start" })}\n`,
    );
    const args = [...CLI_ARGS];
    if (profile)
      args.push("--profile-json", path.join(output, `${shardLabel}.profile.json`));
    const rssFile = path.join(output, `${shardLabel}.rss.txt`);
    const temp = fs.mkdtempSync(
      path.join(process.env.RUNNER_TEMP ?? output, `patina-${shardLabel}-`),
    );
    const shardEnv = {
      ...env,
      TMPDIR: temp,
      TMP: temp,
      TEMP: temp,
      XDG_CACHE_HOME: path.join(temp, "xdg-cache"),
    };
    const result = spawnSync("/usr/bin/time", ["-v", "-o", rssFile, "--", binaries[side], ...args], {
      cwd: directory,
      env: shardEnv,
      timeout: 180_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    fs.rmSync(temp, { recursive: true, force: true });
    fs.writeFileSync(path.join(output, `${shardLabel}.stderr`), result.stderr ?? "");
    try {
      assertProcess(result, cohortStdout);
    } catch (error) {
      fs.writeFileSync(path.join(output, `${shardLabel}.unexpected.stdout`), result.stdout ?? "");
      throw error;
    }
    const shardRows = JSON.parse(result.stdout.toString());
    assert.equal(shardRows.length, frozen.authority.filesPerCohort);
    rows.push(
      ...shardRows.map((row) => ({
        ...row,
        file: `src/cohort-${String(cohort).padStart(2, "0")}/${row.file.slice(4)}`,
      })),
    );
    resources.push(parseGnuTime(fs.readFileSync(rssFile, "utf8")));
    if (profile)
      profiles.push(
        JSON.parse(fs.readFileSync(path.join(output, `${shardLabel}.profile.json`), "utf8")),
      );
    fs.appendFileSync(
      path.join(output, "progress.jsonl"),
      `${JSON.stringify({ side, label, shard: name, phase: "complete" })}\n`,
    );
  }
  const stdout = Buffer.from(JSON.stringify(rows, null, 2));
  assert.deepEqual(
    stdout,
    frozen.stdout,
    "all shards must reconstruct every ordered public diagnostic byte",
  );
  const elapsedMs = performance.now() - started;
  const row = {
    side,
    label,
    elapsedMs,
    stdoutSha256: sha256(stdout),
    aggregateCpuSeconds: resources.reduce(
      (sum, resource) => sum + resource.userSeconds + resource.systemSeconds,
      0,
    ),
    maximumShardRssKiB: Math.max(...resources.map((resource) => resource.maximumRssKiB)),
    sumShardMaximumRssKiB: resources.reduce(
      (sum, resource) => sum + resource.maximumRssKiB,
      0,
    ),
  };
  if (profile) row.requestPhaseCounts = aggregateProfiles(profiles);
  if (record) {
    samples.push(row);
    fs.appendFileSync(path.join(output, "samples.jsonl"), `${JSON.stringify(row)}\n`);
  }
  return row;
}

for (const side of ["base", "head"]) run(side, `${side}-warmup`, { record: false });
const pairs = [];
for (let pair = 0; pair < 5; pair++) {
  const current = {};
  for (const side of pair % 2 === 0 ? ["base", "head"] : ["head", "base"])
    current[side] = run(side, `${side}-pair-${pair}`);
  pairs.push({
    pair,
    baseMs: current.base.elapsedMs,
    headMs: current.head.elapsedMs,
    ratio: current.head.elapsedMs / current.base.elapsedMs,
    baseCpuSeconds: current.base.aggregateCpuSeconds,
    headCpuSeconds: current.head.aggregateCpuSeconds,
    cpuRatio: current.head.aggregateCpuSeconds / current.base.aggregateCpuSeconds,
    baseMaximumShardRssKiB: current.base.maximumShardRssKiB,
    headMaximumShardRssKiB: current.head.maximumShardRssKiB,
    rssRatio: current.head.maximumShardRssKiB / current.base.maximumShardRssKiB,
  });
}

const profileSamples = [];
for (let pair = 0; pair < 2; pair++) {
  for (const side of pair % 2 === 0 ? ["base", "head"] : ["head", "base"])
    profileSamples.push(run(side, `${side}-profile-${pair}`, { profile: true }));
}
const requestPhaseCounts = Object.fromEntries(
  ["base", "head"].map((side) => {
    const vectors = profileSamples
      .filter((sample) => sample.side === side)
      .map((sample) => sample.requestPhaseCounts);
    assert.equal(vectors.length, 2);
    assert.deepEqual(vectors[1], vectors[0], `${side} request vector must repeat exactly`);
    return [side, vectors[0]];
  }),
);
assert.deepEqual(
  requestPhaseCounts.head,
  requestPhaseCounts.base,
  "non-target frozen corpus must retain its complete aggregate checker request vector",
);

const after = inventory(consumer, inputPrefixes);
fs.writeFileSync(path.join(output, "inputs-after.json"), `${JSON.stringify(after, null, 2)}\n`);
assert.deepEqual(
  after,
  inputs,
  "authored corpus, configuration or installed dependency bytes changed during measurement",
);
for (const side of ["base", "head"])
  assert.equal(sha256(fs.readFileSync(binaries[side])), binaryHashes[side]);
assert.equal(sha256(fs.readFileSync(corsa)), runtimeHash);

const ratio = median(pairs.map((pair) => pair.ratio));
const cpuRatio = median(pairs.map((pair) => pair.cpuRatio));
const rssRatio = median(pairs.map((pair) => pair.rssRatio));
const report = {
  schema: "vize.patina-project-batch-sharded-measurement",
  version: 1,
  baseSha,
  headSha,
  harnessSha: process.env.GITHUB_SHA ?? null,
  profile: "ci-opt-release-thin-lto-cgu16",
  platform: process.platform,
  arch: process.arch,
  availableParallelism: availableParallelism(),
  rayonThreads: "default unpinned",
  shards: shardDirectories.length,
  files: frozen.authority.filesPerCohort * frozen.authority.cohorts,
  diagnostics: frozen.authority.diagnosticsPerCohort * frozen.authority.cohorts,
  completeStdoutSha256: frozen.authority.expandedStdoutSha256,
  authoritySha256: authorityHash,
  inputInventorySha256: sha256(JSON.stringify(inputs)),
  pairs,
  samples,
  baseMedianMs: median(pairs.map((pair) => pair.baseMs)),
  headMedianMs: median(pairs.map((pair) => pair.headMs)),
  pairedMedianRatio: ratio,
  changePercent: (ratio - 1) * 100,
  baseMedianAggregateCpuSeconds: median(pairs.map((pair) => pair.baseCpuSeconds)),
  headMedianAggregateCpuSeconds: median(pairs.map((pair) => pair.headCpuSeconds)),
  pairedMedianCpuRatio: cpuRatio,
  cpuChangePercent: (cpuRatio - 1) * 100,
  baseMedianMaximumShardRssKiB: median(
    pairs.map((pair) => pair.baseMaximumShardRssKiB),
  ),
  headMedianMaximumShardRssKiB: median(
    pairs.map((pair) => pair.headMaximumShardRssKiB),
  ),
  pairedMedianRssRatio: rssRatio,
  rssChangePercent: (rssRatio - 1) * 100,
  requestPhaseCounts,
  requestCountTotal: Object.fromEntries(
    Object.entries(requestPhaseCounts).map(([side, rows]) => [
      side,
      rows.reduce((sum, row) => sum + row.count, 0),
    ]),
  ),
  rssScope:
    "Sequential immutable cohorts; maximumShardRssKiB is the largest GNU time peak RSS among all ten shards, while CPU is summed across every shard.",
  profileScope:
    "Two untimed profile passes per side; every 31-phase checker request vector is summed across all shards and must repeat and match between base and head.",
};
fs.writeFileSync(path.join(output, "measurement.json"), `${JSON.stringify(report, null, 2)}\n`);
fs.writeFileSync(
  path.join(output, "summary.md"),
  `## Sharded type-aware project batch\n\nExact optimized source comparison, ${report.files} immutable files and ${report.diagnostics} complete ordered diagnostics across ${report.shards} deterministic cohorts.\n\n| | Base | Head | Paired change |\n| --- | ---: | ---: | ---: |\n| Median elapsed | ${report.baseMedianMs.toFixed(3)} ms | ${report.headMedianMs.toFixed(3)} ms | ${report.changePercent.toFixed(2)}% |\n| Median aggregate CPU | ${report.baseMedianAggregateCpuSeconds.toFixed(3)} s | ${report.headMedianAggregateCpuSeconds.toFixed(3)} s | ${report.cpuChangePercent.toFixed(2)}% |\n| Median maximum shard RSS | ${report.baseMedianMaximumShardRssKiB.toFixed(0)} KiB | ${report.headMedianMaximumShardRssKiB.toFixed(0)} KiB | ${report.rssChangePercent.toFixed(2)}% |\n| Aggregate checker requests | ${report.requestCountTotal.base} | ${report.requestCountTotal.head} | exact vector match |\n\nFive alternating pairs after a warmup per side. Two separate profile passes per side reproduce the exact complete aggregate request vector. Every shard reconstructs the frozen ${report.completeStdoutSha256} complete-output hash, and all input, dependency, binary and checker hashes remain unchanged.\n`,
);
console.log(
  JSON.stringify({
    output,
    baseMedianMs: report.baseMedianMs,
    headMedianMs: report.headMedianMs,
    pairedMedianRatio: ratio,
    pairedMedianCpuRatio: cpuRatio,
    pairedMedianRssRatio: rssRatio,
    requestCountTotal: report.requestCountTotal,
  }),
);
assert.ok(ratio <= 1.05, `sharded type-aware latency regressed ${report.changePercent.toFixed(2)}%, budget 5%`);
assert.ok(cpuRatio <= 1.05, `sharded type-aware CPU regressed ${report.cpuChangePercent.toFixed(2)}%, budget 5%`);
assert.ok(rssRatio <= 1.05, `sharded type-aware RSS regressed ${report.rssChangePercent.toFixed(2)}%, budget 5%`);
